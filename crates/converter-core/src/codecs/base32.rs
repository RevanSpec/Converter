//! Base32 (RFC 4648).

use crate::text::{as_text, strip_whitespace};
use crate::{Category, Codec, CodecError, CodecMeta, ErrorCode, Options};

static META: CodecMeta = CodecMeta {
    id: "base32",
    label: "Base32",
    icon: "B32",
    category: Category::Bytes,
    aliases: &["b32"],
    reversible: true,
    encodes_text: false,
    options: &[],
};

const ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

pub struct Base32;

impl Codec for Base32 {
    fn meta(&self) -> &'static CodecMeta {
        &META
    }

    fn encode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        let mut result = Vec::with_capacity(input.len().div_ceil(5) * 8);
        let mut buffer: u64 = 0;
        let mut bits_left = 0;

        for &byte in input {
            buffer = (buffer << 8) | u64::from(byte);
            bits_left += 8;
            while bits_left >= 5 {
                bits_left -= 5;
                result.push(ALPHABET[((buffer >> bits_left) & 0x1F) as usize]);
            }
        }

        if bits_left > 0 {
            result.push(ALPHABET[((buffer << (5 - bits_left)) & 0x1F) as usize]);
        }

        // Padding RFC 4648
        while !result.len().is_multiple_of(8) {
            result.push(b'=');
        }

        Ok(result)
    }

    /// En majuscules ou minuscules, avec ou sans padding.
    fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        let (chars, positions) = strip_whitespace(as_text(input)?);
        let data_len = chars.iter().position(|&c| c == '=').unwrap_or(chars.len());
        let (data, padding) = chars.split_at(data_len);

        if let Some(offset) = padding.iter().position(|&c| c != '=') {
            let index = positions[data_len + offset];
            return Err(CodecError::at(
                ErrorCode::InvalidPadding,
                index,
                format!(
                    "Caractère « {} » après le padding Base32, en position {}.",
                    chars[data_len + offset],
                    index + 1
                ),
            ));
        }

        let mut buffer: u64 = 0;
        let mut bits = 0;
        let mut bytes = Vec::with_capacity(data.len() * 5 / 8);
        for (i, &c) in data.iter().enumerate() {
            let value = match c.to_ascii_uppercase() {
                letter @ 'A'..='Z' => letter as u64 - 'A' as u64,
                digit @ '2'..='7' => digit as u64 - '2' as u64 + 26,
                _ => {
                    return Err(CodecError::at(
                        ErrorCode::InvalidCharacter,
                        positions[i],
                        format!(
                            "Caractère Base32 invalide « {c} » en position {}.",
                            positions[i] + 1
                        ),
                    ))
                }
            };
            buffer = (buffer << 5) | value;
            bits += 5;
            if bits >= 8 {
                bits -= 8;
                bytes.push((buffer >> bits) as u8);
            }
        }

        // Seules ces longueurs utiles (modulo 8) donnent un nombre entier d'octets.
        let expected_padding = match data.len() % 8 {
            0 => 0,
            2 => 6,
            4 => 4,
            5 => 3,
            7 => 1,
            _ => {
                return Err(CodecError::new(
                    ErrorCode::InvalidLength,
                    format!(
                        "Longueur Base32 impossible ({} caractères utiles) : le texte est peut-être tronqué.",
                        data.len()
                    ),
                ))
            }
        };
        if !padding.is_empty() && padding.len() != expected_padding {
            return Err(CodecError::new(
                ErrorCode::InvalidPadding,
                format!(
                    "Padding Base32 incohérent : {} « = » pour {} caractères utiles ({expected_padding} attendus).",
                    padding.len(),
                    data.len()
                ),
            ));
        }

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codecs::testing::{dec, enc};

    #[test]
    fn roundtrips_and_accepts_unpadded_lowercase() {
        let encoded = enc(&Base32, "Hello Base32!", &Options::new()).unwrap();
        assert_eq!(
            dec(&Base32, &encoded, &Options::new()).unwrap(),
            "Hello Base32!"
        );
        assert_eq!(dec(&Base32, "mzxw6", &Options::new()).unwrap(), "foo");
    }

    #[test]
    fn rejects_truncated_text_and_bad_padding() {
        let codes: Vec<ErrorCode> = ["A", "MZXW6==", "MZ=XW6", "MZ XW1==="]
            .iter()
            .map(|input| dec(&Base32, input, &Options::new()).unwrap_err().code)
            .collect();
        assert_eq!(
            codes,
            [
                ErrorCode::InvalidLength,
                ErrorCode::InvalidPadding,
                ErrorCode::InvalidPadding,
                ErrorCode::InvalidCharacter
            ]
        );
    }
}
