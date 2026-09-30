//! Chaînes de couches et recettes (phase 3) : une chaîne suivie de son inverse redonne
//! l'entrée, et une recette se relit à l'identique sous ses deux formes.

use converter_core::{
    apply_chain, invert_chain, parse_short, registry, run_pipeline, to_short, Direction,
    OptionKind, OptionValue, Options, Output, PipelineRequest, Recipe, Step, StepStatus,
};
use proptest::prelude::*;

/// Formats dont le décodage défait exactement l'encodage, quelle que soit l'entrée : des
/// octets quelconques, ou du texte pour César et HTML.
const EXACT: &[&str] = &[
    "hex", "binary", "base64", "base32", "url", "decimal", "octal", "data_uri", "gzip", "zlib",
    "deflate", "brotli", "caesar", "html",
];

/// Formats dont la sortie n'est pas du texte.
const BINARY_OUTPUT: &[&str] = &["gzip", "zlib", "deflate", "brotli"];

/// Formats qui attendent du texte en entrée.
const TEXT_INPUT: &[&str] = &["caesar", "html"];

/// Options tirées au hasard parmi celles que le registre décrit pour ce format.
fn options_for(codec: &str) -> impl Strategy<Value = Options> {
    let meta = registry().get(codec).expect("format inconnu").meta();
    let values: Vec<BoxedStrategy<(String, OptionValue)>> = meta
        .options
        .iter()
        .map(|spec| {
            let id = spec.id.to_string();
            match spec.kind {
                OptionKind::Bool { .. } => any::<bool>()
                    .prop_map(move |value| (id.clone(), OptionValue::Bool(value)))
                    .boxed(),
                OptionKind::Int { min, max, .. } => (min..=max)
                    .prop_map(move |value| (id.clone(), OptionValue::Int(value)))
                    .boxed(),
                OptionKind::Choice { choices, .. } => {
                    prop::sample::select(choices.iter().map(|c| c.value).collect::<Vec<_>>())
                        .prop_map(move |value| (id.clone(), OptionValue::from(value)))
                        .boxed()
                }
                OptionKind::Text { default, .. } => Just((id, OptionValue::from(default))).boxed(),
            }
        })
        .collect();
    values.prop_map(|pairs| Options(pairs.into_iter().collect()))
}

/// Chaîne d'encodages de 1 à 4 couches exactes. Après une compression, les données ne
/// sont plus du texte : un format de texte y laisse sa place au Base64.
fn exact_chain() -> impl Strategy<Value = Vec<Step>> {
    prop::collection::vec(prop::sample::select(EXACT), 1..=4).prop_flat_map(|codecs| {
        let mut binary = false;
        codecs
            .into_iter()
            .map(|codec| {
                let codec = if binary && TEXT_INPUT.contains(&codec) {
                    "base64"
                } else {
                    codec
                };
                binary = BINARY_OUTPUT.contains(&codec);
                options_for(codec).prop_map(move |options| Step {
                    options,
                    ..Step::new(codec, Direction::Encode)
                })
            })
            .collect::<Vec<_>>()
    })
}

/// Chaîne quelconque : tous les formats, les deux sens, des couches désactivées.
fn any_chain() -> impl Strategy<Value = Vec<Step>> {
    let codecs: Vec<&'static str> = registry().list().iter().map(|meta| meta.id).collect();
    let step = prop::sample::select(codecs).prop_flat_map(|codec| {
        (options_for(codec), any::<bool>(), prop::bool::weighted(0.8)).prop_map(
            move |(options, encode, enabled)| Step {
                codec: codec.to_string(),
                direction: if encode {
                    Direction::Encode
                } else {
                    Direction::Decode
                },
                options,
                enabled,
            },
        )
    });
    prop::collection::vec(step, 1..=5)
}

proptest! {
    /// Critère de sortie de la phase 3.
    #[test]
    fn a_chain_followed_by_its_inverse_gives_the_input_back(
        input in any::<String>(),
        steps in exact_chain(),
    ) {
        let recipe = to_short(registry(), &steps).unwrap();
        let encoded = apply_chain(registry(), &steps, input.as_bytes())
            .map_err(|e| TestCaseError::fail(format!("{recipe} : {}", e.message)))?;
        let inverse = invert_chain(registry(), &steps)
            .map_err(|e| TestCaseError::fail(e.message))?;
        let decoded = apply_chain(registry(), &inverse, &encoded)
            .map_err(|e| TestCaseError::fail(format!("inverse de {recipe} : {}", e.message)))?;
        prop_assert_eq!(decoded, input.as_bytes(), "{}", recipe);
    }

    /// La forme courte se relit en la même chaîne, options par défaut en moins.
    #[test]
    fn short_form_reads_back_identically(steps in any_chain()) {
        let exported = Recipe::export(registry(), &steps).unwrap();
        let text = to_short(registry(), &steps).unwrap();
        prop_assert_eq!(parse_short(registry(), &text).unwrap(), exported.steps, "{}", text);
    }

    #[test]
    fn json_form_reads_back_identically(steps in any_chain()) {
        let exported = Recipe::export(registry(), &steps).unwrap();
        let json = serde_json::to_string(&exported).unwrap();
        let read: Recipe = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(read.into_steps(registry()).unwrap(), exported.steps, "{}", json);
    }
}

/// Critère de sortie de la phase 3, avec la requête telle que l'interface l'envoie.
#[test]
fn base64_then_hex_decodes_to_hello() {
    let request: PipelineRequest = serde_json::from_str(
        r#"{"input":"NDg2NTZjNmM2Zg==","steps":[
            {"codec":"base64","direction":"decode","options":{},"enabled":true},
            {"codec":"hex","direction":"decode","options":{},"enabled":true}]}"#,
    )
    .unwrap();
    let response = run_pipeline(registry(), &request);
    assert_eq!(
        response.output,
        Some(Output::Text {
            text: "Hello".to_string()
        })
    );
    assert!(response
        .steps
        .iter()
        .all(|report| report.status == StepStatus::Done));
}
