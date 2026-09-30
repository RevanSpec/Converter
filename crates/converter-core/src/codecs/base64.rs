//! Base64 (RFC 4648), alphabet standard ou URL-safe.

use crate::options::{Choice, OptionKind, OptionSpec};
use crate::text::{as_text, strip_whitespace};
use crate::{Category, Codec, CodecError, CodecMeta, ErrorCode, Options};
use base64::alphabet;
use base64::engine::{general_purpose, DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use base64::{DecodeError, Engine as _};

const ALPHABET: OptionSpec = OptionSpec {
    id: "alphabet",
    label: "Mode d'encodage",
    kind: OptionKind::Choice {
        default: "standard",
        choices: &[
            Choice {
                value: "standard",
                label: "Standard (RFC 4648)",
            },
            Choice {
                value: "url_safe",
                label: "URL Safe (-_)",
            },
        ],
    },
};

/// Sans padding, le Base64 URL-safe est celui des JWT.
const PADDING: OptionSpec = OptionSpec {
    id: "padding",
    label: "Padding « = »",
    kind: OptionKind::Bool { default: true },
};

static META: CodecMeta = CodecMeta {
    id: "base64",
    label: "Base64",
    icon: "B64",
    category: Category::Bytes,
    aliases: &["b64"],
    reversible: true,
    encodes_text: false,
    options: &[ALPHABET, PADDING],
};

/// Décodeurs tolérants : le « = » final est facultatif (JWT, paramètres d'URL…).
const LENIENT_STANDARD: GeneralPurpose = GeneralPurpose::new(
    &alphabet::STANDARD,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);
const LENIENT_URL_SAFE: GeneralPurpose = GeneralPurpose::new(
    &alphabet::URL_SAFE,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

pub struct Base64;

impl Codec for Base64 {
    fn meta(&self) -> &'static CodecMeta {
        &META
    }

    fn encode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        let engine = match (options.choice(&ALPHABET)?, options.bool(&PADDING)?) {
            ("url_safe", true) => &general_purpose::URL_SAFE,
            ("url_safe", false) => &general_purpose::URL_SAFE_NO_PAD,
            (_, true) => &general_purpose::STANDARD,
            (_, false) => &general_purpose::STANDARD_NO_PAD,
        };
        Ok(engine.encode(input).into_bytes())
    }

    /// Avec ou sans padding ; l'alphabet est déduit des caractères présents.
    fn decode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        let prefer_url_safe = options.choice(&ALPHABET)? == "url_safe";
        decode_base64(as_text(input)?, 0, prefer_url_safe)
    }
}

/// Décode `text`, dont le premier caractère est à l'index `first` du texte saisi : les
/// positions des erreurs portent sur ce texte saisi (un Data URI, par exemple).
pub(crate) fn decode_base64(
    text: &str,
    first: usize,
    prefer_url_safe: bool,
) -> Result<Vec<u8>, CodecError> {
    let (chars, positions) = strip_whitespace(text);
    let cleaned: String = chars.iter().collect();
    let url_safe = cleaned.contains(['-', '_']);
    let standard = cleaned.contains(['+', '/']);
    if url_safe && standard {
        return Err(CodecError::new(
            ErrorCode::MixedAlphabets,
            "Base64 invalide : le texte mélange l'alphabet standard (+ /) et l'alphabet URL-safe (- _).",
        ));
    }

    let engine = if url_safe || (prefer_url_safe && !standard) {
        &LENIENT_URL_SAFE
    } else {
        &LENIENT_STANDARD
    };

    // Caractère à l'octet `offset` du texte nettoyé, et son index dans le texte saisi.
    let locate = |offset: usize| {
        let index = cleaned.get(..offset).map_or(0, |s| s.chars().count());
        let c = chars.get(index).copied().unwrap_or('?');
        (c, first + positions.get(index).copied().unwrap_or(index))
    };

    engine.decode(&cleaned).map_err(|error| match error {
            DecodeError::InvalidByte(offset, _) => {
                let (c, index) = locate(offset);
                CodecError::at(
                    ErrorCode::InvalidCharacter,
                    index,
                    format!("Caractère Base64 invalide « {c} » en position {}.", index + 1),
                )
            }
            DecodeError::InvalidLastSymbol(offset, _) => {
                let (c, index) = locate(offset);
                CodecError::at(
                    ErrorCode::InvalidLength,
                    index,
                    format!(
                        "Dernier caractère Base64 « {c} » (position {}) incohérent : le texte est peut-être tronqué.",
                        index + 1
                    ),
                )
            }
            DecodeError::InvalidLength(length) => CodecError::new(
                ErrorCode::InvalidLength,
                format!(
                    "Longueur Base64 impossible ({length} caractères utiles) : le texte est peut-être tronqué."
                ),
            ),
            DecodeError::InvalidPadding => CodecError::new(
                ErrorCode::InvalidPadding,
                "Padding Base64 invalide : les « = » doivent terminer le texte.",
            ),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codecs::testing::{dec, enc};

    #[test]
    fn url_safe_without_padding_is_the_jwt_form() {
        let jwt = Options::new()
            .with("alphabet", "url_safe")
            .with("padding", false);
        assert_eq!(enc(&Base64, "<<???>>", &jwt).unwrap(), "PDw_Pz8-Pg");
        assert_eq!(
            dec(&Base64, "PDw_Pz8-Pg", &Options::new()).unwrap(),
            "<<???>>"
        );
    }

    #[test]
    fn padding_is_optional_when_decoding() {
        assert_eq!(dec(&Base64, "SGVsbG8", &Options::new()).unwrap(), "Hello");
        assert_eq!(dec(&Base64, "SGVsbG8=", &Options::new()).unwrap(), "Hello");
    }

    #[test]
    fn errors_are_located_in_the_typed_text() {
        let error = dec(&Base64, "SG V$bG8=", &Options::new()).unwrap_err();
        assert_eq!(error.span.map(|s| s.start), Some(4));
        assert!(error.message.contains("position 5"), "{}", error.message);
        assert_eq!(
            dec(&Base64, "PDw/Pz8-Pg", &Options::new())
                .unwrap_err()
                .code,
            ErrorCode::MixedAlphabets
        );
    }
}
