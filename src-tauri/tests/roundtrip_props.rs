//! Tests de propriété : décoder(encoder(x)) doit redonner x.
//!
//! Les propriétés marquées `#[ignore]` décrivent des bugs connus, corrigés en phase 1
//! de la roadmap (ROADMAP.md) : retirer le `#[ignore]` avec le correctif.
//! `cargo test -- --ignored` les exécute.

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
        Just((ConverterFormat::Reverse, ConvertOptions::default())),
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

/// Texte sans blanc au début ni à la fin, que `decode()` supprime aujourd'hui (B4).
fn text_without_edge_whitespace() -> impl Strategy<Value = String> {
    any::<String>().prop_map(|s| s.trim().to_string())
}

/// Mots de l'alphabet Morse en majuscules, séparés par une seule espace.
fn normalized_morse_text() -> impl Strategy<Value = String> {
    prop::collection::vec("[A-Z0-9.,?'!/()&:;=+_\"$@-]{1,8}", 0..6)
        .prop_map(|words| words.join(" "))
}

proptest! {
    #[test]
    fn byte_formats_roundtrip(text in any::<String>(), (format, opts) in byte_formats()) {
        assert_roundtrip(&text, format, &opts)?;
    }

    #[test]
    fn text_formats_roundtrip_without_edge_whitespace(
        text in text_without_edge_whitespace(),
        (format, opts) in text_formats(),
    ) {
        assert_roundtrip(&text, format, &opts)?;
    }

    /// Le Morse perd la casse et les blancs répétés : la propriété porte sur un texte normalisé.
    #[test]
    fn morse_roundtrip_on_normalized_text(text in normalized_morse_text()) {
        assert_roundtrip(&text, ConverterFormat::Morse, &ConvertOptions::default())?;
    }

    #[test]
    #[ignore = "B4 : decode() supprime les blancs de début et de fin (phase 1)"]
    fn text_formats_roundtrip(text in any::<String>(), (format, opts) in text_formats()) {
        assert_roundtrip(&text, format, &opts)?;
    }

    #[test]
    #[ignore = "B2 : le décodage Punycode transforme les labels ASCII (phase 1)"]
    fn punycode_roundtrip_on_domains(
        text in "[a-zàâäçéèêëîïôöùûü]{1,12}(\\.[a-zàâäçéèêëîïôöùûü]{1,12}){0,3}",
    ) {
        assert_roundtrip(&text, ConverterFormat::Punycode, &ConvertOptions::default())?;
    }
}
