//! Launch pipeline: argument substitution, classpath, process spawning

pub mod arguments;
pub mod classpath;
pub mod process;

pub use arguments::{substitute_placeholders, LaunchArguments};
pub use classpath::{
    build_classpath, extract_natives, get_natives_dir, library_artifact_path, library_native_path,
};
pub use process::{launch_game, GameProcess, LaunchHandle};
