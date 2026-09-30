use serde::Serialize;

/// Nature d'une erreur. Stable : l'interface peut s'en servir pour traduire le message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// Caractère interdit dans ce format.
    InvalidCharacter,
    /// Longueur impossible : texte tronqué, chiffre manquant…
    InvalidLength,
    /// Padding (« = ») mal placé ou en nombre incohérent.
    InvalidPadding,
    /// Alphabets Base64 standard et URL-safe mélangés.
    MixedAlphabets,
    /// Nombre hors de la plage d'un octet.
    InvalidNumber,
    /// Code Morse inconnu.
    UnknownSymbol,
    /// Caractère sans équivalent dans le format visé.
    UnsupportedCharacter,
    /// Nom de domaine internationalisé refusé par IDNA.
    InvalidDomain,
    /// Séquence Punycode invalide.
    InvalidPunycode,
    /// Le format attend du texte UTF-8 et a reçu d'autres octets.
    NotText,
    /// Option inconnue du format, ou valeur hors des choix possibles.
    InvalidOption,
    /// Format absent du registre.
    UnknownCodec,
    /// Échec interne.
    Internal,
}

/// Plage fautive dans le texte saisi, en caractères Unicode comptés à partir de 0,
/// fin exclue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Erreur de conversion : un code stable, un message en français et, quand elle est
/// connue, la plage fautive dans le texte saisi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, thiserror::Error)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[error("{message}")]
pub struct CodecError {
    pub code: ErrorCode,
    pub message: String,
    pub span: Option<Span>,
}

impl CodecError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            span: None,
        }
    }

    /// Erreur qui porte sur les caractères `start..end` du texte saisi.
    pub fn spanning(code: ErrorCode, start: usize, end: usize, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            span: Some(Span { start, end }),
        }
    }

    /// Erreur qui porte sur le seul caractère d'index `index`.
    pub fn at(code: ErrorCode, index: usize, message: impl Into<String>) -> Self {
        Self::spanning(code, index, index + 1, message)
    }
}
