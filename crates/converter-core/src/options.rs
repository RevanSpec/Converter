use crate::{CodecError, ErrorCode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Choix proposé par une option à choix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct Choice {
    pub value: &'static str,
    pub label: &'static str,
}

/// Type d'une option, avec sa valeur par défaut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OptionKind {
    Bool {
        default: bool,
    },
    Choice {
        default: &'static str,
        choices: &'static [Choice],
    },
    Int {
        default: i32,
        min: i32,
        max: i32,
    },
    /// Texte libre ; `secret` le masque à l'écran et l'exclut des exports.
    Text {
        default: &'static str,
        secret: bool,
    },
}

/// Option d'un format, décrite pour que l'interface puisse l'afficher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct OptionSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: OptionKind,
}

/// Valeur d'une option, telle qu'envoyée par l'interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(untagged)]
pub enum OptionValue {
    Bool(bool),
    Int(i32),
    Text(String),
}

impl From<bool> for OptionValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i32> for OptionValue {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<&str> for OptionValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

/// Options d'un format, par identifiant. Une option absente prend sa valeur par défaut ;
/// une option que le format ne connaît pas est ignorée.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct Options(pub BTreeMap<String, OptionValue>);

impl Options {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ajoute une option ; pratique pour les tests et la ligne de commande.
    pub fn with(mut self, id: &str, value: impl Into<OptionValue>) -> Self {
        self.0.insert(id.to_string(), value.into());
        self
    }

    pub fn bool(&self, spec: &OptionSpec) -> Result<bool, CodecError> {
        let OptionKind::Bool { default } = spec.kind else {
            return Err(wrong_kind(spec));
        };
        match self.0.get(spec.id) {
            None => Ok(default),
            Some(OptionValue::Bool(value)) => Ok(*value),
            Some(other) => Err(invalid(spec, other)),
        }
    }

    pub fn int(&self, spec: &OptionSpec) -> Result<i32, CodecError> {
        let OptionKind::Int { default, min, max } = spec.kind else {
            return Err(wrong_kind(spec));
        };
        match self.0.get(spec.id) {
            None => Ok(default),
            Some(OptionValue::Int(value)) if (min..=max).contains(value) => Ok(*value),
            Some(other) => Err(invalid(spec, other)),
        }
    }

    /// Valeur d'une option à choix ; seules les valeurs proposées sont acceptées.
    pub fn choice(&self, spec: &OptionSpec) -> Result<&'static str, CodecError> {
        let OptionKind::Choice { default, choices } = spec.kind else {
            return Err(wrong_kind(spec));
        };
        match self.0.get(spec.id) {
            None => Ok(default),
            Some(OptionValue::Text(value)) => choices
                .iter()
                .find(|choice| choice.value == value)
                .map(|choice| choice.value)
                .ok_or_else(|| invalid(spec, &OptionValue::Text(value.clone()))),
            Some(other) => Err(invalid(spec, other)),
        }
    }

    pub fn text(&self, spec: &OptionSpec) -> Result<String, CodecError> {
        let OptionKind::Text { default, .. } = spec.kind else {
            return Err(wrong_kind(spec));
        };
        match self.0.get(spec.id) {
            None => Ok(default.to_string()),
            Some(OptionValue::Text(value)) => Ok(value.clone()),
            Some(other) => Err(invalid(spec, other)),
        }
    }
}

fn invalid(spec: &OptionSpec, value: &OptionValue) -> CodecError {
    CodecError::new(
        ErrorCode::InvalidOption,
        format!(
            "Valeur {value:?} invalide pour l'option « {} ».",
            spec.label
        ),
    )
}

fn wrong_kind(spec: &OptionSpec) -> CodecError {
    CodecError::new(
        ErrorCode::Internal,
        format!(
            "L'option « {} » est lue avec un type qui n'est pas le sien.",
            spec.id
        ),
    )
}
