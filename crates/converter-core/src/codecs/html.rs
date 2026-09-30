//! Entités HTML : échappement des caractères spéciaux, décodage de toutes les entités HTML5.

use crate::text::as_text;
use crate::{Category, Codec, CodecError, CodecMeta, Options};

static META: CodecMeta = CodecMeta {
    id: "html",
    label: "HTML Entities",
    icon: "&;",
    category: Category::Web,
    aliases: &["html_entities", "entities"],
    reversible: true,
    encodes_text: true,
    options: &[],
};

pub struct Html;

impl Codec for Html {
    fn meta(&self) -> &'static CodecMeta {
        &META
    }

    fn encode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        let text = as_text(input)?;
        let mut escaped = String::with_capacity(text.len());
        for c in text.chars() {
            match c {
                '&' => escaped.push_str("&amp;"),
                '<' => escaped.push_str("&lt;"),
                '>' => escaped.push_str("&gt;"),
                '"' => escaped.push_str("&quot;"),
                '\'' => escaped.push_str("&#39;"),
                other => escaped.push(other),
            }
        }
        Ok(escaped.into_bytes())
    }

    fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        Ok(htmlize::unescape(as_text(input)?).into_owned().into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codecs::testing::{dec, enc};

    #[test]
    fn roundtrips_and_decodes_html5_named_entities() {
        let original = "<div class=\"box\">L'éléphant & la souris</div>";
        let encoded = enc(&Html, original, &Options::new()).unwrap();
        assert_eq!(dec(&Html, &encoded, &Options::new()).unwrap(), original);
        assert_eq!(
            dec(&Html, "&eacute;t&eacute; &hellip; &mdash;", &Options::new()).unwrap(),
            "été … —"
        );
    }
}
