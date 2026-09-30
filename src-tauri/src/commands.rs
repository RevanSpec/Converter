use crate::converters::{convert, ConvertOptions, ConvertResult, ConverterFormat};

#[tauri::command]
pub fn convert_text(
    input: String,
    format: String,
    to_encoded: bool,
    options: Option<ConvertOptions>,
) -> Result<ConvertResult, String> {
    let parsed_format: ConverterFormat = format.parse()?;
    let opts = options.unwrap_or_default();
    convert(&input, parsed_format, to_encoded, &opts)
}
