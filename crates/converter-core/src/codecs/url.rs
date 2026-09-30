//! Encodage-pourcent des URL (RFC 3986).

use crate::options::{Choice, OptionKind, OptionSpec};
use crate::text::as_text;
use crate::{Category, Codec, CodecError, CodecMeta, ErrorCode, Options};

const MODE: OptionSpec = OptionSpec {
    id: "mode",
    label: "Mode",
    kind: OptionKind::Choice {
        default: "component",
        choices: &[
            Choice {
                value: "component",
                label: "Composant (tout encoder)",
            },
            Choice {
                value: "uri",
                label: "URI complète (garde :/?#&=)",
            },
        ],
    },
};

/// Convention des formulaires HTML (application/x-www-form-urlencoded).
const PLUS_SPACE: OptionSpec = OptionSpec {
    id: "plus_space",
    label: "« + » = espace",
    kind: OptionKind::Bool { default: false },
};

static META: CodecMeta = CodecMeta {
    id: "url",
    label: "URL Encode",
    icon: "%20",
    category: Category::Web,
    aliases: &["urlencode", "percent"],
    reversible: true,
    encodes_text: false,
    options: &[MODE, PLUS_SPACE],
};

/// Caractères qui structurent une URI et que le mode « URI complète » laisse tels quels,
/// comme `encodeURI` en JavaScript.
const URI_RESERVED: &[u8] = b"!#$&'()*+,/:;=?@";

pub struct Url;

impl Codec for Url {
    fn meta(&self) -> &'static CodecMeta {
        &META
    }

    fn encode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        let keep_reserved = options.choice(&MODE)? == "uri";
        let plus_space = options.bool(&PLUS_SPACE)?;
        let mut encoded = Vec::with_capacity(input.len() * 3);
        for &byte in input {
            let unreserved = byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte);
            // Avec « + » = espace, un « + » littéral doit être encodé pour ne pas devenir une espace.
            let kept_reserved =
                keep_reserved && URI_RESERVED.contains(&byte) && !(plus_space && byte == b'+');
            if unreserved || kept_reserved {
                encoded.push(byte);
            } else if plus_space && byte == b' ' {
                encoded.push(b'+');
            } else {
                encoded.extend_from_slice(format!("%{byte:02X}").as_bytes());
            }
        }
        Ok(encoded)
    }

    fn decode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        let plus_space = options.bool(&PLUS_SPACE)?;
        let chars: Vec<char> = as_text(input)?.chars().collect();
        let mut bytes = Vec::with_capacity(chars.len());
        let mut i = 0;
        while i < chars.len() {
            match chars[i] {
                '%' => {
                    let digits: String = chars[i + 1..].iter().take(2).collect();
                    let byte = (digits.len() == 2)
                        .then(|| u8::from_str_radix(&digits, 16).ok())
                        .flatten()
                        .ok_or_else(|| {
                            CodecError::spanning(
                                ErrorCode::InvalidCharacter,
                                i,
                                i + 1 + digits.chars().count(),
                                format!(
                                    "Séquence « %{digits} » invalide en position {} : « % » doit être suivi de deux chiffres hexadécimaux.",
                                    i + 1
                                ),
                            )
                        })?;
                    bytes.push(byte);
                    i += 3;
                }
                '+' if plus_space => {
                    bytes.push(b' ');
                    i += 1;
                }
                c => {
                    let mut buffer = [0u8; 4];
                    bytes.extend_from_slice(c.encode_utf8(&mut buffer).as_bytes());
                    i += 1;
                }
            }
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codecs::testing::{dec, enc};

    #[test]
    fn component_mode_encodes_everything_but_unreserved() {
        let url = "https://example.com/?q=bonjour le monde&lang=fr";
        let encoded = enc(&Url, url, &Options::new()).unwrap();
        assert_eq!(
            encoded,
            "https%3A%2F%2Fexample.com%2F%3Fq%3Dbonjour%20le%20monde%26lang%3Dfr"
        );
        assert_eq!(dec(&Url, &encoded, &Options::new()).unwrap(), url);
    }

    #[test]
    fn uri_mode_keeps_the_structure() {
        let uri = Options::new().with("mode", "uri");
        assert_eq!(
            enc(&Url, "https://example.com/été?a=1&b=2#fin", &uri).unwrap(),
            "https://example.com/%C3%A9t%C3%A9?a=1&b=2#fin"
        );
    }

    #[test]
    fn plus_means_space_only_when_asked() {
        let form = Options::new().with("plus_space", true);
        assert_eq!(enc(&Url, "a b+c", &form).unwrap(), "a+b%2Bc");
        assert_eq!(dec(&Url, "a+b%2Bc", &form).unwrap(), "a b+c");
        assert_eq!(dec(&Url, "a+b", &Options::new()).unwrap(), "a+b");
    }

    #[test]
    fn invalid_escape_is_located() {
        let error = dec(&Url, "abc%2", &Options::new()).unwrap_err();
        assert_eq!(error.span.map(|s| (s.start, s.end)), Some((3, 5)));
    }
}
