//! Inversion du texte, graphème par graphème.

use crate::text::as_text;
use crate::{Category, Codec, CodecError, CodecMeta, Options};
use unicode_segmentation::UnicodeSegmentation;

static META: CodecMeta = CodecMeta {
    id: "reverse",
    label: "Inversion de texte",
    icon: "⇄",
    category: Category::Text,
    aliases: &["inverse"],
    reversible: true,
    encodes_text: true,
    options: &[],
};

pub struct Reverse;

impl Codec for Reverse {
    fn meta(&self) -> &'static CodecMeta {
        &META
    }

    fn encode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        Ok(reverse_graphemes(as_text(input)?).into_bytes())
    }

    fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        Ok(reverse_graphemes(as_text(input)?).into_bytes())
    }
}

/// Un emoji composé ou une lettre suivie d'un accent combinant reste intact.
fn reverse_graphemes(text: &str) -> String {
    text.graphemes(true).rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codecs::testing::enc;

    #[test]
    fn keeps_graphemes_intact() {
        assert_eq!(
            enc(&Reverse, "antigravity", &Options::new()).unwrap(),
            "ytivargitna"
        );
        assert_eq!(enc(&Reverse, "👍🏽!", &Options::new()).unwrap(), "!👍🏽");
        assert_eq!(
            enc(&Reverse, "e\u{301}t", &Options::new()).unwrap(),
            "te\u{301}"
        );
    }
}
