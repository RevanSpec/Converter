use crate::converters::{convert, ConvertOptions, ConvertResult, ConverterFormat};

/// La conversion tourne sur un thread de travail : une commande synchrone
/// s'exécuterait sur le thread de l'interface et figerait la fenêtre.
#[tauri::command]
pub async fn convert_text(
    input: String,
    format: String,
    to_encoded: bool,
    options: Option<ConvertOptions>,
) -> Result<ConvertResult, String> {
    let parsed_format: ConverterFormat = format.parse()?;
    let opts = options.unwrap_or_default();
    tauri::async_runtime::spawn_blocking(move || convert(&input, parsed_format, to_encoded, &opts))
        .await
        .map_err(|e| format!("La conversion a été interrompue : {e}"))?
}
