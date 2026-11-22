//! Version management module
//!
//! Handles fetching version manifests, parsing version JSONs,
//! library resolution, and inheritance.

pub mod inheritance;
pub mod library;
pub mod manifest;
pub mod model;

pub use inheritance::resolve_inheritance;
pub use library::evaluate_library_rules;
pub use manifest::fetch_version_manifest;
pub use model::{
    Argument, Arguments, AssetIndex, AssetIndexObject, Downloads, JavaVersion, Library,
    LibraryRule, LibraryRuleAction, LibraryRuleOs, LibraryRuleOsName, VersionJson,
};
pub use model::{VersionEntry, VersionManifest, VersionType};
