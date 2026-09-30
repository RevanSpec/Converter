use base64::alphabet;
use base64::engine::{general_purpose, DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use base64::{DecodeError, Engine as _};
use idna::punycode;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConverterFormat {
    Hex,
    Binary,
    Base64,
    Base32,
    Morse,
    AsciiDec,
    AsciiOct,
    Url,
    Html,
    Caesar,
    Reverse,
    Punycode,
}

impl FromStr for ConverterFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hex" | "hexadecimal" => Ok(Self::Hex),
            "binary" | "binaire" | "bin" => Ok(Self::Binary),
            "base64" | "b64" => Ok(Self::Base64),
            "base32" | "b32" => Ok(Self::Base32),
            "morse" => Ok(Self::Morse),
            "asciidec" | "ascii_dec" | "decimal" | "dec" => Ok(Self::AsciiDec),
            "asciioct" | "ascii_oct" | "octal" | "oct" => Ok(Self::AsciiOct),
            "url" | "urlencode" | "percent" => Ok(Self::Url),
            "html" | "html_entities" => Ok(Self::Html),
            "caesar" | "rot13" | "cesar" => Ok(Self::Caesar),
            "reverse" | "inverse" => Ok(Self::Reverse),
            "punycode" | "puny" | "idn" => Ok(Self::Punycode),
            unknown => Err(format!("Format non reconnu: '{}'", unknown)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConvertOptions {
    pub hex_separator: Option<String>,
    pub hex_uppercase: Option<bool>,
    pub binary_spaced: Option<bool>,
    pub base64_url_safe: Option<bool>,
    pub caesar_shift: Option<i32>,
    pub punycode_prefix: Option<bool>,
    pub morse_unknown: Option<MorseUnknown>,
}

/// Traitement, à l'encodage Morse, des caractères qui n'ont pas de code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MorseUnknown {
    /// Erreur au premier caractère sans code.
    Error,
    /// Lettres accentuées ramenées à leur lettre de base (à → A) ; erreur pour le reste.
    #[default]
    Transliterate,
    /// Translittération si possible, sinon le caractère est écarté.
    Ignore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertResult {
    pub output: String,
    pub input_chars: usize,
    pub input_bytes: usize,
    pub output_chars: usize,
    pub output_bytes: usize,
}

pub fn convert(
    input: &str,
    format: ConverterFormat,
    to_encoded: bool,
    options: &ConvertOptions,
) -> Result<ConvertResult, String> {
    let output = if to_encoded {
        encode(input, format, options)?
    } else {
        decode(input, format, options)?
    };

    Ok(ConvertResult {
        input_chars: input.chars().count(),
        input_bytes: input.len(),
        output_chars: output.chars().count(),
        output_bytes: output.len(),
        output,
    })
}

// ==========================================
// ENCODE
// ==========================================
pub fn encode(
    input: &str,
    format: ConverterFormat,
    options: &ConvertOptions,
) -> Result<String, String> {
    if input.is_empty() {
        return Ok(String::new());
    }

    match format {
        ConverterFormat::Hex => {
            let sep = options.hex_separator.as_deref().unwrap_or(" ");
            let uppercase = options.hex_uppercase.unwrap_or(false);
            let bytes = input.as_bytes();

            if sep == "0x" {
                let parts: Vec<String> = bytes
                    .iter()
                    .map(|b| {
                        if uppercase {
                            format!("0x{:02X}", b)
                        } else {
                            format!("0x{:02x}", b)
                        }
                    })
                    .collect();
                Ok(parts.join(" "))
            } else if sep.is_empty() {
                let s = hex::encode(bytes);
                if uppercase {
                    Ok(s.to_uppercase())
                } else {
                    Ok(s)
                }
            } else {
                let parts: Vec<String> = bytes
                    .iter()
                    .map(|b| {
                        if uppercase {
                            format!("{:02X}", b)
                        } else {
                            format!("{:02x}", b)
                        }
                    })
                    .collect();
                Ok(parts.join(sep))
            }
        }
        ConverterFormat::Binary => {
            let spaced = options.binary_spaced.unwrap_or(true);
            let bytes = input.as_bytes();
            let parts: Vec<String> = bytes.iter().map(|b| format!("{:08b}", b)).collect();
            if spaced {
                Ok(parts.join(" "))
            } else {
                Ok(parts.join(""))
            }
        }
        ConverterFormat::Base64 => {
            let url_safe = options.base64_url_safe.unwrap_or(false);
            if url_safe {
                Ok(general_purpose::URL_SAFE.encode(input.as_bytes()))
            } else {
                Ok(general_purpose::STANDARD.encode(input.as_bytes()))
            }
        }
        ConverterFormat::Base32 => Ok(base32_encode(input.as_bytes())),
        ConverterFormat::Morse => text_to_morse(input, options.morse_unknown.unwrap_or_default()),
        ConverterFormat::AsciiDec => {
            let parts: Vec<String> = input.as_bytes().iter().map(|b| b.to_string()).collect();
            Ok(parts.join(" "))
        }
        ConverterFormat::AsciiOct => {
            let parts: Vec<String> = input
                .as_bytes()
                .iter()
                .map(|b| format!("{:03o}", b))
                .collect();
            Ok(parts.join(" "))
        }
        ConverterFormat::Url => Ok(url_encode(input)),
        ConverterFormat::Html => Ok(html_escape(input)),
        ConverterFormat::Caesar => {
            let shift = options.caesar_shift.unwrap_or(13);
            Ok(caesar_shift(input, shift))
        }
        ConverterFormat::Reverse => Ok(reverse_graphemes(input)),
        ConverterFormat::Punycode => {
            punycode_encode(input, options.punycode_prefix.unwrap_or(true))
        }
    }
}

// ==========================================
// DECODE
// ==========================================
pub fn decode(
    input: &str,
    format: ConverterFormat,
    options: &ConvertOptions,
) -> Result<String, String> {
    if input.is_empty() {
        return Ok(String::new());
    }

    // Pas de trim global : les formats textuels gardent l'entrée intacte,
    // les autres ignorent eux-mêmes les blancs.
    match format {
        ConverterFormat::Hex => utf8_text(hex_decode(input)?),
        ConverterFormat::Binary => utf8_text(binary_decode(input)?),
        ConverterFormat::Base64 => utf8_text(base64_decode(
            input,
            options.base64_url_safe.unwrap_or(false),
        )?),
        ConverterFormat::Base32 => utf8_text(base32_decode(input)?),
        ConverterFormat::Morse => morse_to_text(input),
        ConverterFormat::AsciiDec => {
            let tokens = input.split(|c: char| c.is_whitespace() || c == ',' || c == ';');
            let mut bytes = Vec::new();
            for (i, token) in tokens.filter(|t| !t.is_empty()).enumerate() {
                let val: u8 = token.parse().map_err(|_| {
                    format!(
                        "Valeur décimale invalide '{}' à l'élément {} (doit être entre 0 et 255).",
                        token,
                        i + 1
                    )
                })?;
                bytes.push(val);
            }
            utf8_text(bytes)
        }
        ConverterFormat::AsciiOct => {
            let tokens = input.split(|c: char| c.is_whitespace() || c == ',' || c == ';');
            let mut bytes = Vec::new();
            for (i, token) in tokens.filter(|t| !t.is_empty()).enumerate() {
                let val = u8::from_str_radix(token, 8).map_err(|_| {
                    format!(
                        "Valeur octale invalide '{}' à l'élément {} (doit être en base 8, 0-377).",
                        token,
                        i + 1
                    )
                })?;
                bytes.push(val);
            }
            utf8_text(bytes)
        }
        ConverterFormat::Url => url_decode(input),
        ConverterFormat::Html => Ok(htmlize::unescape(input).into_owned()),
        ConverterFormat::Caesar => {
            let shift = options.caesar_shift.unwrap_or(13);
            Ok(caesar_shift(input, -(shift.rem_euclid(26))))
        }
        ConverterFormat::Reverse => Ok(reverse_graphemes(input)),
        ConverterFormat::Punycode => {
            punycode_decode(input, options.punycode_prefix.unwrap_or(true))
        }
    }
}

/// Convertit les octets décodés en texte, ou explique pourquoi ce n'est pas de l'UTF-8.
fn utf8_text(bytes: Vec<u8>) -> Result<String, String> {
    String::from_utf8(bytes).map_err(|e| {
        format!(
            "Les octets décodés ne constituent pas un texte UTF-8 valide (erreur à l'octet {}).",
            e.utf8_error().valid_up_to()
        )
    })
}

/// Retire les blancs de `input` et garde, pour chaque caractère conservé,
/// sa position dans le texte saisi (le premier caractère est en position 1).
fn strip_whitespace(input: &str) -> (String, Vec<usize>) {
    let mut cleaned = String::with_capacity(input.len());
    let mut positions = Vec::with_capacity(input.len());
    for (index, c) in input.chars().enumerate() {
        if !c.is_whitespace() {
            cleaned.push(c);
            positions.push(index + 1);
        }
    }
    (cleaned, positions)
}

// ==========================================
// HEXADÉCIMAL
// ==========================================
/// Accepte les préfixes `0x`, `\x` et `%`, et les séparateurs blancs, `:`, `,`, `-` et `;`.
fn hex_decode(input: &str) -> Result<Vec<u8>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut digits = Vec::with_capacity(chars.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if matches!(c, '0' | '\\') && matches!(chars.get(i + 1).copied(), Some('x' | 'X')) {
            i += 2;
            continue;
        }
        match c.to_digit(16) {
            Some(digit) => digits.push(digit as u8),
            None if c.is_whitespace() || matches!(c, '%' | ':' | ',' | '-' | ';') => {}
            None => {
                return Err(format!(
                    "Caractère hexadécimal invalide « {c} » en position {}.",
                    i + 1
                ))
            }
        }
        i += 1;
    }

    if !digits.len().is_multiple_of(2) {
        return Err(format!(
            "Nombre impair de chiffres hexadécimaux ({}) : chaque octet en demande 2.",
            digits.len()
        ));
    }

    Ok(digits
        .chunks(2)
        .map(|pair| (pair[0] << 4) | pair[1])
        .collect())
}

// ==========================================
// BINAIRE
// ==========================================
/// Accepte les séparateurs blancs, `,` et `-` entre les bits.
fn binary_decode(input: &str) -> Result<Vec<u8>, String> {
    let mut bits = Vec::with_capacity(input.len());
    for (index, c) in input.chars().enumerate() {
        match c {
            '0' => bits.push(0u8),
            '1' => bits.push(1u8),
            c if c.is_whitespace() || c == ',' || c == '-' => {}
            _ => {
                return Err(format!(
                "Caractère binaire invalide « {c} » en position {} : seuls 0 et 1 sont autorisés.",
                index + 1
            ))
            }
        }
    }

    if !bits.len().is_multiple_of(8) {
        return Err(format!(
            "Nombre de bits invalide ({}) : il faut un multiple de 8.",
            bits.len()
        ));
    }

    Ok(bits
        .chunks(8)
        .map(|byte| byte.iter().fold(0u8, |acc, &bit| (acc << 1) | bit))
        .collect())
}

// ==========================================
// BASE64 (RFC 4648)
// ==========================================
/// Décodeurs tolérants : le « = » final est facultatif (JWT, paramètres d'URL…).
const BASE64_STANDARD: GeneralPurpose = GeneralPurpose::new(
    &alphabet::STANDARD,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);
const BASE64_URL_SAFE: GeneralPurpose = GeneralPurpose::new(
    &alphabet::URL_SAFE,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

/// Décode du Base64, avec ou sans padding ; l'alphabet est déduit des caractères présents.
fn base64_decode(input: &str, prefer_url_safe: bool) -> Result<Vec<u8>, String> {
    let (cleaned, positions) = strip_whitespace(input);
    let url_safe = cleaned.contains(['-', '_']);
    let standard = cleaned.contains(['+', '/']);
    if url_safe && standard {
        return Err(
            "Base64 invalide : le texte mélange l'alphabet standard (+ /) et l'alphabet URL-safe (- _)."
                .to_string(),
        );
    }

    let engine = if url_safe || (prefer_url_safe && !standard) {
        &BASE64_URL_SAFE
    } else {
        &BASE64_STANDARD
    };

    // Caractère à l'octet `offset` du texte nettoyé, et sa position dans le texte saisi.
    let locate = |offset: usize| {
        let index = cleaned.get(..offset).map_or(0, |s| s.chars().count());
        let c = cleaned
            .get(offset..)
            .and_then(|s| s.chars().next())
            .unwrap_or('?');
        (c, positions.get(index).copied().unwrap_or(index + 1))
    };

    engine.decode(&cleaned).map_err(|e| match e {
        DecodeError::InvalidByte(offset, _) => {
            let (c, position) = locate(offset);
            format!("Caractère Base64 invalide « {c} » en position {position}.")
        }
        DecodeError::InvalidLastSymbol(offset, _) => {
            let (c, position) = locate(offset);
            format!(
                "Dernier caractère Base64 « {c} » (position {position}) incohérent : le texte est peut-être tronqué."
            )
        }
        DecodeError::InvalidLength(length) => format!(
            "Longueur Base64 impossible ({length} caractères utiles) : le texte est peut-être tronqué."
        ),
        DecodeError::InvalidPadding => {
            "Padding Base64 invalide : les « = » doivent terminer le texte.".to_string()
        }
    })
}

// ==========================================
// BASE32 (RFC 4648)
// ==========================================
const BASE32_ALPHABET: &[u8; 32] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn base32_encode(data: &[u8]) -> String {
    let mut result = String::new();
    let mut buffer: u64 = 0;
    let mut bits_left = 0;

    for &byte in data {
        buffer = (buffer << 8) | (byte as u64);
        bits_left += 8;
        while bits_left >= 5 {
            bits_left -= 5;
            let index = ((buffer >> bits_left) & 0x1F) as usize;
            result.push(BASE32_ALPHABET[index] as char);
        }
    }

    if bits_left > 0 {
        let index = ((buffer << (5 - bits_left)) & 0x1F) as usize;
        result.push(BASE32_ALPHABET[index] as char);
    }

    // Padding RFC 4648
    while !result.len().is_multiple_of(8) {
        result.push('=');
    }

    result
}

/// Décode du Base32 en majuscules ou minuscules, avec ou sans padding.
fn base32_decode(input: &str) -> Result<Vec<u8>, String> {
    let (cleaned, positions) = strip_whitespace(input);
    let chars: Vec<char> = cleaned.chars().collect();
    let data_len = chars.iter().position(|&c| c == '=').unwrap_or(chars.len());
    let (data, padding) = chars.split_at(data_len);

    if let Some(offset) = padding.iter().position(|&c| c != '=') {
        let index = data_len + offset;
        return Err(format!(
            "Caractère « {} » après le padding Base32, en position {}.",
            chars[index], positions[index]
        ));
    }

    let mut buffer: u64 = 0;
    let mut bits = 0;
    let mut bytes = Vec::with_capacity(data.len() * 5 / 8);
    for (index, &c) in data.iter().enumerate() {
        let value = match c.to_ascii_uppercase() {
            letter @ 'A'..='Z' => letter as u64 - 'A' as u64,
            digit @ '2'..='7' => digit as u64 - '2' as u64 + 26,
            _ => {
                return Err(format!(
                    "Caractère Base32 invalide « {c} » en position {}.",
                    positions[index]
                ))
            }
        };
        buffer = (buffer << 5) | value;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            bytes.push((buffer >> bits) as u8);
        }
    }

    // Seules ces longueurs utiles (modulo 8) donnent un nombre entier d'octets.
    let expected_padding = match data.len() % 8 {
        0 => 0,
        2 => 6,
        4 => 4,
        5 => 3,
        7 => 1,
        _ => {
            return Err(format!(
            "Longueur Base32 impossible ({} caractères utiles) : le texte est peut-être tronqué.",
            data.len()
        ))
        }
    };
    if !padding.is_empty() && padding.len() != expected_padding {
        return Err(format!(
            "Padding Base32 incohérent : {} « = » pour {} caractères utiles ({expected_padding} attendus).",
            padding.len(),
            data.len()
        ));
    }

    Ok(bytes)
}

// ==========================================
// MORSE CODE (ITU-R M.1677-1)
// ==========================================
fn morse_char(c: char) -> Option<&'static str> {
    match c.to_ascii_uppercase() {
        'A' => Some(".-"),
        'B' => Some("-..."),
        'C' => Some("-.-."),
        'D' => Some("-.."),
        'E' => Some("."),
        'F' => Some("..-."),
        'G' => Some("--."),
        'H' => Some("...."),
        'I' => Some(".."),
        'J' => Some(".---"),
        'K' => Some("-.-"),
        'L' => Some(".-.."),
        'M' => Some("--"),
        'N' => Some("-."),
        'O' => Some("---"),
        'P' => Some(".--."),
        'Q' => Some("--.-"),
        'R' => Some(".-."),
        'S' => Some("..."),
        'T' => Some("-"),
        'U' => Some("..-"),
        'V' => Some("...-"),
        'W' => Some(".--"),
        'X' => Some("-..-"),
        'Y' => Some("-.--"),
        'Z' => Some("--.."),
        '0' => Some("-----"),
        '1' => Some(".----"),
        '2' => Some("..---"),
        '3' => Some("...--"),
        '4' => Some("....-"),
        '5' => Some("....."),
        '6' => Some("-...."),
        '7' => Some("--..."),
        '8' => Some("---.."),
        '9' => Some("----."),
        '.' => Some(".-.-.-"),
        ',' => Some("--..--"),
        '?' => Some("..--.."),
        '\'' => Some(".----."),
        '!' => Some("-.-.--"),
        '/' => Some("-..-."),
        '(' => Some("-.--."),
        ')' => Some("-.--.-"),
        '&' => Some(".-..."),
        ':' => Some("---..."),
        ';' => Some("-.-.-."),
        '=' => Some("-...-"),
        '+' => Some(".-.-."),
        '-' => Some("-....-"),
        '_' => Some("..--.-"),
        '"' => Some(".-..-."),
        '$' => Some("...-..-"),
        '@' => Some(".--.-."),
        // Seule lettre accentuée de la recommandation ITU.
        'É' | 'é' => Some("..-.."),
        _ => None,
    }
}

