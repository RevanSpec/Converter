//! Un test par bug corrigé en phase 1 ; les codes (B2, D8…) sont ceux de ROADMAP.md.

use glass_converter_lib::converters::{
    decode, encode, ConvertOptions, ConverterFormat, MorseUnknown,
};

fn enc(text: &str, format: ConverterFormat, opts: &ConvertOptions) -> Result<String, String> {
    encode(text, format, opts)
}

fn dec(text: &str, format: ConverterFormat, opts: &ConvertOptions) -> Result<String, String> {
    decode(text, format, opts)
}

fn default() -> ConvertOptions {
    ConvertOptions::default()
}

fn error_of(result: Result<String, String>) -> String {
    result.expect_err("une erreur était attendue")
}

// B2 : le décodage Punycode transformait les labels ASCII (« example.com » → « Ωίθηδ.㯘 »).
#[test]
fn b2_punycode_decoding_keeps_labels_without_prefix() {
    let p = ConverterFormat::Punycode;
    assert_eq!(
        dec("xn--caf-dma.com", p, &default()).as_deref(),
        Ok("café.com")
    );
    assert_eq!(
        dec("example.com", p, &default()).as_deref(),
        Ok("example.com")
    );
    assert_eq!(dec("hello", p, &default()).as_deref(), Ok("hello"));
    assert_eq!(
        dec("Voir xn--caf-dma.fr\net XN--MNCHEN-3YA.de", p, &default()).as_deref(),
        Ok("Voir café.fr\net münchen.de")
    );
}

#[test]
fn b2_invalid_punycode_label_is_an_error() {
    let message = error_of(dec("xn--caf-dm!.fr", ConverterFormat::Punycode, &default()));
    assert!(message.contains("xn--caf-dm!"), "{message}");
}

// D7 : l'encodage Punycode ne normalisait pas (« Café.fr » → « xn--Caf-dma.fr ») et
// mélangeait mots et labels (« café.fr et münchen.de »).
#[test]
fn d7_punycode_encoding_follows_idna() {
    let p = ConverterFormat::Punycode;
    assert_eq!(
        enc("Café.fr", p, &default()).as_deref(),
        Ok("xn--caf-dma.fr")
    );
    assert_eq!(
        enc("café.fr et münchen.de", p, &default()).as_deref(),
        Ok("xn--caf-dma.fr et xn--mnchen-3ya.de")
    );
    assert_eq!(
        enc("café-crème.fr", p, &default()).as_deref(),
        Ok("xn--caf-crme-60ag.fr")
    );
}

#[test]
fn d7_raw_punycode_encodes_every_word() {
    let raw = ConvertOptions {
        punycode_prefix: Some(false),
        ..default()
    };
    let p = ConverterFormat::Punycode;
    assert_eq!(enc("münchen", p, &raw).as_deref(), Ok("mnchen-3ya"));
    assert_eq!(enc("abc", p, &raw).as_deref(), Ok("abc-"));
    assert_eq!(
        dec("mnchen-3ya abc-", p, &raw).as_deref(),
        Ok("münchen abc")
    );
}

