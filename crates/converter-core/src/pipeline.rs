//! Conversion multi-couche : des formats appliqués l'un après l'autre, sur des octets.
//!
//! La sortie d'une couche est l'entrée de la suivante. La chaîne s'arrête à la première
//! erreur, dont le message nomme la couche.

use crate::conversion::{apply as apply_codec, spaced_hex, unknown_codec};
use crate::{CodecError, Direction, ErrorCode, OptionValue, Options, Output, Registry, Span};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Taille maximale de l'aperçu de chaque couche, en octets.
pub const PREVIEW_BYTES: usize = 4096;

/// Une couche : un format, un sens et des options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct Step {
    /// Identifiant ou alias du format.
    pub codec: String,
    pub direction: Direction,
    #[serde(default)]
    pub options: Options,
    /// Une couche désactivée laisse passer les données sans les toucher.
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
}

fn enabled_by_default() -> bool {
    true
}

impl Step {
    pub fn new(codec: &str, direction: Direction) -> Self {
        Self {
            codec: codec.to_string(),
            direction,
            options: Options::new(),
            enabled: true,
        }
    }

    /// Ajoute une option ; pratique pour les tests et la ligne de commande.
    pub fn with(mut self, id: &str, value: impl Into<OptionValue>) -> Self {
        self.options = self.options.with(id, value);
        self
    }
}

/// Chaîne à exécuter sur un texte saisi.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct PipelineRequest {
    pub input: String,
    pub steps: Vec<Step>,
}

/// État d'une couche après l'exécution de la chaîne.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    /// Couche exécutée sans erreur.
    Done,
    /// Couche désactivée : les données l'ont traversée sans changement.
    Disabled,
    /// Couche en erreur : la chaîne s'est arrêtée là.
    Failed,
    /// Couche non exécutée, car une couche précédente a échoué.
    Skipped,
}

/// Compte rendu d'une couche.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct StepReport {
    pub status: StepStatus,
    /// Début de la sortie de la couche : [`PREVIEW_BYTES`] octets au plus.
    pub preview: Option<Output>,
    /// La sortie est plus longue que son aperçu.
    pub truncated: bool,
    /// Taille de la sortie, en octets.
    pub bytes: usize,
    /// La sortie est du texte UTF-8 valide.
    pub is_text: bool,
    /// Durée de la couche, en microsecondes.
    pub micros: u32,
    /// Erreur du format ; sa plage porte sur l'entrée de la couche.
    pub error: Option<CodecError>,
}

impl StepReport {
    fn empty(status: StepStatus) -> Self {
        Self {
            status,
            preview: None,
            truncated: false,
            bytes: 0,
            is_text: false,
            micros: 0,
            error: None,
        }
    }

    fn done(output: &[u8], micros: u32) -> Self {
        let end = output.len().min(PREVIEW_BYTES);
        let (preview, is_text) = match std::str::from_utf8(output) {
            Ok(text) => {
                // L'aperçu s'arrête avant un caractère coupé en deux.
                let end = (0..=end)
                    .rev()
                    .find(|&index| text.is_char_boundary(index))
                    .unwrap_or(0);
                (
                    Output::Text {
                        text: text[..end].to_string(),
                    },
                    true,
                )
            }
            Err(_) => (
                Output::Bytes {
                    hex: spaced_hex(&output[..end]),
                },
                false,
            ),
        };
        Self {
            status: StepStatus::Done,
            preview: Some(preview),
            truncated: output.len() > PREVIEW_BYTES,
            bytes: output.len(),
            is_text,
            micros,
            error: None,
        }
    }

    fn failed(error: CodecError, micros: u32) -> Self {
        Self {
            micros,
            error: Some(error),
            ..Self::empty(StepStatus::Failed)
        }
    }
}

/// Erreur d'une chaîne : la couche en cause et un message qui la nomme.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, thiserror::Error)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[error("{message}")]
pub struct PipelineError {
    /// Index de la couche, à partir de 0.
    pub step: usize,
    pub code: ErrorCode,
    /// Message précédé du numéro, du format et du sens de la couche.
    pub message: String,
    /// Plage fautive dans le texte saisi, quand la couche en erreur est celle qui le lit.
    pub span: Option<Span>,
}

impl PipelineError {
    fn named(
        registry: &Registry,
        index: usize,
        step: &Step,
        error: &CodecError,
        reads_input: bool,
    ) -> Self {
        let layer = match registry.get(&step.codec) {
            Some(codec) => format!(
                "Couche {} ({}, {})",
                index + 1,
                codec.meta().label,
                step.direction.noun()
            ),
            None => format!("Couche {}", index + 1),
        };
        Self {
            step: index,
            code: error.code,
            message: format!("{layer} : {}", error.message),
            span: if reads_input { error.span } else { None },
        }
    }
}

