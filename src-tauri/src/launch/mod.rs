//! Launch pipeline: argument substitution, classpath, process spawning

pub mod arguments;
pub mod classpath;
pub mod process;

pub use arguments::{LaunchArguments, substitute_placeholders};
pub use classpath::{build_classpath, get_natives_dir, extract_natives, library_artifact_path, library_native_path};
pub use process::{launch_game, LaunchHandle, GameProcess};