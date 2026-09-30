//! Tests de propriété : décoder(encoder(x)) doit redonner x, pour chaque format.
//!
//! Deux formats ne sont réversibles que sur un texte bien formé : le Morse (qui perd
//! la casse et les blancs répétés) et l'Inversion (un accent combinant isolé en tête
//! de texte se rattache, une fois inversé, à la lettre qui le suit).

use converter_core::{registry, Options};
use proptest::prelude::*;

/// Encode puis décode `input`, et vérifie que l'on retrouve les octets d'origine.
fn assert_roundtrip(input: &[u8], codec: &str, options: &Options) -> Result<(), TestCaseError> {
    let codec = registry().get(codec).expect("format inconnu");
    let encoded = codec
        .encode(input, options)
        .map_err(|e| TestCaseError::fail(e.message))?;
    let decoded = codec
        .decode(&encoded, options)
        .map_err(|e| TestCaseError::fail(e.message))?;
    prop_assert_eq!(
        decoded.as_slice(),
        input,
        "{} a encodé {:?}",
        codec.meta().id,
        String::from_utf8_lossy(&encoded)
    );
    Ok(())
}

/// Formats qui représentent des octets quelconques, avec chaque jeu d'options de l'interface.
fn byte_formats() -> impl Strategy<Value = (&'static str, Options)> {
    prop_oneof![
        (
            prop::sample::select(vec![" ", "", "0x", ":"]),
            any::<bool>()
        )
            .prop_map(|(separator, uppercase)| (
                "hex",
                Options::new()
                    .with("separator", separator)
                    .with("uppercase", uppercase)
            )),
        prop::sample::select(vec!["grouped", "continuous"])
            .prop_map(|layout| ("binary", Options::new().with("layout", layout))),
        (
            prop::sample::select(vec!["standard", "url_safe"]),
            any::<bool>()
        )
            .prop_map(|(alphabet, padding)| (
                "base64",
                Options::new()
                    .with("alphabet", alphabet)
                    .with("padding", padding)
            )),
        Just(("base32", Options::new())),
        Just(("decimal", Options::new())),
        Just(("octal", Options::new())),
        (
            prop::sample::select(vec!["component", "uri"]),
            any::<bool>()
        )
            .prop_map(|(mode, plus_space)| (
                "url",
                Options::new()
                    .with("mode", mode)
                    .with("plus_space", plus_space)
            )),
    ]
}

/// Formats qui transforment le texte lui-même ; César avec les décalages de l'interface.
fn text_formats() -> impl Strategy<Value = (&'static str, Options)> {
    prop_oneof![
        Just(("html", Options::new())),
        (1..=25i32).prop_map(|shift| ("caesar", Options::new().with("shift", shift))),
    ]
}

/// Graphèmes complets qui restent distincts de leurs voisins dans n'importe quel ordre :
/// lettres simples ou accentuées (précomposées ou non), emojis composés, drapeaux,
/// idéogrammes, blancs et fin de ligne Windows.
const GRAPHEMES: &[&str] = &[
    "a",
    "Z",
    "7",
    " ",
    "!",
    "\t",
    "\n",
    "\r\n",
    "é",
    "ç",
    "e\u{301}",
    "a\u{300}\u{327}",
    "🦀",
    "👍🏽",
    "👨\u{200d}👩\u{200d}👧",
    "🇫🇷",
    "🇯🇵",
    "漢",
    "한",
    "ع",
];

fn well_formed_text() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::sample::select(GRAPHEMES), 0..24).prop_map(|parts| parts.concat())
}

/// Mots de l'alphabet Morse en majuscules, séparés par une espace, sur une ou plusieurs lignes.
fn normalized_morse_text() -> impl Strategy<Value = String> {
    let line = prop::collection::vec("[A-Z0-9É.,?'!/()&:;=+_\"$@-]{1,8}", 0..6)
        .prop_map(|words| words.join(" "));
    prop::collection::vec(line, 1..4).prop_map(|lines| lines.join("\n"))
}

proptest! {
    /// Le moteur travaille sur des octets : n'importe quelle suite d'octets fait l'aller-retour.
    #[test]
    fn byte_formats_roundtrip_any_bytes(
        bytes in prop::collection::vec(any::<u8>(), 0..64),
        (codec, options) in byte_formats(),
    ) {
        assert_roundtrip(&bytes, codec, &options)?;
    }

    #[test]
    fn byte_formats_roundtrip_any_text(text in any::<String>(), (codec, options) in byte_formats()) {
        assert_roundtrip(text.as_bytes(), codec, &options)?;
    }

    #[test]
    fn text_formats_roundtrip(text in any::<String>(), (codec, options) in text_formats()) {
        assert_roundtrip(text.as_bytes(), codec, &options)?;
    }

    #[test]
    fn reverse_roundtrip_on_well_formed_text(text in well_formed_text()) {
        assert_roundtrip(text.as_bytes(), "reverse", &Options::new())?;
    }

    #[test]
    fn morse_roundtrip_on_normalized_text(text in normalized_morse_text()) {
        assert_roundtrip(text.as_bytes(), "morse", &Options::new())?;
    }

    #[test]
    fn punycode_roundtrip_on_domains(
        text in "[a-zàâäçéèêëîïôöùûü]{1,12}(\\.[a-zàâäçéèêëîïôöùûü]{1,12}){0,3}",
    ) {
        assert_roundtrip(text.as_bytes(), "punycode", &Options::new())?;
    }

    /// En mode brut (RFC 3492 sans préfixe), tout texte fait l'aller-retour.
    #[test]
    fn raw_punycode_roundtrip(text in any::<String>()) {
        assert_roundtrip(text.as_bytes(), "punycode", &Options::new().with("mode", "raw"))?;
    }
}