/// Résultat d'une chaîne, couche par couche.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct PipelineResponse {
    /// Un compte rendu par couche, dans l'ordre de la chaîne.
    pub steps: Vec<StepReport>,
    /// Sortie complète de la dernière couche ; absente si une couche a échoué.
    pub output: Option<Output>,
    pub input_chars: usize,
    pub input_bytes: usize,
    /// Caractères du texte produit, ou nombre d'octets s'il ne s'agit pas de texte.
    pub output_chars: usize,
    pub output_bytes: usize,
    pub error: Option<PipelineError>,
}

/// Exécute la chaîne sur `input` et rend la sortie de sa dernière couche active.
pub fn apply_chain(
    registry: &Registry,
    steps: &[Step],
    input: &[u8],
) -> Result<Vec<u8>, PipelineError> {
    let mut data = input.to_vec();
    let mut reads_input = true;
    for (index, step) in steps.iter().enumerate().filter(|(_, step)| step.enabled) {
        data = run_step(registry, step, &data)
            .map_err(|error| PipelineError::named(registry, index, step, &error, reads_input))?;
        reads_input = false;
    }
    Ok(data)
}

/// Exécute la chaîne demandée par l'interface, avec un compte rendu par couche : aperçu,
/// taille, texte ou non, durée.
pub fn run_pipeline(registry: &Registry, request: &PipelineRequest) -> PipelineResponse {
    let mut data = request.input.as_bytes().to_vec();
    let mut reports = Vec::with_capacity(request.steps.len());
    let mut error = None;
    // La prochaine couche active lit-elle directement le texte saisi ?
    let mut reads_input = true;

    for (index, step) in request.steps.iter().enumerate() {
        if error.is_some() {
            reports.push(StepReport::empty(StepStatus::Skipped));
            continue;
        }
        if !step.enabled {
            reports.push(StepReport::empty(StepStatus::Disabled));
            continue;
        }

        let started = Instant::now();
        let result = run_step(registry, step, &data);
        let micros = u32::try_from(started.elapsed().as_micros()).unwrap_or(u32::MAX);
        match result {
            Ok(output) => {
                reports.push(StepReport::done(&output, micros));
                data = output;
            }
            Err(codec_error) => {
                error = Some(PipelineError::named(
                    registry,
                    index,
                    step,
                    &codec_error,
                    reads_input,
                ));
                reports.push(StepReport::failed(codec_error, micros));
            }
        }
        reads_input = false;
    }

    let (output, output_chars, output_bytes) = if error.is_none() {
        let bytes = data.len();
        let (output, chars) = Output::from_bytes(data);
        (Some(output), chars, bytes)
    } else {
        (None, 0, 0)
    };

    PipelineResponse {
        steps: reports,
        output,
        input_chars: request.input.chars().count(),
        input_bytes: request.input.len(),
        output_chars,
        output_bytes,
        error,
    }
}

/// Chaîne inverse : les couches dans l'ordre inverse, chacune dans l'autre sens. Appliquée
/// à la sortie de la chaîne, elle redonne son entrée. Refusée si une couche n'a pas
/// d'opération inverse.
pub fn invert_chain(registry: &Registry, steps: &[Step]) -> Result<Vec<Step>, PipelineError> {
    steps
        .iter()
        .enumerate()
        .rev()
        .map(|(index, step)| {
            let codec = registry.get(&step.codec).ok_or_else(|| {
                PipelineError::named(registry, index, step, &unknown_codec(&step.codec), false)
            })?;
            if !codec.meta().reversible {
                return Err(PipelineError {
                    step: index,
                    code: ErrorCode::IrreversibleStep,
                    message: format!(
                        "Inversion impossible : la couche {} ({}) n'a pas d'opération inverse.",
                        index + 1,
                        codec.meta().label
                    ),
                    span: None,
                });
            }
            Ok(Step {
                direction: step.direction.reversed(),
                ..step.clone()
            })
        })
        .collect()
}

