//! Un fichier par format. Pour en ajouter un : créer le fichier, puis l'ajouter à [`all`].

mod base32;
mod base64;
mod binary;
mod caesar;
mod decimal;
mod hex;
mod html;
mod morse;
mod octal;
mod punycode;
mod reverse;
mod url;

use crate::Codec;

/// Tous les formats, dans l'ordre des onglets.
pub(crate) fn all() -> Vec<Box<dyn Codec>> {
    vec![
        Box::new(self::hex::Hex),
        Box::new(self::binary::Binary),
        Box::new(self::base64::Base64),
        Box::new(self::base32::Base32),
        Box::new(self::morse::Morse),
        Box::new(self::url::Url),
        Box::new(self::caesar::Caesar),
        Box::new(self::html::Html),
        Box::new(self::decimal::Decimal),
        Box::new(self::octal::Octal),
        Box::new(self::reverse::Reverse),
        Box::new(self::punycode::Punycode),
    ]
}

/// Aides pour les tests des formats : entrée et sortie en texte.
#[cfg(test)]
pub(crate) mod testing {
    use crate::{Codec, CodecError, Options};

    pub fn enc(codec: &dyn Codec, text: &str, options: &Options) -> Result<String, CodecError> {
        codec
            .encode(text.as_bytes(), options)
            .map(|bytes| String::from_utf8(bytes).expect("sortie texte attendue"))
    }

    pub fn dec(codec: &dyn Codec, text: &str, options: &Options) -> Result<String, CodecError> {
        codec
            .decode(text.as_bytes(), options)
            .map(|bytes| String::from_utf8(bytes).expect("sortie texte attendue"))
    }
}
