//! « Mes recettes » : un fichier JSON dans le dossier de configuration de l'application.
//!
//! Il n'est écrit que lorsque l'utilisateur enregistre ou supprime une recette, et ne
//! contient que des chaînes de formats : jamais de texte saisi ni d'option secrète.

use converter_core::{CodecError, ErrorCode, RecipeBook};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const FILE_NAME: &str = "recipes.json";

/// Chemin du fichier, par exemple `%APPDATA%\com.converter.glass\recipes.json` sous Windows.
pub fn path(app: &AppHandle) -> Result<PathBuf, CodecError> {
    app.path()
        .app_config_dir()
        .map(|dir| dir.join(FILE_NAME))
        .map_err(|error| storage(format!("dossier de configuration introuvable ({error})")))
}

/// Recettes enregistrées ; aucune tant que le fichier n'existe pas.
pub fn read(path: &Path) -> Result<RecipeBook, CodecError> {
    match fs::read_to_string(path) {
        Ok(json) => serde_json::from_str(&json)
            .map_err(|error| storage(format!("{} est illisible ({error})", path.display()))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(RecipeBook::default()),
        Err(error) => Err(storage(format!(
            "lecture de {} impossible ({error})",
            path.display()
        ))),
    }
}

/// Applique `change` aux recettes et réécrit le fichier. Un fichier illisible n'est pas
/// écrasé : il est d'abord mis de côté sous le nom `recipes.json.bak`.
pub fn update(
    path: &Path,
    change: impl FnOnce(&mut RecipeBook) -> Result<(), CodecError>,
) -> Result<RecipeBook, CodecError> {
    let mut book = match read(path) {
        Ok(book) => book,
        Err(_) if path.exists() => {
            let backup = path.with_extension("json.bak");
            fs::rename(path, &backup).map_err(|error| {
                storage(format!(
                    "{} est illisible et n'a pas pu être mis de côté ({error})",
                    path.display()
                ))
            })?;
            RecipeBook::default()
        }
        Err(error) => return Err(error),
    };
    change(&mut book)?;
    write(path, &book)?;
    Ok(book)
}

/// Écrit dans un fichier temporaire puis le renomme : jamais de fichier à moitié écrit.
fn write(path: &Path, book: &RecipeBook) -> Result<(), CodecError> {
    let failed = |error: io::Error| {
        storage(format!(
            "écriture de {} impossible ({error})",
            path.display()
        ))
    };
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(failed)?;
    }
    let json = serde_json::to_string_pretty(book)
        .map_err(|error| storage(format!("recettes impossibles à écrire ({error})")))?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, json).map_err(failed)?;
    fs::rename(&temporary, path).map_err(failed)
}

fn storage(detail: String) -> CodecError {
    CodecError::new(ErrorCode::Storage, format!("Mes recettes : {detail}."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use converter_core::{registry, Direction, Recipe, Step};

    /// Dossier temporaire propre à chaque test, supprimé à la fin.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("glass-converter-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn recipe() -> Recipe {
        Recipe::export(registry(), &[Step::new("base64", Direction::Decode)]).unwrap()
    }

    #[test]
    fn nothing_is_written_until_a_recipe_is_saved() {
        let dir = TempDir::new("vide");
        let file = dir.0.join(FILE_NAME);
        assert_eq!(read(&file).unwrap(), RecipeBook::default());
        assert!(!dir.0.exists());

        let book = update(&file, |book| book.save("Base64", recipe())).unwrap();
        assert_eq!(book.recipes.len(), 1);
        assert_eq!(read(&file).unwrap(), book);

        let book = update(&file, |book| {
            book.remove("Base64");
            Ok(())
        })
        .unwrap();
        assert!(book.recipes.is_empty());
    }

    #[test]
    fn an_unreadable_file_is_set_aside_instead_of_overwritten() {
        let dir = TempDir::new("illisible");
        let file = dir.0.join(FILE_NAME);
        fs::create_dir_all(&dir.0).unwrap();
        fs::write(&file, "{ pas du JSON").unwrap();

        assert_eq!(read(&file).unwrap_err().code, ErrorCode::Storage);
        update(&file, |book| book.save("Base64", recipe())).unwrap();
        assert_eq!(
            fs::read_to_string(dir.0.join("recipes.json.bak")).unwrap(),
            "{ pas du JSON"
        );
        assert_eq!(read(&file).unwrap().recipes.len(), 1);
    }

    #[test]
    fn a_refused_change_leaves_the_file_untouched() {
        let dir = TempDir::new("refus");
        let file = dir.0.join(FILE_NAME);
        update(&file, |book| book.save("Base64", recipe())).unwrap();
        let before = fs::read_to_string(&file).unwrap();

        assert!(update(&file, |book| book.save("   ", recipe())).is_err());
        assert_eq!(fs::read_to_string(&file).unwrap(), before);
    }
}