// B3 : le Base64 sans « = » final était refusé.
#[test]
fn b3_base64_without_padding() {
    let b64 = ConverterFormat::Base64;
    assert_eq!(dec("SGVsbG8", b64, &default()).as_deref(), Ok("Hello"));
    assert_eq!(dec("SGVsbG8=", b64, &default()).as_deref(), Ok("Hello"));
    assert_eq!(dec("PDw_Pz8-Pg", b64, &default()).as_deref(), Ok("<<???>>"));
    let jwt_payload = "eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ";
    assert_eq!(
        dec(jwt_payload, b64, &default()).as_deref(),
        Ok(r#"{"sub":"1234567890","name":"John Doe","iat":1516239022}"#)
    );
}

#[test]
fn b3_base64_mixing_both_alphabets_is_an_error() {
    let message = error_of(dec("PDw/Pz8-Pg", ConverterFormat::Base64, &default()));
    assert!(message.contains("mélange"), "{message}");
}

// B4 : decode() supprimait les blancs de début et de fin.
#[test]
fn b4_text_formats_keep_edge_whitespace() {
    assert_eq!(
        dec(" Uryyb\n", ConverterFormat::Caesar, &default()).as_deref(),
        Ok(" Hello\n")
    );
    assert_eq!(
        dec("\ncba", ConverterFormat::Reverse, &default()).as_deref(),
        Ok("abc\n")
    );
    assert_eq!(
        dec("  &lt;b&gt;  ", ConverterFormat::Html, &default()).as_deref(),
        Ok("  <b>  ")
    );
}

// D4 : le Morse laissait passer les accents (indécodables) et perdait les retours à la ligne.
#[test]
fn d4_morse_encodes_e_acute_and_keeps_lines() {
    let m = ConverterFormat::Morse;
    assert_eq!(
        enc("Élan", m, &default()).as_deref(),
        Ok("..-.. .-.. .- -.")
    );
    assert_eq!(
        dec("..-.. .-.. .- -.", m, &default()).as_deref(),
        Ok("ÉLAN")
    );
    assert_eq!(
        enc("SOS\nOK", m, &default()).as_deref(),
        Ok("... --- ...\n--- -.-")
    );
    assert_eq!(
        dec("... --- ...\n--- -.-", m, &default()).as_deref(),
        Ok("SOS\nOK")
    );
}

#[test]
fn d4_morse_unknown_characters_follow_the_option() {
    let m = ConverterFormat::Morse;
    let with = |unknown| ConvertOptions {
        morse_unknown: Some(unknown),
        ..default()
    };

    // Translittérer (par défaut) : à → A, mais l'emoji reste une erreur.
    assert_eq!(
        enc("ça va", m, &default()).as_deref(),
        Ok("-.-. .- / ...- .-")
    );
    assert!(error_of(enc("a🦀", m, &default())).contains("Ignorer"));

    // Erreur : même les lettres translittérables sont refusées, avec leur position.
    let message = error_of(enc("ça", m, &with(MorseUnknown::Error)));
    assert!(message.contains("position 1"), "{message}");

    // Ignorer : translittère ce qui peut l'être, écarte le reste.
    assert_eq!(
        enc("à🦀b", m, &with(MorseUnknown::Ignore)).as_deref(),
        Ok(".- -...")
    );
}

#[test]
fn d4_morse_option_uses_the_names_sent_by_the_interface() {
    let options: ConvertOptions = serde_json::from_str(r#"{"morse_unknown":"ignore"}"#).unwrap();
    assert_eq!(options.morse_unknown, Some(MorseUnknown::Ignore));
}

// D5 : l'inversion cassait les emojis composés et les accents combinants.
#[test]
fn d5_reverse_keeps_graphemes_intact() {
    let r = ConverterFormat::Reverse;
    assert_eq!(enc("👍🏽", r, &default()).as_deref(), Ok("👍🏽"));
    assert_eq!(
        enc("e\u{301}t\u{e9}", r, &default()).as_deref(),
        Ok("\u{e9}te\u{301}")
    );
    assert_eq!(
        enc("👨\u{200d}👩\u{200d}👧!", r, &default()).as_deref(),
        Ok("!👨\u{200d}👩\u{200d}👧")
    );
}

// D6 : seules 9 entités HTML nommées étaient reconnues.
#[test]
fn d6_all_html5_named_entities() {
    assert_eq!(
        dec(
            "&eacute;t&eacute; &hellip; &mdash; &OElig;",
            ConverterFormat::Html,
            &default()
        )
        .as_deref(),
        Ok("été … — Œ")
    );
}

// D8 : Base32 tronqué accepté, séparateurs hexadécimaux refusés, positions décalées.
#[test]
fn d8_base32_rejects_impossible_lengths_and_bad_padding() {
    let b32 = ConverterFormat::Base32;
    assert!(dec("A", b32, &default()).is_err());
    assert!(dec("MZXW6==", b32, &default()).is_err());
    assert!(dec("MZ=XW6", b32, &default()).is_err());
    assert_eq!(dec("MZXW6", b32, &default()).as_deref(), Ok("foo"));
    assert_eq!(dec("mzxw6===", b32, &default()).as_deref(), Ok("foo"));
}

#[test]
fn d8_hex_accepts_common_separators() {
    let h = ConverterFormat::Hex;
    for input in [
        "48-65-6c-6c-6f",
        "\\x48\\x65\\x6c\\x6c\\x6f",
        "%48%65%6C%6C%6F",
        "0x48,0x65,0x6c,0x6c,0x6f",
        "48:65;6c 6c\n6f",
    ] {
        assert_eq!(dec(input, h, &default()).as_deref(), Ok("Hello"), "{input}");
    }
}

#[test]
fn d8_error_positions_refer_to_the_typed_text() {
    let hex = error_of(dec("48 65 zz", ConverterFormat::Hex, &default()));
    assert!(hex.contains("position 7"), "{hex}");

    let binary = error_of(dec(
        "01000001 0100000x",
        ConverterFormat::Binary,
        &default(),
    ));
    assert!(binary.contains("position 17"), "{binary}");

    let base64 = error_of(dec("SG V$bG8=", ConverterFormat::Base64, &default()));
    assert!(base64.contains("position 5"), "{base64}");

    let base32 = error_of(dec("MZ XW1===", ConverterFormat::Base32, &default()));
    assert!(base32.contains("position 6"), "{base32}");
}