fn morse_to_char(m: &str) -> Option<char> {
    match m {
        ".-" => Some('A'),
        "-..." => Some('B'),
        "-.-." => Some('C'),
        "-.." => Some('D'),
        "." => Some('E'),
        "..-." => Some('F'),
        "--." => Some('G'),
        "...." => Some('H'),
        ".." => Some('I'),
        ".---" => Some('J'),
        "-.-" => Some('K'),
        ".-.." => Some('L'),
        "--" => Some('M'),
        "-." => Some('N'),
        "---" => Some('O'),
        ".--." => Some('P'),
        "--.-" => Some('Q'),
        ".-." => Some('R'),
        "..." => Some('S'),
        "-" => Some('T'),
        "..-" => Some('U'),
        "...-" => Some('V'),
        ".--" => Some('W'),
        "-..-" => Some('X'),
        "-.--" => Some('Y'),
        "--.." => Some('Z'),
        "-----" => Some('0'),
        ".----" => Some('1'),
        "..---" => Some('2'),
        "...--" => Some('3'),
        "....-" => Some('4'),
        "....." => Some('5'),
        "-...." => Some('6'),
        "--..." => Some('7'),
        "---.." => Some('8'),
        "----." => Some('9'),
        ".-.-.-" => Some('.'),
        "--..--" => Some(','),
        "..--.." => Some('?'),
        ".----." => Some('\''),
        "-.-.--" => Some('!'),
        "-..-." => Some('/'),
        "-.--." => Some('('),
        "-.--.-" => Some(')'),
        ".-..." => Some('&'),
        "---..." => Some(':'),
        "-.-.-." => Some(';'),
        "-...-" => Some('='),
        ".-.-." => Some('+'),
        "-....-" => Some('-'),
        "..--.-" => Some('_'),
        ".-..-." => Some('"'),
        "...-..-" => Some('$'),
        ".--.-." => Some('@'),
        "..-.." => Some('É'),
        _ => None,
    }
}

