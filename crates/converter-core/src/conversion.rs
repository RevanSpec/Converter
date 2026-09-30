use crate::{CodecError, Direction, ErrorCode, Options, Registry};
use serde::{Deserialize, Serialize};

/// Demande de conversion envoyée par l'interface.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct ConvertRequest {
    /// Identifiant ou alias du format.
    pub codec: String,
    pub direction: Direction,
    pub input: String,
    #[serde(default)]
    pub options: Options,
}

/// Résultat d'une conversion : du texte si les octets produits sont de l'UTF-8 valide,
/// sinon ces octets en hexadécimal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Output {
    Text { text: String },
    Bytes { hex: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct ConvertResponse {
    pub output: Output,
    pub input_chars: usize,
    pub input_bytes: usize,
    /// Caractères du texte produit, ou nombre d'octets s'il ne s'agit pas de texte.
    pub output_chars: usize,
    pub output_bytes: usize,
}

pub fn convert(
    registry: &Registry,
    request: &ConvertRequest,
) -> Result<ConvertResponse, CodecError> {
    let codec = registry.get(&request.codec).ok_or_else(|| {
        CodecError::new(
            ErrorCode::UnknownCodec,
            format!("Format inconnu : « {} ».", request.codec),
        )
    })?;

    let input = request.input.as_bytes();
    let bytes = match request.direction {
        Direction::Encode => codec.encode(input, &request.options)?,
        Direction::Decode => codec.decode(input, &request.options)?,
    };

    let output_bytes = bytes.len();
    let (output, output_chars) = match String::from_utf8(bytes) {
        Ok(text) => {
            let chars = text.chars().count();
            (Output::Text { text }, chars)
        }
        Err(error) => {
            let bytes = error.into_bytes();
            let hex = bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<Vec<_>>()
                .join(" ");
            (Output::Bytes { hex }, bytes.len())
        }
    };

    Ok(ConvertResponse {
        output,
        input_chars: request.input.chars().count(),
        input_bytes: input.len(),
        output_chars,
        output_bytes,
    })
}
