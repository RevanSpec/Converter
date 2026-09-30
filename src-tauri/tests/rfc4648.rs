//! Vecteurs de test officiels de la RFC 4648 (section 10) pour Base16, Base32 et Base64.

use glass_converter_lib::converters::{decode, encode, ConvertOptions, ConverterFormat};

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

fn assert_vectors(format: ConverterFormat, opts: &ConvertOptions, vectors: &[(&str, &str)]) {
    for &(plain, encoded) in vectors {
        assert_eq!(
            encode(plain, format, opts).as_deref(),
            Ok(encoded),
            "encodage {format:?} de {plain:?}"
        );
        assert_eq!(
            decode(encoded, format, opts).as_deref(),
            Ok(plain),
            "décodage {format:?} de {encoded:?}"
        );
    }
}

#[test]
fn base64_rfc4648_vectors() {
    assert_vectors(ConverterFormat::Base64, &ConvertOptions::default(), &BASE64);
}

#[test]
fn base32_rfc4648_vectors() {
    assert_vectors(ConverterFormat::Base32, &ConvertOptions::default(), &BASE32);
}

#[test]
fn base16_rfc4648_vectors() {
    // Le Base16 de la RFC correspond à l'hexadécimal continu en majuscules.
    let opts = ConvertOptions {
        hex_separator: Some(String::new()),
        hex_uppercase: Some(true),
        ..Default::default()
    };
    assert_vectors(ConverterFormat::Hex, &opts, &BASE16);
}

#[test]
fn base16_decoding_accepts_lowercase() {
    let opts = ConvertOptions::default();
    assert_eq!(
        decode("666f6f626172", ConverterFormat::Hex, &opts).as_deref(),
        Ok("foobar")
    );
}

#[test]
fn base64url_uses_url_safe_alphabet() {
    // RFC 4648 section 5 : « - » et « _ » remplacent « + » et « / ».
    let standard = ConvertOptions::default();
    let url_safe = ConvertOptions {
        base64_url_safe: Some(true),
        ..Default::default()
    };
    assert_eq!(
        encode("<<???>>", ConverterFormat::Base64, &standard).as_deref(),
        Ok("PDw/Pz8+Pg==")
    );
    assert_eq!(
        encode("<<???>>", ConverterFormat::Base64, &url_safe).as_deref(),
        Ok("PDw_Pz8-Pg==")
    );
    assert_eq!(
        decode("PDw_Pz8-Pg==", ConverterFormat::Base64, &url_safe).as_deref(),
        Ok("<<???>>")
    );
}
