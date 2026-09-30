use base64::{engine::general_purpose, Engine as _};
use idna::punycode;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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
        ConverterFormat::Morse => Ok(text_to_morse(input)),
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
        ConverterFormat::Reverse => Ok(input.chars().rev().collect()),
        ConverterFormat::Punycode => {
            let use_prefix = options.punycode_prefix.unwrap_or(true);
            Ok(punycode_encode(input, use_prefix))
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
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    match format {
        ConverterFormat::Hex => {
            // Nettoyage : retirer préfixes 0x, espaces, virgules, deux-points
            let cleaned = trimmed
                .replace("0x", "")
                .replace("0X", "")
                .replace([' ', ':', ',', '\n', '\r', '\t'], "");

            if !cleaned.len().is_multiple_of(2) {
                return Err(format!(
                    "Longueur hexadécimale impaire ({} caractères). Chaque octet nécessite 2 caractères hexadécimaux.",
                    cleaned.len()
                ));
            }

            let bytes = hex::decode(&cleaned).map_err(|e| match e {
                hex::FromHexError::InvalidHexCharacter { c, index } => {
                    format!(
                        "Caractère hexadécimal invalide '{}' à la position {}.",
                        c, index
                    )
                }
                hex::FromHexError::OddLength => "Longueur hexadécimale impaire.".to_string(),
                _ => format!("Erreur de décodage hexadécimal: {}", e),
            })?;

            String::from_utf8(bytes).map_err(|e| {
                format!(
                    "Les octets décodés ne constituent pas un texte UTF-8 valide (erreur à l'octet {}).",
                    e.utf8_error().valid_up_to()
                )
            })
        }
        ConverterFormat::Binary => {
            let cleaned: String = trimmed
                .chars()
                .filter(|c| !c.is_whitespace() && *c != ',' && *c != '-')
                .collect();

            if cleaned.is_empty() {
                return Ok(String::new());
            }

            for (i, c) in cleaned.chars().enumerate() {
                if c != '0' && c != '1' {
                    return Err(format!(
                        "Caractère binaire invalide '{}' à la position {}. Seuls '0' et '1' sont autorisés.",
                        c, i
                    ));
                }
            }

            if !cleaned.len().is_multiple_of(8) {
                return Err(format!(
                    "Longueur binaire invalide ({} bits). Elle doit être un multiple de 8 bits.",
                    cleaned.len()
                ));
            }

            let mut bytes = Vec::with_capacity(cleaned.len() / 8);
            for chunk in cleaned.as_bytes().chunks(8) {
                let s = std::str::from_utf8(chunk).unwrap();
                let byte = u8::from_str_radix(s, 2)
                    .map_err(|_| "Erreur de conversion binaire".to_string())?;
                bytes.push(byte);
            }

            String::from_utf8(bytes).map_err(|e| {
                format!(
                    "Les octets décodés ne constituent pas un texte UTF-8 valide (erreur à l'octet {}).",
                    e.utf8_error().valid_up_to()
                )
            })
        }
        ConverterFormat::Base64 => {
            let cleaned: String = trimmed.chars().filter(|c| !c.is_whitespace()).collect();
            let url_safe = options.base64_url_safe.unwrap_or(false);

            let bytes = if url_safe {
                general_purpose::URL_SAFE
                    .decode(&cleaned)
                    .or_else(|_| general_purpose::STANDARD.decode(&cleaned))
            } else {
                general_purpose::STANDARD
                    .decode(&cleaned)
                    .or_else(|_| general_purpose::URL_SAFE.decode(&cleaned))
            }
            .map_err(|e| format!("Chaîne Base64 invalide: {}", e))?;

            String::from_utf8(bytes).map_err(|e| {
                format!(
                    "Les octets décodés ne constituent pas un texte UTF-8 valide (erreur à l'octet {}).",
                    e.utf8_error().valid_up_to()
                )
            })
        }
        ConverterFormat::Base32 => {
            let bytes = base32_decode(trimmed)?;
            String::from_utf8(bytes).map_err(|e| {
                format!(
                    "Les octets décodés ne constituent pas un texte UTF-8 valide (erreur à l'octet {}).",
                    e.utf8_error().valid_up_to()
                )
            })
        }
        ConverterFormat::Morse => morse_to_text(trimmed),
        ConverterFormat::AsciiDec => {
            let tokens = trimmed.split(|c: char| c.is_whitespace() || c == ',' || c == ';');
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
            String::from_utf8(bytes).map_err(|e| {
                format!(
                    "Les octets décodés ne constituent pas un texte UTF-8 valide (erreur à l'octet {}).",
                    e.utf8_error().valid_up_to()
                )
            })
        }
        ConverterFormat::AsciiOct => {
            let tokens = trimmed.split(|c: char| c.is_whitespace() || c == ',' || c == ';');
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
            String::from_utf8(bytes).map_err(|e| {
                format!(
                    "Les octets décodés ne constituent pas un texte UTF-8 valide (erreur à l'octet {}).",
                    e.utf8_error().valid_up_to()
                )
            })
        }
        ConverterFormat::Url => url_decode(trimmed),
        ConverterFormat::Html => html_unescape(trimmed),
        ConverterFormat::Caesar => {
            let shift = options.caesar_shift.unwrap_or(13);
            Ok(caesar_shift(trimmed, -shift))
        }
        ConverterFormat::Reverse => Ok(trimmed.chars().rev().collect()),
        ConverterFormat::Punycode => punycode_decode(trimmed),
    }
}

