//! Recettes : une chaîne décrite pour être partagée ou enregistrée, sous deux formes.
//!
//! - JSON versionné, pour les fichiers :
//!   `{"v":1,"steps":[{"codec":"base64","direction":"decode","options":{},"enabled":true}]}` ;
//! - forme courte, à copier-coller : `base64:dec|hex:dec`. Une couche désactivée commence
//!   par `!`, ses options suivent entre parenthèses et les valeurs de texte sont encodées
//!   en pourcent : `!caesar:enc(shift=3)|hex:enc(separator=%3A,uppercase=true)`.
//!
//! Les deux formes désignent les formats par leur identifiant, n'écrivent que les options
//! qui diffèrent de leur valeur par défaut, et jamais les options secrètes.

use crate::codecs::percent_decode;
use crate::conversion::unknown_codec;
use crate::{
    CodecError, CodecMeta, Direction, ErrorCode, OptionKind, OptionSpec, OptionValue, Options,
    Registry, Step,
};
use serde::{Deserialize, Serialize};

/// Version actuelle du format JSON des recettes.
pub const RECIPE_VERSION: u32 = 1;

/// Longueur maximale du nom d'une recette enregistrée, en caractères.
pub const MAX_NAME_CHARS: usize = 60;

/// Recette sous sa forme JSON versionnée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct Recipe {
    /// Version du format de recette.
    pub v: u32,
    pub steps: Vec<Step>,
}

impl Recipe {
    /// Recette d'une chaîne, prête à être partagée : identifiants des formats, options
    /// différentes de leur valeur par défaut, aucune option secrète.
    pub fn export(registry: &Registry, steps: &[Step]) -> Result<Self, CodecError> {
        let steps = steps
            .iter()
            .enumerate()
            .map(|(index, step)| exported(registry, step).map_err(|error| in_layer(index, error)))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            v: RECIPE_VERSION,
            steps,
        })
    }

    /// Couches de la recette, après vérification de sa version, de ses formats et de ses
    /// options : une recette lue dans un fichier peut venir d'une autre version.
    pub fn into_steps(self, registry: &Registry) -> Result<Vec<Step>, CodecError> {
        if self.v != RECIPE_VERSION {
            return Err(CodecError::new(
                ErrorCode::InvalidRecipe,
                format!(
                    "Recette de version {} : cette version de Glass Converter lit la version {RECIPE_VERSION}.",
                    self.v
                ),
            ));
        }
        self.steps
            .into_iter()
            .enumerate()
            .map(|(index, step)| {
                let checked = || {
                    let meta = find(registry, &step.codec)?;
                    for (id, value) in &step.options.0 {
                        check_option(meta, id, value)?;
                    }
                    Ok(meta.id)
                };
                let id = checked().map_err(|error| in_layer(index, error))?;
                Ok(Step {
                    codec: id.to_string(),
                    ..step
                })
            })
            .collect()
    }
}

/// Forme courte d'une chaîne, par exemple `base64:dec|hex:dec`.
pub fn to_short(registry: &Registry, steps: &[Step]) -> Result<String, CodecError> {
    let recipe = Recipe::export(registry, steps)?;
    let layers: Vec<String> = recipe
        .steps
        .iter()
        .map(|step| {
            let mut layer = String::new();
            if !step.enabled {
                layer.push('!');
            }
            layer.push_str(&step.codec);
            layer.push_str(match step.direction {
                Direction::Encode => ":enc",
                Direction::Decode => ":dec",
            });
            if !step.options.0.is_empty() {
                let options: Vec<String> = step
                    .options
                    .0
                    .iter()
                    .map(|(id, value)| format!("{id}={}", short_value(value)))
                    .collect();
                layer.push_str(&format!("({})", options.join(",")));
            }
            layer
        })
        .collect();
    Ok(layers.join("|"))
}

