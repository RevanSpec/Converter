//! Un test par bug corrigé ou comportement ajouté ; les codes (B2, D8…) sont ceux de ROADMAP.md.

mod common;

use common::{decode, encode};
use converter_core::{convert, registry, ConvertRequest, ErrorCode, Options, Output, Span};

fn none() -> Options {
    Options::new()
}

fn span(start: usize, end: usize) -> Option<Span> {
    Some(Span { start, end })
}

// B1 : le moteur travaille sur des octets ; un décodage qui ne donne pas de texte
// n'est plus une erreur.
#[test]
fn b1_bytes_that_are_not_text_are_returned_as_hex() {
    let request: ConvertRequest =
        serde_json::from_str(r#"{"codec":"base64","direction":"decode","input":"/9j/4A=="}"#)
            .unwrap();
    let response = convert(registry(), &request).unwrap();
    assert_eq!(
        response.output,
        Output::Bytes {
            hex: "ff d8 ff e0".to_string()
        }
    );
    assert_eq!(response.output_bytes, 4);
}

// B2 : le décodage Punycode transformait les labels ASCII (« example.com » → « Ωίθηδ.㯘 »).
#[test]
fn b2_punycode_decoding_keeps_labels_without_prefix() {
    assert_eq!(
        decode("punycode", "xn--caf-dma.com", &none()).as_deref(),
        Ok("café.com")
    );
    assert_eq!(
        decode("punycode", "example.com", &none()).as_deref(),
        Ok("example.com")
    );
    assert_eq!(
        decode(
            "punycode",
            "Voir xn--caf-dma.fr\net XN--MNCHEN-3YA.de",
            &none()
        )
        .as_deref(),
        Ok("Voir café.fr\net münchen.de")
    );
}

// D7 : l'encodage Punycode ne normalisait pas et mélangeait mots et labels.
#[test]
fn d7_punycode_encoding_follows_idna() {
    assert_eq!(
        encode("punycode", "café.fr et münchen.de", &none()).as_deref(),
        Ok("xn--caf-dma.fr et xn--mnchen-3ya.de")
    );
    assert_eq!(
        encode("punycode", "café-crème.fr", &none()).as_deref(),
        Ok("xn--caf-crme-60ag.fr")
    );
}

// B3 : le Base64 sans « = » final était refusé.
#[test]
fn b3_base64_without_padding() {
    let jwt_payload = "eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ";
    assert_eq!(
        decode("base64", jwt_payload, &none()).as_deref(),
        Ok(r#"{"sub":"1234567890","name":"John Doe","iat":1516239022}"#)
    );
    assert_eq!(
        decode("base64", "PDw/Pz8-Pg", &none()).unwrap_err().code,
        ErrorCode::MixedAlphabets
    );
}

// A7 : Base64 URL-safe sans « = », la forme des JWT.
#[test]
fn a7_base64_url_safe_without_padding() {
    let jwt = Options::new()
        .with("alphabet", "url_safe")
        .with("padding", false);
    // 14 octets : le Base64 standard se terminerait par « = ».
    assert_eq!(
        encode("base64", r#"{"alg":"none"}"#, &jwt).as_deref(),
        Ok("eyJhbGciOiJub25lIn0")
    );
    assert_eq!(
        encode("base64", r#"{"alg":"none"}"#, &none()).as_deref(),
        Ok("eyJhbGciOiJub25lIn0=")
    );
}

// B4 : decode() supprimait les blancs de début et de fin.
#[test]
fn b4_text_formats_keep_edge_whitespace() {
    assert_eq!(
        decode("caesar", " Uryyb\n", &none()).as_deref(),
        Ok(" Hello\n")
    );
    assert_eq!(decode("reverse", "\ncba", &none()).as_deref(), Ok("abc\n"));
    assert_eq!(
        decode("html", "  &lt;b&gt;  ", &none()).as_deref(),
        Ok("  <b>  ")
    );
}

// D4 : le Morse laissait passer les accents et perdait les retours à la ligne ; l'option
// arrive de l'interface sous la forme JSON ci-dessous.
#[test]
fn d4_morse_unknown_characters_option_from_the_interface() {
    let request: ConvertRequest = serde_json::from_str(
        r#"{"codec":"morse","direction":"encode","input":"à🦀b\nOK","options":{"unknown":"ignore"}}"#,
    )
    .unwrap();
    assert_eq!(
        convert(registry(), &request).unwrap().output,
        Output::Text {
            text: ".- -...\n--- -.-".to_string()
        }
    );
}

// D5 : l'inversion cassait les emojis composés et les accents combinants.
#[test]
fn d5_reverse_keeps_graphemes_intact() {
    assert_eq!(
        encode("reverse", "👨\u{200d}👩\u{200d}👧!", &none()).as_deref(),
        Ok("!👨\u{200d}👩\u{200d}👧")
    );
}

// D6 : seules 9 entités HTML nommées étaient reconnues.
#[test]
fn d6_all_html5_named_entities() {
    assert_eq!(
        decode("html", "&OElig;uvre &laquo;&nbsp;fin&nbsp;&raquo;", &none()).as_deref(),
        Ok("Œuvre «\u{a0}fin\u{a0}»")
    );
}

// D8 : séparateurs hexadécimaux refusés, positions décalées.
#[test]
fn d8_hex_accepts_common_separators() {
    for input in [
        "48-65-6c-6c-6f",
        "\\x48\\x65\\x6c\\x6c\\x6f",
        "%48%65%6C%6C%6F",
        "0x48,0x65,0x6c,0x6c,0x6f",
        "48:65;6c 6c\n6f",
    ] {
        assert_eq!(
            decode("hex", input, &none()).as_deref(),
            Ok("Hello"),
            "{input}"
        );
    }
}

// A2 : chaque erreur situable indique la plage fautive du texte saisi.
#[test]
fn a2_errors_carry_the_span_of_the_faulty_text() {
    let cases = [
        ("hex", "48 65 zz", span(6, 7)),
        ("binary", "01000001 0100000x", span(16, 17)),
        ("base64", "SG V$bG8=", span(4, 5)),
        ("base32", "MZ XW1===", span(5, 6)),
        ("morse", "... ------- ...", span(4, 11)),
        ("decimal", "72 256", span(3, 6)),
        ("url", "a%zz", span(1, 4)),
    ];
    for (codec, input, expected) in cases {
        let error = decode(codec, input, &none()).unwrap_err();
        assert_eq!(error.span, expected, "{codec} : {}", error.message);
    }
}

// A8 : URL en mode « URI complète » et « + » = espace.
#[test]
fn a8_url_modes() {
    let uri = Options::new().with("mode", "uri");
    assert_eq!(
        encode("url", "https://example.com/a b?x=1&y=2", &uri).as_deref(),
        Ok("https://example.com/a%20b?x=1&y=2")
    );
    let form = Options::new().with("plus_space", true);
    assert_eq!(decode("url", "a+b%2B", &form).as_deref(), Ok("a b+"));
}

// A9 : « ASCII décimal » et « ASCII octal » encodaient en réalité des octets UTF-8.
#[test]
fn a9_byte_formats_are_named_after_bytes() {
    let label = |id| registry().get(id).map(|codec| codec.meta().label);
    assert_eq!(label("asciidec"), Some("Octets (décimal)"));
    assert_eq!(label("asciioct"), Some("Octets (octal)"));
}

#[test]
fn invalid_option_values_are_refused() {
    let error = encode("hex", "a", &Options::new().with("separator", "|")).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidOption);
}
