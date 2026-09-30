//! Code Morse international (ITU-R M.1677-1).

use crate::options::{Choice, OptionKind, OptionSpec};
use crate::text::as_text;
use crate::{Category, Codec, CodecError, CodecMeta, ErrorCode, Options};

const UNKNOWN: OptionSpec = OptionSpec {
    id: "unknown",
    label: "Caractères sans code",
    kind: OptionKind::Choice {
        default: "transliterate",
        choices: &[
            Choice {
                value: "error",
                label: "Erreur",
            },
            Choice {
                value: "transliterate",
                label: "Translittérer (à → A)",
            },
            Choice {
                value: "ignore",
                label: "Ignorer",
            },
        ],
    },
};

static META: CodecMeta = CodecMeta {
    id: "morse",
    label: "Code Morse",
    icon: "•−",
    category: Category::Text,
    aliases: &[],
    reversible: true,
    encodes_text: true,
    options: &[UNKNOWN],
};

pub struct Morse;

impl Codec for Morse {
    fn meta(&self) -> &'static CodecMeta {
        &META
    }

    /// Lettres séparées par une espace, mots par « / », lignes par un retour à la ligne.
    fn encode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        let unknown = options.choice(&UNKNOWN)?;

        // Chaque caractère est gardé, translittéré, écarté ou refusé selon `unknown`.
        let mut normalized = String::with_capacity(input.len());
        for (index, c) in as_text(input)?.chars().enumerate() {
            if c.is_whitespace() || morse_char(c).is_some() {
                normalized.push(c);
                continue;
            }
            match (unknown, transliterate(c)) {
                ("transliterate" | "ignore", Some(base)) => normalized.push_str(base),
                ("ignore", None) => {}
                (_, base) => {
                    let hint = match base {
                        Some(base) => {
                            format!("choisissez « Translittérer » pour le remplacer par « {base} »")
                        }
                        None => "choisissez « Ignorer » pour l'écarter".to_string(),
                    };
                    return Err(CodecError::at(
                        ErrorCode::UnsupportedCharacter,
                        index,
                        format!(
                            "« {c} » (position {}) n'existe pas en Morse : {hint}.",
                            index + 1
                        ),
                    ));
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
        Ok(lines.join("\n").into_bytes())
    }

    fn decode(&self, input: &[u8], _options: &Options) -> Result<Vec<u8>, CodecError> {
        let mut text = String::new();
        let mut code = String::new();
        let mut code_start = 0;

        // Ajoute la lettre du code en cours, ou signale un code inconnu.
        let flush = |code: &mut String, start: usize, text: &mut String| {
            if code.is_empty() {
                return Ok(());
            }
            let letter = morse_to_char(code).ok_or_else(|| {
                CodecError::spanning(
                    ErrorCode::UnknownSymbol,
                    start,
                    start + code.chars().count(),
                    format!("Code Morse inconnu « {code} » en position {}.", start + 1),
                )
            })?;
            text.push(letter);
            code.clear();
            Ok::<(), CodecError>(())
        };

        for (index, c) in as_text(input)?.chars().enumerate() {
            match c {
                '.' | '-' => {
                    if code.is_empty() {
                        code_start = index;
                    }
                    code.push(c);
                }
                '/' => {
                    flush(&mut code, code_start, &mut text)?;
                    text.push(' ');
                }
                '\n' => {
                    flush(&mut code, code_start, &mut text)?;
                    text.push('\n');
                }
                c if c.is_whitespace() => flush(&mut code, code_start, &mut text)?,
                _ => {
                    return Err(CodecError::at(
                        ErrorCode::InvalidCharacter,
                        index,
                        format!(
                            "Caractère « {c} » inattendu en position {} : le Morse n'utilise que « . », « - » et « / ».",
                            index + 1
                        ),
                    ))
                }
            }
        }
        flush(&mut code, code_start, &mut text)?;
        Ok(text.into_bytes())
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

fn morse_char(c: char) -> Option<&'static str> {
    Some(match c.to_ascii_uppercase() {
        'A' => ".-",
        'B' => "-...",
        'C' => "-.-.",
        'D' => "-..",
        'E' => ".",
        'F' => "..-.",
        'G' => "--.",
        'H' => "....",
        'I' => "..",
        'J' => ".---",
        'K' => "-.-",
        'L' => ".-..",
        'M' => "--",
        'N' => "-.",
        'O' => "---",
        'P' => ".--.",
        'Q' => "--.-",
        'R' => ".-.",
        'S' => "...",
        'T' => "-",
        'U' => "..-",
        'V' => "...-",
        'W' => ".--",
        'X' => "-..-",
        'Y' => "-.--",
        'Z' => "--..",
        '0' => "-----",
        '1' => ".----",
        '2' => "..---",
        '3' => "...--",
        '4' => "....-",
        '5' => ".....",
        '6' => "-....",
        '7' => "--...",
        '8' => "---..",
        '9' => "----.",
        '.' => ".-.-.-",
        ',' => "--..--",
        '?' => "..--..",
        '\'' => ".----.",
        '!' => "-.-.--",
        '/' => "-..-.",
        '(' => "-.--.",
        ')' => "-.--.-",
        '&' => ".-...",
        ':' => "---...",
        ';' => "-.-.-.",
        '=' => "-...-",
        '+' => ".-.-.",
        '-' => "-....-",
        '_' => "..--.-",
        '"' => ".-..-.",
        '$' => "...-..-",
        '@' => ".--.-.",
        // Seule lettre accentuée de la recommandation ITU.
        'É' | 'é' => "..-..",
        _ => return None,
    })
}

fn morse_to_char(code: &str) -> Option<char> {
    Some(match code {
        ".-" => 'A',
        "-..." => 'B',
        "-.-." => 'C',
        "-.." => 'D',
        "." => 'E',
        "..-." => 'F',
        "--." => 'G',
        "...." => 'H',
        ".." => 'I',
        ".---" => 'J',
        "-.-" => 'K',
        ".-.." => 'L',
        "--" => 'M',
        "-." => 'N',
        "---" => 'O',
        ".--." => 'P',
        "--.-" => 'Q',
        ".-." => 'R',
        "..." => 'S',
        "-" => 'T',
        "..-" => 'U',
        "...-" => 'V',
        ".--" => 'W',
        "-..-" => 'X',
        "-.--" => 'Y',
        "--.." => 'Z',
        "-----" => '0',
        ".----" => '1',
        "..---" => '2',
        "...--" => '3',
        "....-" => '4',
        "....." => '5',
        "-...." => '6',
        "--..." => '7',
        "---.." => '8',
        "----." => '9',
        ".-.-.-" => '.',
        "--..--" => ',',
        "..--.." => '?',
        ".----." => '\'',
        "-.-.--" => '!',
        "-..-." => '/',
        "-.--." => '(',
        "-.--.-" => ')',
        ".-..." => '&',
        "---..." => ':',
        "-.-.-." => ';',
        "-...-" => '=',
        ".-.-." => '+',
        "-....-" => '-',
        "..--.-" => '_',
        ".-..-." => '"',
        "...-..-" => '$',
        ".--.-." => '@',
        "..-.." => 'É',
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codecs::testing::{dec, enc};

    #[test]
    fn keeps_lines_and_the_e_acute() {
        let encoded = enc(&Morse, "SOS ÉTÉ\nOK", &Options::new()).unwrap();
        assert_eq!(encoded, "... --- ... / ..-.. - ..-..\n--- -.-");
        assert_eq!(
            dec(&Morse, &encoded, &Options::new()).unwrap(),
            "SOS ÉTÉ\nOK"
        );
    }

    #[test]
    fn unknown_characters_follow_the_option() {
        let with = |mode| Options::new().with("unknown", mode);
        assert_eq!(enc(&Morse, "ça", &Options::new()).unwrap(), "-.-. .-");
        assert_eq!(enc(&Morse, "à🦀b", &with("ignore")).unwrap(), ".- -...");
        let error = enc(&Morse, "ça", &with("error")).unwrap_err();
        assert_eq!(error.code, ErrorCode::UnsupportedCharacter);
        assert!(error.message.contains("Translittérer"), "{}", error.message);
    }

    #[test]
    fn unknown_codes_are_located() {
        let error = dec(&Morse, "... ------- ...", &Options::new()).unwrap_err();
        assert_eq!(error.code, ErrorCode::UnknownSymbol);
        assert_eq!(error.span.map(|s| (s.start, s.end)), Some((4, 11)));
    }
}