/// Lettre de base d'un caractère accentué (à → A) ou équivalent ASCII d'une ponctuation
/// typographique (’ → '), pour les caractères que le Morse ne connaît pas.
fn transliterate(c: char) -> Option<&'static str> {
    Some(match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' => "A",
        'ç' | 'Ç' => "C",
        'è' | 'ê' | 'ë' | 'È' | 'Ê' | 'Ë' => "E",
        'ì' | 'í' | 'î' | 'ï' | 'Ì' | 'Í' | 'Î' | 'Ï' => "I",
        'ñ' | 'Ñ' => "N",
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' => "O",
        'ù' | 'ú' | 'û' | 'ü' | 'Ù' | 'Ú' | 'Û' | 'Ü' => "U",
        'ý' | 'ÿ' | 'Ý' | 'Ÿ' => "Y",
        'æ' | 'Æ' => "AE",
        'œ' | 'Œ' => "OE",
        'ß' => "SS",
        '’' | '‘' => "'",
        '«' | '»' | '“' | '”' => "\"",
        _ => return None,
    })
}

/// Lettres séparées par une espace, mots par « / », lignes par un retour à la ligne.
fn text_to_morse(text: &str, unknown: MorseUnknown) -> Result<String, String> {
    // Chaque caractère est gardé, translittéré, écarté ou refusé selon `unknown`.
    let mut normalized = String::with_capacity(text.len());
    for (index, c) in text.chars().enumerate() {
        if c.is_whitespace() || morse_char(c).is_some() {
            normalized.push(c);
            continue;
        }
        let base = transliterate(c);
        match (unknown, base) {
            (MorseUnknown::Transliterate | MorseUnknown::Ignore, Some(base)) => {
                normalized.push_str(base)
            }
            (MorseUnknown::Ignore, None) => {}
            (MorseUnknown::Error, Some(base)) => {
                return Err(format!(
                    "« {c} » (position {}) n'existe pas en Morse : choisissez « Translittérer » pour le remplacer par « {base} ».",
                    index + 1
                ))
            }
            (_, None) => {
                return Err(format!(
                    "« {c} » (position {}) n'existe pas en Morse : choisissez « Ignorer » pour l'écarter.",
                    index + 1
                ))
            }
        }
    }

    let lines: Vec<String> = normalized
        .split('\n')
        .map(|line| {
            line.split_whitespace()
                .map(|word| {
                    word.chars()
                        .filter_map(morse_char)
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect::<Vec<_>>()
                .join(" / ")
        })
        .collect();
    Ok(lines.join("\n"))
}

fn morse_to_text(morse: &str) -> Result<String, String> {
    let mut lines = Vec::new();
    for line in morse.split('\n') {
        let mut words = Vec::new();
        for word in line.split('/') {
            let mut text = String::new();
            for code in word.split_whitespace() {
                let c = morse_to_char(code)
                    .ok_or_else(|| format!("Code Morse invalide ou non reconnu : « {code} »."))?;
                text.push(c);
            }
            words.push(text);
        }
        lines.push(words.join(" "));
    }
    Ok(lines.join("\n"))
}

// ==========================================
// CAESAR / ROT13
// ==========================================
fn caesar_shift(text: &str, shift: i32) -> String {
    let s = shift.rem_euclid(26);
    text.chars()
        .map(|c| {
            if c.is_ascii_lowercase() {
                let base = b'a' as i32;
                let offset = (c as i32 - base + s) % 26;
                (base + offset) as u8 as char
            } else if c.is_ascii_uppercase() {
                let base = b'A' as i32;
                let offset = (c as i32 - base + s) % 26;
                (base + offset) as u8 as char
            } else {
                c
            }
        })
        .collect()
}

// ==========================================
// INVERSION
// ==========================================
/// Inverse l'ordre des graphèmes : un emoji composé ou une lettre suivie
/// d'un accent combinant reste intact.
fn reverse_graphemes(text: &str) -> String {
    text.graphemes(true).rev().collect()
}

// ==========================================
// URL PERCENT ENCODING
// ==========================================
fn url_encode(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len() * 2);
    for b in text.as_bytes() {
        match *b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(*b as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

fn url_decode(text: &str) -> Result<String, String> {
    let mut bytes = Vec::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next().ok_or("Caractère '%' incomplet dans l'URL.")?;
            let h2 = chars.next().ok_or("Caractère '%' incomplet dans l'URL.")?;
            let hex_str = format!("{}{}", h1, h2);
            let byte = u8::from_str_radix(&hex_str, 16)
                .map_err(|_| format!("Séquence hexadécimale URL invalide: %{}", hex_str))?;
            bytes.push(byte);
        } else if c == '+' {
            bytes.push(b' ');
        } else {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            bytes.extend_from_slice(s.as_bytes());
        }
    }

    utf8_text(bytes)
}

// ==========================================
// HTML ENTITIES
// ==========================================
fn html_escape(text: &str) -> String {
    let mut res = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => res.push_str("&amp;"),
            '<' => res.push_str("&lt;"),
            '>' => res.push_str("&gt;"),
            '"' => res.push_str("&quot;"),
            '\'' => res.push_str("&#39;"),
            other => res.push(other),
        }
    }
    res
}

