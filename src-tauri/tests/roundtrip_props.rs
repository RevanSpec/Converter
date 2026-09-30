//! Tests de propriété : décoder(encoder(x)) doit redonner x, pour chaque format.
//!
//! Deux formats ne sont réversibles que sur un texte bien formé : le Morse (qui perd
//! la casse et les blancs répétés) et l'Inversion (un accent combinant isolé en tête
//! de texte se rattache, une fois inversé, à la lettre qui le suit).

use glass_converter_lib::converters::{decode, encode, ConvertOptions, ConverterFormat};
use proptest::prelude::*;

/// Encode puis décode `text`, et vérifie que l'on retrouve le texte d'origine.
fn assert_roundtrip(
    text: &str,
    format: ConverterFormat,
    opts: &ConvertOptions,
) -> Result<(), TestCaseError> {
    let encoded = encode(text, format, opts).map_err(TestCaseError::fail)?;
    let decoded = decode(&encoded, format, opts).map_err(TestCaseError::fail)?;
    prop_assert_eq!(
        decoded.as_str(),
        text,
        "{:?} a encodé {:?}",
        format,
        encoded
    );
    Ok(())
}

/// Formats qui travaillent sur les octets UTF-8 du texte, avec chaque jeu d'options de l'interface.
fn byte_formats() -> impl Strategy<Value = (ConverterFormat, ConvertOptions)> {
    prop_oneof![
        (
            prop::sample::select(vec![" ", "", "0x", ":"]),
            any::<bool>()
        )
            .prop_map(|(separator, uppercase)| (
                ConverterFormat::Hex,
                ConvertOptions {
                    hex_separator: Some(separator.to_string()),
                    hex_uppercase: Some(uppercase),
                    ..Default::default()
                },
            )),
        any::<bool>().prop_map(|spaced| (
            ConverterFormat::Binary,
            ConvertOptions {
                binary_spaced: Some(spaced),
                ..Default::default()
            },
        )),
        any::<bool>().prop_map(|url_safe| (
            ConverterFormat::Base64,
            ConvertOptions {
                base64_url_safe: Some(url_safe),
                ..Default::default()
            },
        )),
        Just((ConverterFormat::Base32, ConvertOptions::default())),
        Just((ConverterFormat::AsciiDec, ConvertOptions::default())),
        Just((ConverterFormat::AsciiOct, ConvertOptions::default())),
        Just((ConverterFormat::Url, ConvertOptions::default())),
    ]
}

/// Formats qui transforment le texte lui-même ; César avec les décalages de l'interface.
fn text_formats() -> impl Strategy<Value = (ConverterFormat, ConvertOptions)> {
    prop_oneof![
        Just((ConverterFormat::Html, ConvertOptions::default())),
        (1..=25i32).prop_map(|shift| (
            ConverterFormat::Caesar,
            ConvertOptions {
                caesar_shift: Some(shift),
                ..Default::default()
            },
        )),
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
    #[test]
    fn byte_formats_roundtrip(text in any::<String>(), (format, opts) in byte_formats()) {
        assert_roundtrip(&text, format, &opts)?;
    }

    #[test]
    fn text_formats_roundtrip(text in any::<String>(), (format, opts) in text_formats()) {
        assert_roundtrip(&text, format, &opts)?;
    }

    #[test]
    fn reverse_roundtrip_on_well_formed_text(text in well_formed_text()) {
        assert_roundtrip(&text, ConverterFormat::Reverse, &ConvertOptions::default())?;
    }

    #[test]
    fn morse_roundtrip_on_normalized_text(text in normalized_morse_text()) {
        assert_roundtrip(&text, ConverterFormat::Morse, &ConvertOptions::default())?;
    }

    #[test]
    fn punycode_roundtrip_on_domains(
        text in "[a-zàâäçéèêëîïôöùûü]{1,12}(\\.[a-zàâäçéèêëîïôöùûü]{1,12}){0,3}",
    ) {
        assert_roundtrip(&text, ConverterFormat::Punycode, &ConvertOptions::default())?;
    }

    /// En mode brut (RFC 3492 sans préfixe), tout texte fait l'aller-retour.
    #[test]
    fn raw_punycode_roundtrip(text in any::<String>()) {
        let raw = ConvertOptions { punycode_prefix: Some(false), ..Default::default() };
        assert_roundtrip(&text, ConverterFormat::Punycode, &raw)?;
    }
}
