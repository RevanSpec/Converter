use crate::{Codec, CodecMeta};
use std::sync::OnceLock;

/// Ensemble des formats disponibles.
pub struct Registry {
    codecs: Vec<Box<dyn Codec>>,
}

impl Registry {
    /// Tous les formats de l'application, dans l'ordre des onglets.
    pub fn standard() -> Self {
        Self {
            codecs: crate::codecs::all(),
        }
    }

    /// Format désigné par son identifiant ou l'un de ses alias, sans tenir compte de la casse.
    pub fn get(&self, name: &str) -> Option<&dyn Codec> {
        let name = name.to_lowercase();
        self.codecs.iter().map(Box::as_ref).find(|codec| {
            let meta = codec.meta();
            meta.id == name || meta.aliases.contains(&name.as_str())
        })
    }

    pub fn list(&self) -> Vec<&'static CodecMeta> {
        self.codecs.iter().map(|codec| codec.meta()).collect()
    }
}

/// Registre standard, créé au premier appel et partagé ensuite.
pub fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(Registry::standard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn identifiers_and_aliases_are_unique() {
        let mut names = HashSet::new();
        for meta in registry().list() {
            for name in std::iter::once(&meta.id).chain(meta.aliases) {
                assert!(names.insert(*name), "nom en double : {name}");
            }
        }
    }

    #[test]
    fn lookup_ignores_case_and_accepts_aliases() {
        assert_eq!(registry().get("ROT13").map(|c| c.meta().id), Some("caesar"));
        assert_eq!(registry().get("b64").map(|c| c.meta().id), Some("base64"));
        assert!(registry().get("inconnu").is_none());
    }
}