/// Lit la forme courte d'une recette. Les erreurs situent le caractère fautif.
pub fn parse_short(registry: &Registry, text: &str) -> Result<Vec<Step>, CodecError> {
    let chars: Vec<char> = text.chars().collect();
    if chars.iter().all(|c| c.is_whitespace()) {
        return Err(CodecError::new(
            ErrorCode::InvalidRecipe,
            "Recette vide : écrivez par exemple « base64:dec|hex:dec ».",
        ));
    }

    let separators = chars
        .iter()
        .enumerate()
        .filter(|(_, c)| **c == '|')
        .map(|(index, _)| index);
    let mut steps = Vec::new();
    let mut start = 0;
    for end in separators.chain([chars.len()]) {
        let mut parser = Parser {
            chars: &chars,
            pos: start,
            end,
        };
        steps.push(parser.layer(registry)?);
        start = end + 1;
    }
    Ok(steps)
}

/// Recette prête à l'emploi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct Preset {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    /// Forme courte de la recette.
    pub recipe: &'static str,
}

/// Recettes prêtes à l'emploi, dans le sens du décodage : « Inverser » donne l'encodage.
pub const PRESETS: &[Preset] = &[
    Preset {
        id: "base64_double",
        name: "Base64 double",
        description: "Texte encodé deux fois en Base64.",
        recipe: "base64:dec|base64:dec",
    },
    Preset {
        id: "data_uri",
        name: "Data URI",
        description: "Données d'un Data URI (data:…;base64,… ou data:…,…).",
        recipe: "data_uri:dec",
    },
    Preset {
        id: "gzip_base64",
        name: "gzip + Base64",
        description: "Base64 d'un contenu compressé en gzip.",
        recipe: "base64:dec|gzip:dec",
    },
    Preset {
        id: "saml_redirect",
        name: "SAML (deflate + Base64 + URL)",
        description: "Paramètre SAMLRequest ou SAMLResponse de la liaison HTTP-Redirect.",
        recipe: "url:dec|base64:dec|deflate:dec",
    },
];

/// Recette enregistrée par l'utilisateur dans « Mes recettes ».
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct SavedRecipe {
    pub name: String,
    pub recipe: Recipe,
}

/// Contenu du fichier « Mes recettes » : des recettes nommées, jamais de texte saisi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeBook {
    pub v: u32,
    pub recipes: Vec<SavedRecipe>,
}

impl Default for RecipeBook {
    fn default() -> Self {
        Self {
            v: RECIPE_VERSION,
            recipes: Vec::new(),
        }
    }
}

impl RecipeBook {
    /// Ajoute la recette, ou remplace celle qui porte déjà ce nom (casse ignorée). Les
    /// recettes restent triées par nom.
    pub fn save(&mut self, name: &str, recipe: Recipe) -> Result<(), CodecError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(CodecError::new(
                ErrorCode::InvalidRecipe,
                "Donnez un nom à la recette.",
            ));
        }
        if name.chars().count() > MAX_NAME_CHARS {
            return Err(CodecError::new(
                ErrorCode::InvalidRecipe,
                format!("Nom trop long : {MAX_NAME_CHARS} caractères au plus."),
            ));
        }

        let key = name.to_lowercase();
        match self
            .recipes
            .iter_mut()
            .find(|saved| saved.name.to_lowercase() == key)
        {
            Some(saved) => {
                saved.name = name.to_string();
                saved.recipe = recipe;
            }
            None => self.recipes.push(SavedRecipe {
                name: name.to_string(),
                recipe,
            }),
        }
        self.recipes.sort_by_key(|saved| saved.name.to_lowercase());
        Ok(())
    }

    /// Retire la recette de ce nom ; indique si elle existait.
    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.recipes.len();
        self.recipes.retain(|saved| saved.name != name);
        self.recipes.len() != before
    }
}

fn find(registry: &Registry, name: &str) -> Result<&'static CodecMeta, CodecError> {
    registry
        .get(name)
        .map(|codec| codec.meta())
        .ok_or_else(|| unknown_codec(name))
}

fn exported(registry: &Registry, step: &Step) -> Result<Step, CodecError> {
    let meta = find(registry, &step.codec)?;
    let mut options = Options::new();
    for (id, value) in &step.options.0 {
        // Une option que le format ne connaît pas est sans effet : elle n'est pas exportée.
        let Some(spec) = meta.options.iter().find(|spec| spec.id == id) else {
            continue;
        };
        check_option(meta, id, value)?;
        let secret = matches!(spec.kind, OptionKind::Text { secret: true, .. });
        if !secret && !is_default(&spec.kind, value) {
            options.0.insert(id.clone(), value.clone());
        }
    }
    Ok(Step {
        codec: meta.id.to_string(),
        direction: step.direction,
        options,
        enabled: step.enabled,
    })
}

