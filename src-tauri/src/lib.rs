pub mod commands;
mod recipes;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_codecs,
            commands::convert,
            commands::run_pipeline,
            commands::invert_chain,
            commands::recipe_to_short,
            commands::recipe_from_short,
            commands::list_presets,
            commands::list_saved_recipes,
            commands::save_recipe,
            commands::delete_saved_recipe
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
