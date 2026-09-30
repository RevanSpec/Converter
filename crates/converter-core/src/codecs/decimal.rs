//! Octets en décimal : une valeur de 0 à 255 par octet.

use crate::text::{as_text, number_tokens};
use crate::{Category, Codec, CodecError, CodecMeta, ErrorCode, Options};

static META: CodecMeta = CodecMeta {
    id: "decimal",
    label: "Octets (décimal)",
    icon: "#10",
    category: Category::Bytes,
    aliases: &["asciidec", "ascii_dec", "dec"],
    reversible: true,
    encodes_text: false,
    options: &[],
};

pub struct Decimal;

impl Codec for Decimal {
    fn meta(&self) -> &'static CodecMeta {
        &META
    }

    fn encode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        let values: Vec<String> = input.iter().map(u8::to_string).collect();
        Ok(values.join(" ").into_bytes())
    }

    /// Valeurs séparées par des blancs, `,` ou `;`.
    fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        number_tokens(as_text(input)?)
            .enumerate()
            .map(|(element, (start, end, token))| {
                token.parse::<u8>().map_err(|_| {
                    CodecError::spanning(
                        ErrorCode::InvalidNumber,
                        start,
                        end,
                        format!(
                            "Valeur décimale invalide « {token} » (élément {}) : un octet vaut de 0 à 255.",
                            element + 1
                        ),
                    )
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codecs::testing::{dec, enc};

    #[test]
    fn roundtrips_and_locates_out_of_range_values() {
        assert_eq!(
            enc(&Decimal, "Rust", &Options::new()).unwrap(),
            "82 117 115 116"
        );
        assert_eq!(
            dec(&Decimal, "82, 117;115 116", &Options::new()).unwrap(),
            "Rust"
        );
        let error = dec(&Decimal, "72 256", &Options::new()).unwrap_err();
        assert_eq!(error.span.map(|s| (s.start, s.end)), Some((3, 6)));
    }
}
