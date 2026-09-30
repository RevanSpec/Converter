//! Outils partagés par les formats qui lisent du texte.

use crate::{CodecError, ErrorCode};

/// L'entrée en tant que texte, ou une erreur si ses octets ne sont pas de l'UTF-8 valide.
pub(crate) fn as_text(input: &[u8]) -> Result<&str, CodecError> {
    std::str::from_utf8(input).map_err(|error| {
        CodecError::new(
            ErrorCode::NotText,
            format!(
                "Ce format attend du texte, mais l'entrée n'est pas de l'UTF-8 valide (octet {}).",
                error.valid_up_to()
            ),
        )
    })
}

/// Retire les blancs de `input` et garde, pour chaque caractère conservé,
/// son index (à partir de 0) dans le texte saisi.
pub(crate) fn strip_whitespace(input: &str) -> (Vec<char>, Vec<usize>) {
    input
        .chars()
        .enumerate()
        .filter(|(_, c)| !c.is_whitespace())
        .map(|(index, c)| (c, index))
        .unzip()
}

/// Nombres séparés par des blancs, `,` ou `;`, avec la plage de caractères de chacun.
pub(crate) fn number_tokens(input: &str) -> impl Iterator<Item = (usize, usize, &str)> + '_ {
    let is_separator = |c: char| c.is_whitespace() || c == ',' || c == ';';
    let mut tokens = Vec::new();
    let mut token: Option<(usize, usize)> = None; // (octet de début, index du premier caractère)
    let mut char_count = 0;
    for (char_index, (byte_index, c)) in input.char_indices().enumerate() {
        if is_separator(c) {
            if let Some((start, first_char)) = token.take() {
                tokens.push((first_char, char_index, &input[start..byte_index]));
            }
        } else if token.is_none() {
            token = Some((byte_index, char_index));
        }
        char_count = char_index + 1;
    }
    if let Some((start, first_char)) = token {
        tokens.push((first_char, char_count, &input[start..]));
    }
    tokens.into_iter()
}

/// Applique `convert` à chaque mot (suite de caractères non blancs) en conservant les
/// blancs tels quels, retours à la ligne compris. `convert` reçoit le mot et l'index
/// de son premier caractère dans le texte saisi.
pub(crate) fn map_words(
    input: &str,
    mut convert: impl FnMut(&str, usize) -> Result<String, CodecError>,
) -> Result<String, CodecError> {
    let mut output = String::with_capacity(input.len());
    let mut word: Option<(usize, usize)> = None; // (octet de début, index du premier caractère)
    for (char_index, (byte_index, c)) in input.char_indices().enumerate() {
        if c.is_whitespace() {
            if let Some((start, first_char)) = word.take() {
                output.push_str(&convert(&input[start..byte_index], first_char)?);
            }
            output.push(c);
        } else if word.is_none() {
            word = Some((byte_index, char_index));
        }
    }
    if let Some((start, first_char)) = word {
        output.push_str(&convert(&input[start..], first_char)?);
    }
    Ok(output)
}