// ==========================================
// BASE32 IMPLEMENTATION (RFC 4648)
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

fn base32_decode(input: &str) -> Result<Vec<u8>, String> {
    let cleaned: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    let mut buffer: u64 = 0;
    let mut bits_left = 0;
    let mut result = Vec::new();

    for (pos, ch) in cleaned.chars().enumerate() {
        if ch == '=' {
            break;
        }
        let upper = ch.to_ascii_uppercase();
        let val = match upper {
            'A'..='Z' => (upper as u8 - b'A') as u64,
            '2'..='7' => (upper as u8 - b'2' + 26) as u64,
            _ => {
                return Err(format!(
                    "Caractère Base32 invalide '{}' à la position {}.",
                    ch, pos
                ))
            }
        };

        buffer = (buffer << 5) | val;
        bits_left += 5;

        if bits_left >= 8 {
            bits_left -= 8;
            result.push(((buffer >> bits_left) & 0xFF) as u8);
        }
    }

    Ok(result)
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
        _ => None,
    }
}

fn text_to_morse(text: &str) -> String {
    let mut words = Vec::new();
    for word in text.split_whitespace() {
        let mut letters = Vec::new();
        for ch in word.chars() {
            if let Some(m) = morse_char(ch) {
                letters.push(m.to_string());
            } else {
                letters.push(ch.to_string());
            }
        }
        words.push(letters.join(" "));
    }
    words.join(" / ")
}

