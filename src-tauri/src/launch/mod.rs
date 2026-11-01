//! Launch pipeline: argument substitution, classpath, process spawning

pub mod arguments;
pub mod classpath;
pub mod process;

pub use arguments::{LaunchArguments, substitute_placeholders};
pub use classpath::build_classpath;
pub use process::{launch_game, LaunchHandle, GameProcess};