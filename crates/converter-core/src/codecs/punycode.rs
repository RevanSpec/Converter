//! Punycode (RFC 3492) et noms de domaine internationalisés (IDNA, UTS #46).

use crate::options::{Choice, OptionKind, OptionSpec};
use crate::text::{as_text, map_words};
use crate::{Category, Codec, CodecError, CodecMeta, ErrorCode, Options};
use idna::punycode;

const MODE: OptionSpec = OptionSpec {
    id: "mode",
    label: "Format de sortie",
    kind: OptionKind::Choice {
        default: "idn",
        choices: &[
            Choice {
                value: "idn",
                label: "Préfixe xn-- (Standard IDN)",
            },
            Choice {
                value: "raw",
                label: "Punycode brut (RFC 3492)",
            },
        ],
    },
};

static META: CodecMeta = CodecMeta {
    id: "punycode",
    label: "Punycode (IDN)",
    icon: "xn--",
    category: Category::Web,
    aliases: &["puny", "idn"],
    reversible: true,
    encodes_text: true,
    options: &[MODE],
};

pub struct Punycode;

impl Codec for Punycode {
    fn meta(&self) -> &'static CodecMeta {
        &META
    }

    /// Mode IDN : chaque mot non ASCII est traité comme un nom de domaine (minuscules,
    /// normalisation, préfixe `xn--`). Mode brut : chaque mot suit la RFC 3492, sans préfixe.
    fn encode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        let idn = options.choice(&MODE)? == "idn";
        let text = map_words(as_text(input)?, |word, start| {
            let span = |code, message: String| {
                CodecError::spanning(code, start, start + word.chars().count(), message)
            };
            if !idn {
                punycode::encode_str(word).ok_or_else(|| {
                    span(
                        ErrorCode::Internal,
                        format!("Impossible d'encoder « {word} » en Punycode."),
                    )
                })
            } else if word.is_ascii() {
                Ok(word.to_string())
            } else {
                idna::domain_to_ascii(word).map_err(|_| {
                    span(
                        ErrorCode::InvalidDomain,
                        format!("« {word} » n'est pas un nom de domaine internationalisé valide."),
                    )
                })
            }
        })?;
        Ok(text.into_bytes())
    }

    /// Mode IDN : seuls les labels qui commencent par `xn--` sont décodés, le reste du
    /// texte est conservé. Mode brut : chaque mot est décodé selon la RFC 3492.
    fn decode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError> {
        let idn = options.choice(&MODE)? == "idn";
        let text = map_words(as_text(input)?, |word, start| {
            let invalid = |label: &str| {
                CodecError::spanning(
                    ErrorCode::InvalidPunycode,
                    start,
                    start + word.chars().count(),
                    format!("Punycode invalide : « {label} »."),
                )
            };
            if !idn {
                return punycode::decode_to_string(word).ok_or_else(|| invalid(word));
            }
            let labels = word
                .split('.')
                .map(|label| match label.get(..4) {
                    // Les noms de domaine ignorent la casse : « XN--MNCHEN-3YA » donne « münchen ».
                    Some(prefix) if prefix.eq_ignore_ascii_case("xn--") => {
                        punycode::decode_to_string(&label[4..].to_ascii_lowercase())
                            .ok_or_else(|| invalid(label))
                    }
                    _ => Ok(label.to_string()),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(labels.join("."))
        })?;
        Ok(text.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codecs::testing::{dec, enc};

    #[test]
    fn idn_mode_decodes_only_prefixed_labels() {
        assert_eq!(
            enc(&Punycode, "Café.fr", &Options::new()).unwrap(),
            "xn--caf-dma.fr"
        );
        assert_eq!(
            dec(&Punycode, "example.com xn--caf-dma.com", &Options::new()).unwrap(),
            "example.com café.com"
        );
    }

    #[test]
    fn raw_mode_roundtrips_every_word() {
        let raw = Options::new().with("mode", "raw");
        assert_eq!(
            enc(&Punycode, "münchen abc", &raw).unwrap(),
            "mnchen-3ya abc-"
        );
        assert_eq!(
            dec(&Punycode, "mnchen-3ya abc-", &raw).unwrap(),
            "münchen abc"
        );
    }

    #[test]
    fn invalid_label_is_located() {
        let error = dec(&Punycode, "voir xn--caf-dm!.fr", &Options::new()).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidPunycode);
        assert_eq!(error.span.map(|s| (s.start, s.end)), Some((5, 19)));
    }
}
