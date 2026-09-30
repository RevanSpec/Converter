//! Binaire : huit bits par octet.

use crate::options::{Choice, OptionKind, OptionSpec};
use crate::text::as_text;
use crate::{Category, Codec, CodecError, CodecMeta, ErrorCode, Options};

const LAYOUT: OptionSpec = OptionSpec {
    id: "layout",
    label: "Format des octets",
    kind: OptionKind::Choice {
        default: "grouped",
        choices: &[
            Choice {
                value: "grouped",
                label: "Groupes de 8 bits",
            },
            Choice {
                value: "continuous",
                label: "Séquence continue",
            },
        ],
    },
};

static META: CodecMeta = CodecMeta {
    id: "binary",
    label: "Binaire",
    icon: "01",
    category: Category::Bytes,
    aliases: &["bin", "binaire"],
    reversible: true,
    encodes_text: false,
    options: &[LAYOUT],
};

pub struct Binary;

impl Codec for Binary {
    fn meta(&self) -> &'static CodecMeta {
        &META
    }

    fn encode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        let joiner = if options.choice(&LAYOUT)? == "grouped" {
            " "
        } else {
            ""
        };
        let bytes: Vec<String> = input.iter().map(|byte| format!("{byte:08b}")).collect();
        Ok(bytes.join(joiner).into_bytes())
    }

    /// Accepte les séparateurs blancs, `,` et `-` entre les bits.
    fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        let mut bits = Vec::with_capacity(input.len());
        for (index, c) in as_text(input)?.chars().enumerate() {
            match c {
                '0' => bits.push(0u8),
                '1' => bits.push(1u8),
                c if c.is_whitespace() || c == ',' || c == '-' => {}
                _ => {
                    return Err(CodecError::at(
                        ErrorCode::InvalidCharacter,
                        index,
                        format!(
                            "Caractère binaire invalide « {c} » en position {} : seuls 0 et 1 sont autorisés.",
                            index + 1
                        ),
                    ))
                }
            }
        }

        if !bits.len().is_multiple_of(8) {
            return Err(CodecError::new(
                ErrorCode::InvalidLength,
                format!(
                    "Nombre de bits invalide ({}) : il faut un multiple de 8.",
                    bits.len()
                ),
            ));
        }

        Ok(bits
            .chunks(8)
            .map(|byte| byte.iter().fold(0u8, |acc, &bit| (acc << 1) | bit))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codecs::testing::{dec, enc};

    #[test]
    fn both_layouts_roundtrip() {
        for layout in ["grouped", "continuous"] {
            let options = Options::new().with("layout", layout);
            let encoded = enc(&Binary, "Tauri Rust 🦀", &options).unwrap();
            assert_eq!(dec(&Binary, &encoded, &options).unwrap(), "Tauri Rust 🦀");
        }
        assert_eq!(enc(&Binary, "A", &Options::new()).unwrap(), "01000001");
    }

    #[test]
    fn invalid_character_is_located_in_the_typed_text() {
        let error = dec(&Binary, "01000001 0100000x", &Options::new()).unwrap_err();
        assert_eq!(error.span.map(|s| s.start), Some(16));
        assert!(error.message.contains("position 17"), "{}", error.message);
        assert_eq!(
            dec(&Binary, "01001002", &Options::new()).unwrap_err().code,
            ErrorCode::InvalidCharacter
        );
    }
}
