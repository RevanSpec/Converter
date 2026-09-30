//! Moteur de conversion de Glass Converter, sans dépendance à Tauri.
//!
//! Chaque format est un [`Codec`] qui transforme des octets en octets. Le
//! [`Registry`] les rassemble et décrit leurs options, ce qui permet à
//! l'interface de se construire toute seule à partir de [`Registry::list`].

mod codec;
mod codecs;
mod conversion;
mod error;
mod options;
mod registry;
mod text;

pub use codec::{Category, Codec, CodecMeta, Direction};
pub use conversion::{convert, ConvertRequest, ConvertResponse, Output};
pub use error::{CodecError, ErrorCode, Span};
pub use options::{Choice, OptionKind, OptionSpec, OptionValue, Options};
pub use registry::{registry, Registry};
