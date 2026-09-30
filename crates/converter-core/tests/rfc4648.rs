//! Vecteurs de test officiels de la RFC 4648 (section 10) pour Base16, Base32 et Base64.

mod common;

use common::{decode, encode};
use converter_core::Options;

/// (texte clair, encodage attendu)
const BASE64: [(&str, &str); 7] = [
    ("", ""),
    ("f", "Zg=="),
    ("fo", "Zm8="),
    ("foo", "Zm9v"),
    ("foob", "Zm9vYg=="),
    ("fooba", "Zm9vYmE="),
    ("foobar", "Zm9vYmFy"),
];

const BASE32: [(&str, &str); 7] = [
    ("", ""),
    ("f", "MY======"),
    ("fo", "MZXQ===="),
    ("foo", "MZXW6==="),
    ("foob", "MZXW6YQ="),
    ("fooba", "MZXW6YTB"),
    ("foobar", "MZXW6YTBOI======"),
];

const BASE16: [(&str, &str); 7] = [
    ("", ""),
    ("f", "66"),
    ("fo", "666F"),
    ("foo", "666F6F"),
    ("foob", "666F6F62"),
    ("fooba", "666F6F6261"),
    ("foobar", "666F6F626172"),
];

fn assert_vectors(codec: &str, options: &Options, vectors: &[(&str, &str)]) {
    for &(plain, encoded) in vectors {
        assert_eq!(
            encode(codec, plain, options).as_deref(),
            Ok(encoded),
            "encodage {codec} de {plain:?}"
        );
        assert_eq!(
            decode(codec, encoded, options).as_deref(),
            Ok(plain),
            "décodage {codec} de {encoded:?}"
        );
    }
}

#[test]
fn base64_rfc4648_vectors() {
    assert_vectors("base64", &Options::new(), &BASE64);
}

#[test]
fn base32_rfc4648_vectors() {
    assert_vectors("base32", &Options::new(), &BASE32);
}

#[test]
fn base16_rfc4648_vectors() {
    // Le Base16 de la RFC correspond à l'hexadécimal continu en majuscules.
    let options = Options::new().with("separator", "").with("uppercase", true);
    assert_vectors("hex", &options, &BASE16);
}

#[test]
fn base16_decoding_accepts_lowercase() {
    assert_eq!(
        decode("hex", "666f6f626172", &Options::new()).as_deref(),
        Ok("foobar")
    );
}

#[test]
fn base64url_uses_url_safe_alphabet() {
    // RFC 4648 section 5 : « - » et « _ » remplacent « + » et « / ».
    let url_safe = Options::new().with("alphabet", "url_safe");
    assert_eq!(
        encode("base64", "<<???>>", &Options::new()).as_deref(),
        Ok("PDw/Pz8+Pg==")
    );
    assert_eq!(
        encode("base64", "<<???>>", &url_safe).as_deref(),
        Ok("PDw_Pz8-Pg==")
    );
    assert_eq!(
        decode("base64", "PDw_Pz8-Pg==", &url_safe).as_deref(),
        Ok("<<???>>")
    );
}