// ==========================================
// PUNYCODE (RFC 3492 / IDN)
// ==========================================
/// Applique `convert` à chaque mot (suite de caractères non blancs)
/// et conserve les blancs tels quels, retours à la ligne compris.
fn map_words(
    input: &str,
    mut convert: impl FnMut(&str) -> Result<String, String>,
) -> Result<String, String> {
    let mut output = String::with_capacity(input.len());
    let mut word_start = None;
    for (i, c) in input.char_indices() {
        if c.is_whitespace() {
            if let Some(start) = word_start.take() {
                output.push_str(&convert(&input[start..i])?);
            }
            output.push(c);
        } else if word_start.is_none() {
            word_start = Some(i);
        }
    }
    if let Some(start) = word_start {
        output.push_str(&convert(&input[start..])?);
    }
    Ok(output)
}

/// Mode IDN : chaque mot non ASCII est traité comme un nom de domaine (UTS #46 :
/// minuscules, normalisation, préfixe `xn--`). Mode brut : chaque mot est encodé
/// tel quel selon la RFC 3492, sans préfixe.
fn punycode_encode(input: &str, idn: bool) -> Result<String, String> {
    map_words(input, |word| {
        if !idn {
            punycode::encode_str(word)
                .ok_or_else(|| format!("Impossible d'encoder « {word} » en Punycode."))
        } else if word.is_ascii() {
            Ok(word.to_string())
        } else {
            idna::domain_to_ascii(word).map_err(|_| {
                format!("« {word} » n'est pas un nom de domaine internationalisé valide.")
            })
        }
    })
}

