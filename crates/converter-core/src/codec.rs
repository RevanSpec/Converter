use crate::{CodecError, OptionSpec, Options};
use serde::{Deserialize, Serialize};

/// Sens de la conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Encode,
    Decode,
}

impl Direction {
    /// L'autre sens.
    pub fn reversed(self) -> Self {
        match self {
            Direction::Encode => Direction::Decode,
            Direction::Decode => Direction::Encode,
        }
    }

    /// Nom de l'opération, pour les messages : « encodage » ou « décodage ».
    pub(crate) fn noun(self) -> &'static str {
        match self {
            Direction::Encode => "encodage",
            Direction::Decode => "décodage",
        }
    }
}

/// Famille de formats, pour regrouper les onglets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
#[serde(rename_all = "snake_case")]
pub enum Category {
    /// Représentations d'octets : hexadécimal, binaire, bases 64 et 32…
    Bytes,
    /// Formats du web : URL, HTML, noms de domaine.
    Web,
    /// Transformations du texte lui-même.
    Text,
    /// Chiffrements.
    Cipher,
    /// Compression : des octets en entrée comme en sortie.
    Compression,
}

/// Plafond de la sortie d'une conversion, et de chaque couche d'une chaîne : au-delà,
/// la conversion échoue au lieu de saturer la mémoire (bombe de décompression, encodage
/// binaire d'un texte énorme…).
pub const MAX_OUTPUT_BYTES: usize = 100_000_000;

/// Description d'un format pour l'interface et les recettes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS), ts(export))]
pub struct CodecMeta {
    /// Identifiant stable : interface, recettes, ligne de commande.
    pub id: &'static str,
    pub label: &'static str,
    /// Libellé court affiché dans l'onglet.
    pub icon: &'static str,
    pub category: Category,
    /// Autres noms acceptés par le registre.
    pub aliases: &'static [&'static str],
    /// Décoder l'encodage redonne le texte d'origine, s'il est bien formé.
    pub reversible: bool,
    /// L'encodage attend du texte UTF-8 ; sinon, il accepte des octets quelconques.
    pub encodes_text: bool,
    pub options: &'static [OptionSpec],
}

/// Un format de conversion : des octets en entrée, des octets en sortie.
///
/// Pour ajouter un format : un fichier dans `codecs/` qui implémente ce trait,
/// puis une ligne dans `codecs::all`.
pub trait Codec: Send + Sync {
    fn meta(&self) -> &'static CodecMeta;
    fn encode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError>;
    fn decode(&self, input: &[u8], options: &Options) -> Result<Vec<u8>, CodecError>;
}