fn find_option(meta: &'static CodecMeta, id: &str) -> Result<&'static OptionSpec, CodecError> {
    meta.options
        .iter()
        .find(|spec| spec.id == id)
        .ok_or_else(|| {
            let known: Vec<&str> = meta.options.iter().map(|spec| spec.id).collect();
            let hint = if known.is_empty() {
                "ce format n'a pas d'option".to_string()
            } else {
                format!("options possibles : {}", known.join(", "))
            };
            CodecError::new(
                ErrorCode::InvalidOption,
                format!("Option inconnue « {id} » pour {} ({hint}).", meta.label),
            )
        })
}

/// Vérifie que `value` convient à l'option `id` du format.
fn check_option(
    meta: &'static CodecMeta,
    id: &str,
    value: &OptionValue,
) -> Result<&'static OptionSpec, CodecError> {
    let spec = find_option(meta, id)?;
    // Les accesseurs d'`Options` vérifient le type, la plage et les choix possibles.
    let options = Options::new().with(id, value.clone());
    match spec.kind {
        OptionKind::Bool { .. } => options.bool(spec).map(drop),
        OptionKind::Int { .. } => options.int(spec).map(drop),
        OptionKind::Choice { .. } => options.choice(spec).map(drop),
        OptionKind::Text { .. } => options.text(spec).map(drop),
    }?;
    Ok(spec)
}

fn is_default(kind: &OptionKind, value: &OptionValue) -> bool {
    match (kind, value) {
        (OptionKind::Bool { default }, OptionValue::Bool(value)) => value == default,
        (OptionKind::Int { default, .. }, OptionValue::Int(value)) => value == default,
        (OptionKind::Choice { default, .. }, OptionValue::Text(value))
        | (OptionKind::Text { default, .. }, OptionValue::Text(value)) => value == default,
        _ => false,
    }
}

fn in_layer(index: usize, error: CodecError) -> CodecError {
    CodecError::new(
        error.code,
        format!("Couche {} : {}", index + 1, error.message),
    )
}

/// Valeur d'option dans la forme courte : les caractères autres que lettres, chiffres
/// et `-_.~` sont encodés en pourcent, pour ne jamais se confondre avec `,` `)` ou `|`.
fn short_value(value: &OptionValue) -> String {
    match value {
        OptionValue::Bool(value) => value.to_string(),
        OptionValue::Int(value) => value.to_string(),
        OptionValue::Text(text) => {
            let mut encoded = String::with_capacity(text.len());
            for byte in text.bytes() {
                if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
                    encoded.push(char::from(byte));
                } else {
                    encoded.push_str(&format!("%{byte:02X}"));
                }
            }
            encoded
        }
    }
}

/// Lecture d'une couche de la forme courte, entre les index `pos` et `end` du texte.
struct Parser<'a> {
    chars: &'a [char],
    pos: usize,
    end: usize,
}

