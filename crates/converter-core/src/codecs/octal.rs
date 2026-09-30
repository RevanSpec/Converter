//! Octets en octal : une valeur de 000 à 377 par octet.

use crate::text::{as_text, number_tokens};
use crate::{Category, Codec, CodecError, CodecMeta, ErrorCode, Options};

static META: CodecMeta = CodecMeta {
    id: "octal",
    label: "Octets (octal)",
    icon: "#08",
    category: Category::Bytes,
    aliases: &["asciioct", "ascii_oct", "oct"],
    reversible: true,
    encodes_text: false,
    options: &[],
};

pub struct Octal;

impl Codec for Octal {
    fn meta(&self) -> &'static CodecMeta {
        &META
    }

    fn encode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        let values: Vec<String> = input.iter().map(|byte| format!("{byte:03o}")).collect();
        Ok(values.join(" ").into_bytes())
    }

    /// Valeurs séparées par des blancs, `,` ou `;`.
    fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        number_tokens(as_text(input)?)
            .enumerate()
            .map(|(element, (start, end, token))| {
                u8::from_str_radix(token, 8).map_err(|_| {
                    CodecError::spanning(
                        ErrorCode::InvalidNumber,
                        start,
                        end,
                        format!(
                            "Valeur octale invalide « {token} » (élément {}) : un octet vaut de 0 à 377 en base 8.",
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
    fn roundtrips_and_rejects_non_octal_digits() {
        assert_eq!(enc(&Octal, "Hi", &Options::new()).unwrap(), "110 151");
        assert_eq!(dec(&Octal, "110 151", &Options::new()).unwrap(), "Hi");
        assert_eq!(
            dec(&Octal, "110 158", &Options::new()).unwrap_err().code,
            ErrorCode::InvalidNumber
        );
    }
}
