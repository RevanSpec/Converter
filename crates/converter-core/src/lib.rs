//! Moteur de conversion de Glass Converter, sans dépendance à Tauri.
//!
//! Chaque format est un [`Codec`] qui transforme des octets en octets. Le
//! [`Registry`] les rassemble et décrit leurs options, ce qui permet à
//! l'interface de se construire toute seule à partir de [`Registry::list`].
//! Une chaîne de couches ([`Step`]) enchaîne plusieurs formats ; une [`Recipe`]
//! la décrit pour la partager ou l'enregistrer.

mod codec;
mod codecs;
mod conversion;
mod error;
mod options;
mod pipeline;
mod recipe;
mod registry;
mod text;

pub use codec::{Category, Codec, CodecMeta, Direction, MAX_OUTPUT_BYTES};
pub use conversion::{convert, ConvertRequest, ConvertResponse, Output};
pub use error::{CodecError, ErrorCode, Span};
pub use options::{Choice, OptionKind, OptionSpec, OptionValue, Options};
pub use pipeline::{
    apply_chain, invert_chain, run_pipeline, PipelineError, PipelineRequest, PipelineResponse,
    Step, StepReport, StepStatus, PREVIEW_BYTES,
};
pub use recipe::{
    parse_short, to_short, Preset, Recipe, RecipeBook, SavedRecipe, MAX_NAME_CHARS, PRESETS,
    RECIPE_VERSION,
};
pub use registry::{registry, Registry};