impl Parser<'_> {
    fn layer(&mut self, registry: &Registry) -> Result<Step, CodecError> {
        self.skip_whitespace();
        while self.end > self.pos && self.chars[self.end - 1].is_whitespace() {
            self.end -= 1;
        }
        if self.pos == self.end {
            // Couche vide : on montre le « | » voisin.
            let at = if self.pos < self.chars.len() {
                self.pos
            } else {
                self.pos.saturating_sub(1)
            };
            return Err(syntax(at, at + 1, "couche vide autour de ce « | »"));
        }

        let enabled = !self.eat('!');
        let (name_start, name) = self.take(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        if name.is_empty() {
            return Err(self.unexpected("nom de format attendu"));
        }
        if !self.eat(':') {
            return Err(self.unexpected(&format!(
                "« : » attendu après « {name} », puis le sens : enc ou dec"
            )));
        }
        let (direction_start, direction) = self.take(|c| c.is_alphabetic());
        let direction = match direction.to_lowercase().as_str() {
            "enc" | "encode" => Direction::Encode,
            "dec" | "decode" => Direction::Decode,
            _ => {
                return Err(syntax(
                    direction_start,
                    self.pos.max(direction_start + 1),
                    &format!("sens « {direction} » inconnu : enc ou dec attendu"),
                ))
            }
        };
        let meta = find(registry, &name).map_err(|error| {
            CodecError::spanning(
                error.code,
                name_start,
                name_start + name.chars().count(),
                error.message,
            )
        })?;

        let mut options = Options::new();
        self.skip_whitespace();
        if self.eat('(') {
            self.options(meta, &mut options)?;
        }
        if self.pos < self.end {
            return Err(self.unexpected("fin de la couche attendue"));
        }
        Ok(Step {
            codec: meta.id.to_string(),
            direction,
            options,
            enabled,
        })
    }

    /// Options entre parenthèses, la parenthèse ouvrante déjà lue.
    fn options(
        &mut self,
        meta: &'static CodecMeta,
        options: &mut Options,
    ) -> Result<(), CodecError> {
        self.skip_whitespace();
        if self.eat(')') {
            return Ok(());
        }
        loop {
            self.skip_whitespace();
            let (key_start, key) = self.take(|c| c.is_ascii_alphanumeric() || c == '_');
            if key.is_empty() {
                return Err(self.unexpected("nom d'option attendu"));
            }
            self.skip_whitespace();
            if !self.eat('=') {
                return Err(self.unexpected(&format!("« = » attendu après « {key} »")));
            }
            self.skip_whitespace();
            let (value_start, raw) = self.take(|c| c != ',' && c != ')');
            let raw = raw.trim_end();
            let value_end = value_start + raw.chars().count();
            // L'erreur d'une option porte sur l'option entière : « nom=valeur ».
            let located = |error: CodecError| {
                CodecError::spanning(
                    error.code,
                    key_start,
                    value_end.max(key_start + 1),
                    error.message,
                )
            };

            let spec = find_option(meta, &key).map_err(located)?;
            let value = parse_value(spec, raw, value_start).map_err(located)?;
            check_option(meta, &key, &value).map_err(located)?;
            if options.0.insert(key.clone(), value).is_some() {
                return Err(syntax(
                    key_start,
                    value_end.max(key_start + 1),
                    &format!("option « {key} » répétée"),
                ));
            }

            self.skip_whitespace();
            if self.eat(',') {
                continue;
            }
            if self.eat(')') {
                return Ok(());
            }
            return Err(self.unexpected("« , » ou « ) » attendu"));
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.end && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn eat(&mut self, expected: char) -> bool {
        let found = self.pos < self.end && self.chars[self.pos] == expected;
        if found {
            self.pos += 1;
        }
        found
    }

    /// Caractères suivants tant que `accept` les accepte, avec l'index du premier.
    fn take(&mut self, accept: impl Fn(char) -> bool) -> (usize, String) {
        let start = self.pos;
        while self.pos < self.end && accept(self.chars[self.pos]) {
            self.pos += 1;
        }
        (start, self.chars[start..self.pos].iter().collect())
    }

    /// Erreur sur le caractère courant, ou sur le dernier si la couche est finie.
    fn unexpected(&self, expected: &str) -> CodecError {
        match self.chars.get(self.pos).filter(|_| self.pos < self.end) {
            Some(c) => syntax(
                self.pos,
                self.pos + 1,
                &format!("{expected}, trouvé « {c} »"),
            ),
            None => {
                let at = self.end.saturating_sub(1);
                syntax(at, at + 1, &format!("{expected} après la fin de la couche"))
            }
        }
    }
}

fn parse_value(spec: &OptionSpec, raw: &str, first: usize) -> Result<OptionValue, CodecError> {
    let invalid = |expected: &str| {
        CodecError::new(
            ErrorCode::InvalidOption,
            format!(
                "Valeur « {raw} » invalide pour l'option « {} » : {expected}.",
                spec.label
            ),
        )
    };
    match spec.kind {
        OptionKind::Bool { .. } => match raw.to_lowercase().as_str() {
            "true" => Ok(OptionValue::Bool(true)),
            "false" => Ok(OptionValue::Bool(false)),
            _ => Err(invalid("true ou false attendu")),
        },
        OptionKind::Int { .. } => raw
            .parse()
            .map(OptionValue::Int)
            .map_err(|_| invalid("nombre entier attendu")),
        OptionKind::Choice { .. } | OptionKind::Text { .. } => {
            let bytes = percent_decode(raw, first, false)?;
            String::from_utf8(bytes)
                .map(OptionValue::Text)
                .map_err(|_| invalid("texte UTF-8 attendu"))
        }
    }
}

fn syntax(start: usize, end: usize, detail: &str) -> CodecError {
    CodecError::spanning(
        ErrorCode::InvalidRecipe,
        start,
        end,
        format!("Recette illisible en position {} : {detail}.", start + 1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{registry, Category, Codec, Span};
    use Direction::{Decode, Encode};

    fn short(steps: &[Step]) -> String {
        to_short(registry(), steps).unwrap()
    }

    fn parse(text: &str) -> Result<Vec<Step>, CodecError> {
        parse_short(registry(), text)
    }

    fn span(error: &CodecError) -> Option<(usize, usize)> {
        error.span.map(|Span { start, end }| (start, end))
    }

    #[test]
    fn short_form_names_formats_by_id_and_skips_defaults() {
        let mut disabled = Step::new("rot13", Encode).with("shift", 3);
        disabled.enabled = false;
        let steps = vec![
            Step::new("b64", Decode).with("alphabet", "standard"),
            Step::new("hex", Encode)
                .with("separator", ":")
                .with("uppercase", true),
            disabled,
        ];
        let text = short(&steps);
        assert_eq!(
            text,
            "base64:dec|hex:enc(separator=%3A,uppercase=true)|!caesar:enc(shift=3)"
        );

        let parsed = parse(&text).unwrap();
        assert_eq!(parsed[0], Step::new("base64", Decode));
        assert_eq!(
            parsed[1],
            Step::new("hex", Encode)
                .with("separator", ":")
                .with("uppercase", true)
        );
        assert!(!parsed[2].enabled);
        assert_eq!(short(&parsed), text);
    }

    #[test]
    fn short_form_accepts_aliases_blanks_and_long_directions() {
        let parsed = parse("  B64:Decode | hex:dec ( separator = %20 )  ").unwrap();
        assert_eq!(
            parsed,
            vec![
                Step::new("base64", Decode),
                Step::new("hex", Decode).with("separator", " "),
            ]
        );
        assert_eq!(
            parse("hex:enc(separator=)").unwrap()[0].options.0["separator"],
            OptionValue::Text(String::new())
        );
    }

    #[test]
    fn short_form_errors_point_at_the_culprit() {
        let error = parse("").unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidRecipe);

        let error = parse("base64:dec|nope:dec").unwrap_err();
        assert_eq!(error.code, ErrorCode::UnknownCodec);
        assert_eq!(span(&error), Some((11, 15)));

        let error = parse("base64:up").unwrap_err();
        assert_eq!(span(&error), Some((7, 9)));
        assert!(error.message.contains("position 8"), "{}", error.message);

        let error = parse("hex:dec||base64:dec").unwrap_err();
        assert_eq!(span(&error), Some((8, 9)));

        let error = parse("caesar:enc(shift=40)").unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidOption);
        assert_eq!(span(&error), Some((11, 19)));

        let error = parse("caesar:enc(key=3)").unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidOption);
        assert!(
            error.message.contains("options possibles : shift"),
            "{}",
            error.message
        );

        let error = parse("hex:dec(uppercase=true").unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidRecipe);

        let error = parse("hex dec").unwrap_err();
        assert_eq!(span(&error), Some((3, 4)));
    }

    #[test]
    fn json_form_is_versioned_and_checked() {
        let recipe = Recipe::export(
            registry(),
            &[Step::new("b64", Decode), Step::new("hex", Decode)],
        )
        .unwrap();
        let json = serde_json::to_string(&recipe).unwrap();
        assert_eq!(
            json,
            r#"{"v":1,"steps":[{"codec":"base64","direction":"decode","options":{},"enabled":true},{"codec":"hex","direction":"decode","options":{},"enabled":true}]}"#
        );

        // Options et `enabled` peuvent manquer ; les alias sont acceptés.
        let read: Recipe =
            serde_json::from_str(r#"{"v":1,"steps":[{"codec":"b64","direction":"decode"}]}"#)
                .unwrap();
        assert_eq!(
            read.into_steps(registry()).unwrap(),
            vec![Step::new("base64", Decode)]
        );

        let future: Recipe = serde_json::from_str(r#"{"v":2,"steps":[]}"#).unwrap();
        assert_eq!(
            future.into_steps(registry()).unwrap_err().code,
            ErrorCode::InvalidRecipe
        );
        let bad: Recipe = serde_json::from_str(
            r#"{"v":1,"steps":[{"codec":"hex","direction":"decode"},{"codec":"caesar","direction":"encode","options":{"shift":99}}]}"#,
        )
        .unwrap();
        let error = bad.into_steps(registry()).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidOption);
        assert!(
            error.message.starts_with("Couche 2 : "),
            "{}",
            error.message
        );
    }

    /// Format de test avec une option secrète, comme le seront les chiffrements.
    struct Locked;

    const PASSWORD: OptionSpec = OptionSpec {
        id: "password",
        label: "Mot de passe",
        kind: OptionKind::Text {
            default: "",
            secret: true,
        },
    };

    const HINT: OptionSpec = OptionSpec {
        id: "hint",
        label: "Indice",
        kind: OptionKind::Text {
            default: "",
            secret: false,
        },
    };

    static LOCKED: CodecMeta = CodecMeta {
        id: "locked",
        label: "Verrouillé",
        icon: "🔒",
        category: Category::Cipher,
        aliases: &[],
        reversible: true,
        encodes_text: false,
        options: &[PASSWORD, HINT],
    };

    impl Codec for Locked {
        fn meta(&self) -> &'static CodecMeta {
            &LOCKED
        }

        fn encode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
            Ok(input.to_vec())
        }

        fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
            Ok(input.to_vec())
        }
    }

    #[test]
    fn secret_options_are_never_exported() {
        let registry = Registry::from_codecs(vec![Box::new(Locked)]);
        let steps = [Step::new("locked", Encode)
            .with("password", "hunter2")
            .with("hint", "chat")];

        let text = to_short(&registry, &steps).unwrap();
        assert_eq!(text, "locked:enc(hint=chat)");
        let json = serde_json::to_string(&Recipe::export(&registry, &steps).unwrap()).unwrap();
        assert!(
            !json.contains("hunter2") && !json.contains("password"),
            "{json}"
        );
    }

    #[test]
    fn presets_are_valid_recipes() {
        for preset in PRESETS {
            let steps =
                parse(preset.recipe).unwrap_or_else(|e| panic!("{} : {}", preset.id, e.message));
            assert_eq!(short(&steps), preset.recipe, "{}", preset.id);
        }
    }

    #[test]
    fn saml_preset_reads_a_redirect_binding_request() {
        let steps = parse(PRESETS[3].recipe).unwrap();
        // `deflateRawSync` de Node.js, puis Base64, puis encodage-pourcent.
        let request = b"sylOzM0psHIsLcnIC0otLE0tLlHwdLFVijdU0rcDAA%3D%3D";
        assert_eq!(
            crate::apply_chain(registry(), &steps, request).unwrap(),
            br#"<samlp:AuthnRequest ID="_1"/>"#
        );
    }

    #[test]
    fn saved_recipes_are_named_sorted_and_replaced() {
        let recipe = |short: &str| Recipe::export(registry(), &parse(short).unwrap()).unwrap();
        let mut book = RecipeBook::default();
        book.save("SAML", recipe("url:dec")).unwrap();
        book.save("  base64 double ", recipe("base64:dec|base64:dec"))
            .unwrap();
        book.save("saml", recipe("url:dec|base64:dec")).unwrap();

        let names: Vec<&str> = book
            .recipes
            .iter()
            .map(|saved| saved.name.as_str())
            .collect();
        assert_eq!(names, ["base64 double", "saml"]);
        assert_eq!(book.recipes[1].recipe.steps.len(), 2);

        assert_eq!(
            book.save("   ", recipe("url:dec")).unwrap_err().code,
            ErrorCode::InvalidRecipe
        );
        assert!(book.save(&"x".repeat(61), recipe("url:dec")).is_err());
        assert!(book.remove("saml"));
        assert!(!book.remove("saml"));
    }
}
