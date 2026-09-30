//! Hexadécimal : deux chiffres par octet.

use crate::options::{Choice, OptionKind, OptionSpec};
use crate::text::as_text;
use crate::{Category, Codec, CodecError, CodecMeta, ErrorCode, Options};

const SEPARATOR: OptionSpec = OptionSpec {
    id: "separator",
    label: "Séparateur",
    kind: OptionKind::Choice {
        default: " ",
        choices: &[
            Choice {
                value: " ",
                label: "Espace",
            },
            Choice {
                value: "",
                label: "Continu",
            },
            Choice {
                value: "0x",
                label: "0x...",
            },
            Choice {
                value: ":",
                label: "Deux-points (:)",
            },
        ],
    },
};

const UPPERCASE: OptionSpec = OptionSpec {
    id: "uppercase",
    label: "Majuscules",
    kind: OptionKind::Bool { default: false },
};

static META: CodecMeta = CodecMeta {
    id: "hex",
    label: "Hexadécimal",
    icon: "⬡",
    category: Category::Bytes,
    aliases: &["hexadecimal", "base16"],
    reversible: true,
    encodes_text: false,
    options: &[SEPARATOR, UPPERCASE],
};

pub struct Hex;

impl Codec for Hex {
    fn meta(&self) -> &'static CodecMeta {
        &META
    }

    fn encode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        let separator = options.choice(&SEPARATOR)?;
        let uppercase = options.bool(&UPPERCASE)?;
        let prefix = if separator == "0x" { "0x" } else { "" };
        let joiner = if separator == "0x" { " " } else { separator };
        let pairs: Vec<String> = input
            .iter()
            .map(|byte| {
                if uppercase {
                    format!("{prefix}{byte:02X}")
                } else {
                    format!("{prefix}{byte:02x}")
                }
            })
            .collect();
        Ok(pairs.join(joiner).into_bytes())
    }

    /// Accepte les préfixes `0x`, `\x` et `%`, et les séparateurs blancs, `:`, `,`, `-` et `;`.
    fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        let chars: Vec<char> = as_text(input)?.chars().collect();
        let mut digits = Vec::with_capacity(chars.len());
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if matches!(c, '0' | '\\') && matches!(chars.get(i + 1).copied(), Some('x' | 'X')) {
                i += 2;
                continue;
            }
            match c.to_digit(16) {
                Some(digit) => digits.push(digit as u8),
                None if c.is_whitespace() || matches!(c, '%' | ':' | ',' | '-' | ';') => {}
                None => {
                    return Err(CodecError::at(
                        ErrorCode::InvalidCharacter,
                        i,
                        format!(
                            "Caractère hexadécimal invalide « {c} » en position {}.",
                            i + 1
                        ),
                    ))
                }
            }
            i += 1;
        }

        if !digits.len().is_multiple_of(2) {
            return Err(CodecError::new(
                ErrorCode::InvalidLength,
                format!(
                    "Nombre impair de chiffres hexadécimaux ({}) : chaque octet en demande 2.",
                    digits.len()
                ),
            ));
        }

        Ok(digits
            .chunks(2)
            .map(|pair| (pair[0] << 4) | pair[1])
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codecs::testing::{dec, enc};

    #[test]
    fn every_separator_roundtrips() {
        for separator in [" ", "", "0x", ":"] {
            let options = Options::new()
                .with("separator", separator)
                .with("uppercase", true);
            let encoded = enc(&Hex, "Bonjour 🦀", &options).unwrap();
            assert_eq!(
                dec(&Hex, &encoded, &options).unwrap(),
                "Bonjour 🦀",
                "{encoded}"
            );
        }
        let continuous = Options::new().with("separator", "");
        assert_eq!(enc(&Hex, "Hi", &continuous).unwrap(), "4869");
    }

    #[test]
    fn invalid_character_is_located() {
        let error = dec(&Hex, "48 65 zz", &Options::new()).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidCharacter);
        assert_eq!(error.span.map(|s| (s.start, s.end)), Some((6, 7)));
        assert!(error.message.contains("position 7"), "{}", error.message);
    }

    #[test]
    fn decodes_bytes_that_are_not_text() {
        assert_eq!(
            Hex.decode(b"ff d8", &Options::new()).unwrap(),
            vec![0xff, 0xd8]
        );
    }
}