/// Mode IDN : seuls les labels qui commencent par `xn--` sont décodés, le reste du
/// texte est conservé tel quel. Mode brut : chaque mot est décodé selon la RFC 3492.
fn punycode_decode(input: &str, idn: bool) -> Result<String, String> {
    map_words(input, |word| {
        if !idn {
            return punycode::decode_to_string(word)
                .ok_or_else(|| format!("Punycode invalide : « {word} »."));
        }
        let labels = word
            .split('.')
            .map(|label| match label.get(..4) {
                // Les noms de domaine ignorent la casse : « XN--MNCHEN-3YA » donne « münchen ».
                Some(prefix) if prefix.eq_ignore_ascii_case("xn--") => {
                    punycode::decode_to_string(&label[4..].to_ascii_lowercase())
                        .ok_or_else(|| format!("Label Punycode invalide : « {label} »."))
                }
                _ => Ok(label.to_string()),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(labels.join("."))
    })
}

// ==========================================
// TESTS UNITAIRES
// ==========================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_roundtrip() {
        let opts = ConvertOptions::default();
        let original = "Bonjour le monde ! 123";
        let encoded = encode(original, ConverterFormat::Hex, &opts).unwrap();
        let decoded = decode(&encoded, ConverterFormat::Hex, &opts).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_binary_roundtrip() {
        let opts = ConvertOptions::default();
        let original = "Tauri Rust 🦀";
        let encoded = encode(original, ConverterFormat::Binary, &opts).unwrap();
        let decoded = decode(&encoded, ConverterFormat::Binary, &opts).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_base64_roundtrip() {
        let opts = ConvertOptions::default();
        let original = "Texte secret en clair";
        let encoded = encode(original, ConverterFormat::Base64, &opts).unwrap();
        let decoded = decode(&encoded, ConverterFormat::Base64, &opts).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_base32_roundtrip() {
        let opts = ConvertOptions::default();
        let original = "Hello Base32!";
        let encoded = encode(original, ConverterFormat::Base32, &opts).unwrap();
        let decoded = decode(&encoded, ConverterFormat::Base32, &opts).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_morse_roundtrip() {
        let opts = ConvertOptions::default();
        let original = "SOS HELLO WORLD";
        let encoded = encode(original, ConverterFormat::Morse, &opts).unwrap();
        let decoded = decode(&encoded, ConverterFormat::Morse, &opts).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_rot13_roundtrip() {
        let opts = ConvertOptions {
            caesar_shift: Some(13),
            ..Default::default()
        };
        let original = "Attack at Dawn!";
        let encoded = encode(original, ConverterFormat::Caesar, &opts).unwrap();
        let decoded = decode(&encoded, ConverterFormat::Caesar, &opts).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_url_roundtrip() {
        let opts = ConvertOptions::default();
        let original = "https://example.com/test?query=bonjour le monde&lang=fr";
        let encoded = encode(original, ConverterFormat::Url, &opts).unwrap();
        let decoded = decode(&encoded, ConverterFormat::Url, &opts).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_html_roundtrip() {
        let opts = ConvertOptions::default();
        let original = "<div class=\"box\">L'éléphant & la souris</div>";
        let encoded = encode(original, ConverterFormat::Html, &opts).unwrap();
        let decoded = decode(&encoded, ConverterFormat::Html, &opts).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_ascii_dec_roundtrip() {
        let opts = ConvertOptions::default();
        let original = "Rust 2026";
        let encoded = encode(original, ConverterFormat::AsciiDec, &opts).unwrap();
        let decoded = decode(&encoded, ConverterFormat::AsciiDec, &opts).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_ascii_oct_roundtrip() {
        let opts = ConvertOptions::default();
        let original = "Hello Octal";
        let encoded = encode(original, ConverterFormat::AsciiOct, &opts).unwrap();
        let decoded = decode(&encoded, ConverterFormat::AsciiOct, &opts).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_reverse() {
        let opts = ConvertOptions::default();
        let original = "antigravity";
        let encoded = encode(original, ConverterFormat::Reverse, &opts).unwrap();
        assert_eq!(encoded, "ytivargitna");
        let decoded = decode(&encoded, ConverterFormat::Reverse, &opts).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_invalid_hex_error() {
        let opts = ConvertOptions::default();
        let invalid_hex = "48 65 6c 6c ZZ";
        let err = decode(invalid_hex, ConverterFormat::Hex, &opts);
        assert!(err.is_err());
    }

    #[test]
    fn test_invalid_binary_error() {
        let opts = ConvertOptions::default();
        let invalid_bin = "01001002";
        let err = decode(invalid_bin, ConverterFormat::Binary, &opts);
        assert!(err.is_err());
    }

    #[test]
    fn test_punycode_roundtrip() {
        let opts = ConvertOptions::default();
        let domain = "café.fr";
        let encoded = encode(domain, ConverterFormat::Punycode, &opts).unwrap();
        assert_eq!(encoded, "xn--caf-dma.fr");
        let decoded = decode(&encoded, ConverterFormat::Punycode, &opts).unwrap();
        assert_eq!(decoded, domain);

        // Test with sentence
        let sentence = "münchen café";
        let enc_sentence = encode(sentence, ConverterFormat::Punycode, &opts).unwrap();
        let dec_sentence = decode(&enc_sentence, ConverterFormat::Punycode, &opts).unwrap();
        assert_eq!(dec_sentence, sentence);
    }
}
