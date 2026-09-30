use crate::codec::MAX_OUTPUT_BYTES;
use crate::{Codec, CodecError, Direction, ErrorCode, Options, Registry};
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

impl Output {
    /// Texte si `bytes` est de l'UTF-8 valide, sinon ses octets en hexadécimal ; avec la
    /// longueur affichée : caractères du texte, ou nombre d'octets.
    pub fn from_bytes(bytes: Vec<u8>) -> (Self, usize) {
        match String::from_utf8(bytes) {
            Ok(text) => {
                let chars = text.chars().count();
                (Output::Text { text }, chars)
            }
            Err(error) => {
                let bytes = error.into_bytes();
                (
                    Output::Bytes {
                        hex: spaced_hex(&bytes),
                    },
                    bytes.len(),
                )
            }
        }
    }
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
    let codec = registry
        .get(&request.codec)
        .ok_or_else(|| unknown_codec(&request.codec))?;

    let input = request.input.as_bytes();
    let bytes = apply(codec, request.direction, input, &request.options)?;
    let output_bytes = bytes.len();
    let (output, output_chars) = Output::from_bytes(bytes);

    Ok(ConvertResponse {
        output,
        input_chars: request.input.chars().count(),
        input_bytes: input.len(),
        output_chars,
        output_bytes,
    })
}

/// Encode ou décode selon `direction`, puis vérifie que la sortie reste sous le plafond.
pub(crate) fn apply(
    codec: &dyn Codec,
    direction: Direction,
    input: &[u8],
    options: &Options,
) -> Result<Vec<u8>, CodecError> {
    let bytes = match direction {
        Direction::Encode => codec.encode(input, options)?,
        Direction::Decode => codec.decode(input, options)?,
    };
    if bytes.len() > MAX_OUTPUT_BYTES {
        return Err(too_large(bytes.len()));
    }
    Ok(bytes)
}

pub(crate) fn unknown_codec(name: &str) -> CodecError {
    CodecError::new(
        ErrorCode::UnknownCodec,
        format!("Format inconnu : « {name} »."),
    )
}

/// Erreur d'une sortie qui dépasse (ou dépasserait) [`MAX_OUTPUT_BYTES`].
pub(crate) fn too_large(bytes: usize) -> CodecError {
    CodecError::new(
        ErrorCode::OutputTooLarge,
        format!(
            "La sortie ({}) dépasse le plafond de {} par conversion.",
            megabytes(bytes),
            megabytes(MAX_OUTPUT_BYTES)
        ),
    )
}

/// Octets en hexadécimal minuscule, séparés par des espaces : « ff d8 ff e0 ».
pub(crate) fn spaced_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 3);
    for (index, byte) in bytes.iter().enumerate() {
        if index > 0 {
            hex.push(' ');
        }
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

/// Taille lisible, en mégaoctets décimaux comme dans l'interface : « 100 Mo », « 1,5 Mo ».
pub(crate) fn megabytes(bytes: usize) -> String {
    let tenths = (bytes as f64 / 100_000.0).round() as u64;
    if tenths.is_multiple_of(10) {
        format!("{} Mo", tenths / 10)
    } else {
        format!("{},{} Mo", tenths / 10, tenths % 10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Category, CodecMeta};

    /// Format de test dont l'encodage produit un octet de trop.
    struct Oversized;

    static OVERSIZED: CodecMeta = CodecMeta {
        id: "oversized",
        label: "Démesuré",
        icon: "∞",
        category: Category::Text,
        aliases: &[],
        reversible: true,
        encodes_text: false,
        options: &[],
    };

    impl Codec for Oversized {
        fn meta(&self) -> &'static CodecMeta {
            &OVERSIZED
        }

        fn encode(&self, _input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
            Ok(vec![0; MAX_OUTPUT_BYTES + 1])
        }

        fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
            Ok(input.to_vec())
        }
    }

    #[test]
    fn output_above_the_limit_is_refused() {
        let error = apply(&Oversized, Direction::Encode, b"x", &Options::new()).unwrap_err();
        assert_eq!(error.code, ErrorCode::OutputTooLarge);
        assert!(error.message.contains("100 Mo"), "{}", error.message);
        assert!(apply(&Oversized, Direction::Decode, b"x", &Options::new()).is_ok());
    }

    #[test]
    fn sizes_are_written_in_decimal_megabytes() {
        assert_eq!(megabytes(100_000_000), "100 Mo");
        assert_eq!(megabytes(1_530_000), "1,5 Mo");
        assert_eq!(megabytes(40_000), "0 Mo");
    }

    #[test]
    fn bytes_that_are_not_text_are_shown_in_hex() {
        assert_eq!(
            Output::from_bytes(vec![0xff, 0xd8]),
            (
                Output::Bytes {
                    hex: "ff d8".to_string()
                },
                2
            )
        );
        assert_eq!(
            Output::from_bytes("été".as_bytes().to_vec()),
            (
                Output::Text {
                    text: "été".to_string()
                },
                3
            )
        );
    }
}