fn run_step(registry: &Registry, step: &Step, input: &[u8]) -> Result<Vec<u8>, CodecError> {
    let codec = registry
        .get(&step.codec)
        .ok_or_else(|| unknown_codec(&step.codec))?;
    apply_codec(codec, step.direction, input, &step.options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{registry, Category, Codec, CodecMeta};
    use Direction::{Decode, Encode};

    fn run(input: &str, steps: Vec<Step>) -> PipelineResponse {
        run_pipeline(
            registry(),
            &PipelineRequest {
                input: input.to_string(),
                steps,
            },
        )
    }

    fn text(output: &Option<Output>) -> &str {
        match output {
            Some(Output::Text { text }) => text,
            other => panic!("texte attendu : {other:?}"),
        }
    }

    #[test]
    fn base64_then_hex_gives_hello() {
        let response = run(
            "NDg2NTZjNmM2Zg==",
            vec![Step::new("base64", Decode), Step::new("hex", Decode)],
        );
        assert_eq!(text(&response.output), "Hello");
        assert_eq!(response.error, None);
        assert_eq!(text(&response.steps[0].preview), "48656c6c6f");
        assert_eq!(
            response
                .steps
                .iter()
                .map(|report| report.status)
                .collect::<Vec<_>>(),
            [StepStatus::Done, StepStatus::Done]
        );
    }

    #[test]
    fn stops_at_the_first_error_and_names_its_layer() {
        // « aGVsbG8= » donne « hello », qui n'est pas de l'hexadécimal.
        let response = run(
            "aGVsbG8=",
            vec![
                Step::new("b64", Decode),
                Step::new("hex", Decode),
                Step::new("url", Encode),
            ],
        );
        let error = response.error.expect("erreur attendue");
        assert_eq!(error.step, 1);
        assert_eq!(error.code, ErrorCode::InvalidCharacter);
        assert!(
            error.message.starts_with(
                "Couche 2 (Hexadécimal, décodage) : Caractère hexadécimal invalide « h »"
            ),
            "{}",
            error.message
        );
        // La plage porte sur la sortie de la couche 1, pas sur le texte saisi.
        assert_eq!(error.span, None);
        assert_eq!(response.output, None);
        let statuses: Vec<_> = response.steps.iter().map(|r| r.status).collect();
        assert_eq!(
            statuses,
            [StepStatus::Done, StepStatus::Failed, StepStatus::Skipped]
        );
        assert_eq!(
            response.steps[1].error.as_ref().and_then(|e| e.span),
            Some(Span { start: 0, end: 1 })
        );
    }

    #[test]
    fn the_layer_that_reads_the_input_keeps_its_span() {
        let mut disabled = Step::new("base64", Decode);
        disabled.enabled = false;
        let response = run("48 zz", vec![disabled, Step::new("hex", Decode)]);
        let error = response.error.expect("erreur attendue");
        assert_eq!(error.span, Some(Span { start: 3, end: 4 }));
    }

    #[test]
    fn disabled_layers_let_the_data_through() {
        let mut disabled = Step::new("base64", Encode);
        disabled.enabled = false;
        let response = run("Hi", vec![disabled, Step::new("hex", Encode)]);
        assert_eq!(text(&response.output), "48 69");
        assert_eq!(response.steps[0].status, StepStatus::Disabled);
        assert_eq!(response.steps[0].preview, None);
    }

    #[test]
    fn previews_are_truncated_and_binary_is_shown_in_hex() {
        let response = run(&"é".repeat(3000), vec![Step::new("reverse", Encode)]);
        let report = &response.steps[0];
        assert!(report.truncated && report.is_text);
        assert_eq!(report.bytes, 6000);
        assert_eq!(text(&report.preview).len(), PREVIEW_BYTES);
        assert_eq!(response.output_bytes, 6000);

        let response = run("/9j/4A==", vec![Step::new("base64", Decode)]);
        let report = &response.steps[0];
        assert!(!report.is_text && !report.truncated);
        assert_eq!(
            report.preview,
            Some(Output::Bytes {
                hex: "ff d8 ff e0".to_string()
            })
        );
        assert_eq!(response.output_chars, 4);
    }

    #[test]
    fn an_empty_chain_returns_the_input() {
        let response = run("Hello", vec![]);
        assert_eq!(text(&response.output), "Hello");
        assert_eq!(apply_chain(registry(), &[], b"Hello").unwrap(), b"Hello");
    }

    #[test]
    fn unknown_formats_are_named() {
        let error = apply_chain(registry(), &[Step::new("nope", Encode)], b"x").unwrap_err();
        assert_eq!(error.code, ErrorCode::UnknownCodec);
        assert_eq!(error.message, "Couche 1 : Format inconnu : « nope ».");
    }

    #[test]
    fn inversion_reverses_order_and_direction() {
        let chain = vec![
            Step::new("base64", Decode).with("alphabet", "url_safe"),
            Step::new("hex", Decode),
        ];
        let inverse = invert_chain(registry(), &chain).unwrap();
        assert_eq!(
            inverse,
            vec![
                Step::new("hex", Encode),
                Step::new("base64", Encode).with("alphabet", "url_safe"),
            ]
        );
        let encoded = apply_chain(registry(), &inverse, b"Hello").unwrap();
        assert_eq!(apply_chain(registry(), &chain, &encoded).unwrap(), b"Hello");
    }

    /// Format sans opération inverse, comme le seront les hachages.
    struct OneWay;

    static ONE_WAY: CodecMeta = CodecMeta {
        id: "one_way",
        label: "Sens unique",
        icon: "→",
        category: Category::Text,
        aliases: &[],
        reversible: false,
        encodes_text: false,
        options: &[],
    };

    impl Codec for OneWay {
        fn meta(&self) -> &'static CodecMeta {
            &ONE_WAY
        }

        fn encode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
            Ok(input.to_vec())
        }

        fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
            Ok(input.to_vec())
        }
    }

    #[test]
    fn inversion_is_refused_for_an_irreversible_layer() {
        let registry = Registry::from_codecs(vec![Box::new(OneWay)]);
        let error = invert_chain(
            &registry,
            &[Step::new("one_way", Encode), Step::new("one_way", Encode)],
        )
        .unwrap_err();
        assert_eq!(error.code, ErrorCode::IrreversibleStep);
        assert_eq!(error.step, 1);
        assert!(
            error.message.contains("couche 2 (Sens unique)"),
            "{}",
            error.message
        );
    }
}
