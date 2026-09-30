//! Data URI (RFC 2397) : `data:[<type MIME>][;base64],<données>`.

use super::base64::{decode_base64, Base64};
use super::url::{percent_decode, Url};
use crate::options::{Choice, OptionKind, OptionSpec};
use crate::text::as_text;
use crate::{Category, Codec, CodecError, CodecMeta, ErrorCode, Options};

const MEDIA_TYPE: OptionSpec = OptionSpec {
    id: "media_type",
    label: "Type MIME",
    kind: OptionKind::Choice {
        default: "text/plain;charset=utf-8",
        choices: &[
            Choice {
                value: "text/plain;charset=utf-8",
                label: "Texte",
            },
            Choice {
                value: "text/html;charset=utf-8",
                label: "HTML",
            },
            Choice {
                value: "image/svg+xml",
                label: "SVG",
            },
            Choice {
                value: "application/json",
                label: "JSON",
            },
            Choice {
                value: "application/octet-stream",
                label: "Octets",
            },
        ],
    },
};

const ENCODING: OptionSpec = OptionSpec {
    id: "encoding",
    label: "Données",
    kind: OptionKind::Choice {
        default: "base64",
        choices: &[
            Choice {
                value: "base64",
                label: "Base64",
            },
            Choice {
                value: "percent",
                label: "Encodage-pourcent",
            },
        ],
    },
};

static META: CodecMeta = CodecMeta {
    id: "data_uri",
    label: "Data URI",
    icon: "data:",
    category: Category::Web,
    aliases: &["datauri", "data-uri"],
    reversible: true,
    encodes_text: false,
    options: &[MEDIA_TYPE, ENCODING],
};

const PREFIX: &str = "data:";

pub struct DataUri;

impl Codec for DataUri {
    fn meta(&self) -> &'static CodecMeta {
        &META
    }

    fn encode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        let mut uri = format!("{PREFIX}{}", options.choice(&MEDIA_TYPE)?).into_bytes();
        if options.choice(&ENCODING)? == "base64" {
            uri.extend_from_slice(b";base64,");
            uri.extend(Base64.encode(input, &Options::new())?);
        } else {
            uri.push(b',');
            uri.extend(Url.encode(input, &Options::new())?);
        }
        Ok(uri)
    }

    /// Rend les données seules : le type MIME est lu, mais pas conservé. Les blancs autour
    /// du Data URI (copié d'une feuille de style, par exemple) sont ignorés.
    fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        let text = as_text(input)?;
        let leading = text.chars().take_while(|c| c.is_whitespace()).count();
        let uri = text.trim();

        let has_prefix = uri
            .get(..PREFIX.len())
            .is_some_and(|start| start.eq_ignore_ascii_case(PREFIX));
        if !has_prefix {
            return Err(CodecError::spanning(
                ErrorCode::InvalidDataUri,
                leading,
                leading + uri.chars().take(PREFIX.len()).count().max(1),
                "Data URI invalide : il doit commencer par « data: ».",
            ));
        }
        let rest = &uri[PREFIX.len()..];
        let Some(comma) = rest.find(',') else {
            return Err(CodecError::new(
                ErrorCode::InvalidDataUri,
                "Data URI invalide : la virgule qui sépare l'en-tête des données manque.",
            ));
        };

        let header = &rest[..comma];
        let payload = &rest[comma + 1..];
        // Index, dans le texte saisi, du premier caractère des données.
        let first = leading + PREFIX.len() + header.chars().count() + 1;
        let is_base64 = header
            .rsplit(';')
            .next()
            .is_some_and(|parameter| parameter.trim().eq_ignore_ascii_case("base64"));
        if is_base64 {
            decode_base64(payload, first, false)
        } else {
            percent_decode(payload, first, false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codecs::testing::{dec, enc};

    #[test]
    fn encodes_in_base64_or_percent_encoding() {
        assert_eq!(
            enc(&DataUri, "Hello", &Options::new()).unwrap(),
            "data:text/plain;charset=utf-8;base64,SGVsbG8="
        );
        let svg = Options::new()
            .with("media_type", "image/svg+xml")
            .with("encoding", "percent");
        assert_eq!(
            enc(&DataUri, "<svg/>", &svg).unwrap(),
            "data:image/svg+xml,%3Csvg%2F%3E"
        );
    }

    #[test]
    fn decodes_any_media_type() {
        assert_eq!(
            dec(&DataUri, "data:,Hello%20world", &Options::new()).unwrap(),
            "Hello world"
        );
        assert_eq!(
            dec(
                &DataUri,
                "  DATA:image/svg+xml;BASE64,PHN2Zz48\n L3N2Zz4=\n",
                &Options::new()
            )
            .unwrap(),
            "<svg></svg>"
        );
    }

    #[test]
    fn errors_are_located_in_the_whole_uri() {
        let error = dec(&DataUri, "text/plain,abc", &Options::new()).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidDataUri);
        assert_eq!(error.span.map(|s| (s.start, s.end)), Some((0, 5)));

        let error = dec(&DataUri, "data:text/plain", &Options::new()).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidDataUri);

        // « data:;base64, » fait 13 caractères : le « $ » est le 17e.
        let error = dec(&DataUri, "data:;base64,SGV$bG8=", &Options::new()).unwrap_err();
        assert_eq!(error.span.map(|s| s.start), Some(16));
        assert!(error.message.contains("position 17"), "{}", error.message);

        let error = dec(&DataUri, "data:,abc%zz", &Options::new()).unwrap_err();
        assert_eq!(error.span.map(|s| (s.start, s.end)), Some((9, 12)));
    }
}
