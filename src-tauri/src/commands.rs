use crate::recipes;
use converter_core::{
    parse_short, registry, to_short, CodecError, CodecMeta, ConvertRequest, ConvertResponse,
    ErrorCode, PipelineError, PipelineRequest, PipelineResponse, Preset, Recipe, SavedRecipe, Step,
    PRESETS,
};
use tauri::AppHandle;

/// Formats disponibles et leurs options : l'interface se construit à partir de cette liste.
#[tauri::command]
pub fn list_codecs() -> Vec<&'static CodecMeta> {
    registry().list()
}

/// La conversion tourne sur un thread de travail : une commande synchrone
/// s'exécuterait sur le thread de l'interface et figerait la fenêtre.
#[tauri::command]
pub async fn convert(request: ConvertRequest) -> Result<ConvertResponse, CodecError> {
    tauri::async_runtime::spawn_blocking(move || converter_core::convert(registry(), &request))
        .await
        .map_err(interrupted)?
}

/// Toute la chaîne en un seul appel, avec un compte rendu par couche.
#[tauri::command]
pub async fn run_pipeline(request: PipelineRequest) -> Result<PipelineResponse, CodecError> {
    tauri::async_runtime::spawn_blocking(move || converter_core::run_pipeline(registry(), &request))
        .await
        .map_err(interrupted)
}

/// Chaîne inverse : ordre et sens inversés.
#[tauri::command]
pub fn invert_chain(steps: Vec<Step>) -> Result<Vec<Step>, PipelineError> {
    converter_core::invert_chain(registry(), &steps)
}

/// Forme courte d'une chaîne, pour la copier.
#[tauri::command]
pub fn recipe_to_short(steps: Vec<Step>) -> Result<String, CodecError> {
    to_short(registry(), &steps)
}

/// Chaîne décrite par une forme courte collée par l'utilisateur.
#[tauri::command]
pub fn recipe_from_short(text: String) -> Result<Vec<Step>, CodecError> {
    parse_short(registry(), &text)
}

#[tauri::command]
pub fn list_presets() -> Vec<Preset> {
    PRESETS.to_vec()
}

/// Les commandes de « Mes recettes » lisent et écrivent un petit fichier : elles restent
/// asynchrones pour ne jamais attendre le disque sur le thread de l'interface.
#[tauri::command]
pub async fn list_saved_recipes(app: AppHandle) -> Result<Vec<SavedRecipe>, CodecError> {
    Ok(recipes::read(&recipes::path(&app)?)?.recipes)
}

#[tauri::command]
pub async fn save_recipe(
    app: AppHandle,
    name: String,
    steps: Vec<Step>,
) -> Result<Vec<SavedRecipe>, CodecError> {
    let recipe = Recipe::export(registry(), &steps)?;
    let book = recipes::update(&recipes::path(&app)?, |book| book.save(&name, recipe))?;
    Ok(book.recipes)
}

#[tauri::command]
pub async fn delete_saved_recipe(
    app: AppHandle,
    name: String,
) -> Result<Vec<SavedRecipe>, CodecError> {
    let book = recipes::update(&recipes::path(&app)?, |book| {
        book.remove(&name);
        Ok(())
    })?;
    Ok(book.recipes)
}

fn interrupted(error: tauri::Error) -> CodecError {
    CodecError::new(
        ErrorCode::Internal,
        format!("La conversion a été interrompue : {error}"),
    )
}
