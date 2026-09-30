use converter_core::{registry, CodecError, CodecMeta, ConvertRequest, ConvertResponse, ErrorCode};

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
        .map_err(|e| {
            CodecError::new(
                ErrorCode::Internal,
                format!("La conversion a été interrompue : {e}"),
            )
        })?
}