fn morse_to_text(morse: &str) -> Result<String, String> {
    let mut result = String::new();
    let words = morse.split('/');

    for (w_idx, word) in words.enumerate() {
        if w_idx > 0 {
            result.push(' ');
        }
        for token in word.split_whitespace() {
            if token.is_empty() {
                continue;
            }
            if let Some(c) = morse_to_char(token) {
                result.push(c);
            } else {
                return Err(format!("Code Morse invalide ou non reconnu: '{}'", token));
            }
        }
    }

    Ok(result)
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

    String::from_utf8(bytes).map_err(|e| {
        format!(
            "La chaîne décodée ne forme pas un texte UTF-8 valide (erreur à l'octet {}).",
            e.utf8_error().valid_up_to()
        )
    })
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

fn html_unescape(text: &str) -> Result<String, String> {
    let mut res = String::new();
    let mut i = 0;
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    while i < len {
        if chars[i] == '&' {
            // Find terminating ';'
            let mut end = i + 1;
            while end < len && end - i < 12 && chars[end] != ';' {
                end += 1;
            }

            if end < len && chars[end] == ';' {
                let entity: String = chars[i + 1..end].iter().collect();
                if let Some(ch) = match entity.as_str() {
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "quot" => Some('"'),
                    "apos" | "#39" => Some('\''),
                    "nbsp" => Some('\u{00A0}'),
                    "copy" => Some('©'),
                    "reg" => Some('®'),
                    "euro" => Some('€'),
                    s if s.starts_with("#x") || s.starts_with("#X") => {
                        u32::from_str_radix(&s[2..], 16)
                            .ok()
                            .and_then(char::from_u32)
                    }
                    s if s.starts_with('#') => s[1..].parse::<u32>().ok().and_then(char::from_u32),
                    _ => None,
                } {
                    res.push(ch);
                    i = end + 1;
                    continue;
                }
            }
        }
        res.push(chars[i]);
        i += 1;
    }

    Ok(res)
}

// ==========================================
// PUNYCODE IMPLEMENTATION (RFC 3492 / IDN)
// ==========================================
fn punycode_encode(input: &str, use_prefix: bool) -> String {
    let mut result_lines = Vec::new();

    for line in input.lines() {
        if line.contains('.') {
            let labels: Vec<String> = line
                .split('.')
                .map(|label| {
                    if label.is_ascii() {
                        label.to_string()
                    } else {
                        match punycode::encode_str(label) {
                            Some(encoded) => {
                                if use_prefix {
                                    format!("xn--{}", encoded)
                                } else {
                                    encoded
                                }
                            }
                            None => label.to_string(),
                        }
                    }
                })
                .collect();
            result_lines.push(labels.join("."));
        } else if line.contains(' ') {
            let words: Vec<String> = line
                .split(' ')
                .map(|word| {
                    if word.is_ascii() {
                        word.to_string()
                    } else {
                        match punycode::encode_str(word) {
                            Some(encoded) => {
                                if use_prefix {
                                    format!("xn--{}", encoded)
                                } else {
                                    encoded
                                }
                            }
                            None => word.to_string(),
                        }
                    }
                })
                .collect();
            result_lines.push(words.join(" "));
        } else if line.is_ascii() {
            result_lines.push(line.to_string());
        } else {
            match punycode::encode_str(line) {
                Some(encoded) => {
                    if use_prefix {
                        result_lines.push(format!("xn--{}", encoded));
                    } else {
                        result_lines.push(encoded);
                    }
                }
                None => result_lines.push(line.to_string()),
            }
        }
    }

    result_lines.join("\n")
}

fn punycode_decode(input: &str) -> Result<String, String> {
    let mut result_lines = Vec::new();

    for line in input.lines() {
        if line.contains('.') {
            let mut labels = Vec::new();
            for label in line.split('.') {
                let clean = label.trim();
                if clean.to_ascii_lowercase().starts_with("xn--") {
                    let raw = &clean[4..];
                    let decoded = punycode::decode_to_string(raw)
                        .ok_or_else(|| format!("Punycode invalide dans le label: '{}'", clean))?;
                    labels.push(decoded);
                } else if let Some(decoded) = punycode::decode_to_string(clean) {
                    labels.push(decoded);
                } else {
                    labels.push(clean.to_string());
                }
            }
            result_lines.push(labels.join("."));
        } else if line.contains(' ') {
            let mut words = Vec::new();
            for word in line.split(' ') {
                let clean = word.trim();
                if clean.to_ascii_lowercase().starts_with("xn--") {
                    let raw = &clean[4..];
                    let decoded = punycode::decode_to_string(raw)
                        .ok_or_else(|| format!("Punycode invalide: '{}'", clean))?;
                    words.push(decoded);
                } else if let Some(decoded) = punycode::decode_to_string(clean) {
                    words.push(decoded);
                } else {
                    words.push(clean.to_string());
                }
            }
            result_lines.push(words.join(" "));
        } else {
            let clean = line.trim();
            if clean.to_ascii_lowercase().starts_with("xn--") {
                let raw = &clean[4..];
                let decoded = punycode::decode_to_string(raw)
                    .ok_or_else(|| format!("Punycode invalide: '{}'", clean))?;
                result_lines.push(decoded);
            } else if let Some(decoded) = punycode::decode_to_string(clean) {
                result_lines.push(decoded);
            } else {
                return Err(format!(
                    "Impossible de décoder la séquence Punycode: '{}'",
                    clean
                ));
            }
        }
    }

    Ok(result_lines.join("\n"))
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
