//! Chiffre de César : décalage des lettres ASCII dans l'alphabet (13 = ROT13).

use crate::options::{OptionKind, OptionSpec};
use crate::text::as_text;
use crate::{Category, Codec, CodecError, CodecMeta, Options};

const SHIFT: OptionSpec = OptionSpec {
    id: "shift",
    label: "Décalage César",
    kind: OptionKind::Int {
        default: 13,
        min: 1,
        max: 25,
    },
};

static META: CodecMeta = CodecMeta {
    id: "caesar",
    label: "ROT13 / César",
    icon: "ROT",
    category: Category::Cipher,
    aliases: &["rot13", "cesar", "rot"],
    reversible: true,
    encodes_text: true,
    options: &[SHIFT],
};

pub struct Caesar;

impl Codec for Caesar {
    fn meta(&self) -> &'static CodecMeta {
        &META
    }

    fn encode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        Ok(shift(as_text(input)?, options.int(&SHIFT)?).into_bytes())
    }

    fn decode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        Ok(shift(as_text(input)?, -options.int(&SHIFT)?).into_bytes())
    }
}

/// Décale les lettres ASCII ; tout autre caractère est conservé.
fn shift(text: &str, by: i32) -> String {
    let by = by.rem_euclid(26) as u8;
    text.chars()
        .map(|c| match c {
            'a'..='z' => (b'a' + (c as u8 - b'a' + by) % 26) as char,
            'A'..='Z' => (b'A' + (c as u8 - b'A' + by) % 26) as char,
            _ => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codecs::testing::{dec, enc};

    #[test]
    fn rot13_and_other_shifts_roundtrip() {
        assert_eq!(
            enc(&Caesar, "Attack at Dawn!", &Options::new()).unwrap(),
            "Nggnpx ng Qnja!"
        );
        let three = Options::new().with("shift", 3);
        assert_eq!(enc(&Caesar, "Été xyz", &three).unwrap(), "Éwé abc");
        assert_eq!(dec(&Caesar, "Éwé abc", &three).unwrap(), "Été xyz");
    }

    #[test]
    fn shift_outside_the_slider_range_is_refused() {
        let error = enc(&Caesar, "a", &Options::new().with("shift", 26)).unwrap_err();
        assert_eq!(error.code, crate::ErrorCode::InvalidOption);
    }
}
