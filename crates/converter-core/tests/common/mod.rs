//! Aides communes aux tests d'intégration : entrée et sortie en texte.
#![allow(dead_code)] // chaque fichier de test n'utilise pas toutes les aides

use converter_core::{registry, CodecError, Options};

pub fn encode(codec: &str, text: &str, options: &Options) -> Result<String, CodecError> {
    let codec = registry().get(codec).expect("format inconnu");
    codec
        .encode(text.as_bytes(), options)
        .map(|bytes| String::from_utf8(bytes).expect("sortie texte attendue"))
}

pub fn decode(codec: &str, text: &str, options: &Options) -> Result<String, CodecError> {
    let codec = registry().get(codec).expect("format inconnu");
    codec
        .decode(text.as_bytes(), options)
        .map(|bytes| String::from_utf8(bytes).expect("sortie texte attendue"))
}
