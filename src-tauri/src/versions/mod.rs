//! Version management module
//!
//! Handles fetching version manifests, parsing version JSONs,
//! library resolution, and inheritance.

pub mod manifest;
pub mod model;
pub mod library;
pub mod inheritance;

pub use manifest::fetch_version_manifest;
pub use model::{VersionManifest, VersionEntry, VersionType};
pub use model::{VersionJson, Library, AssetIndex, Downloads, Arguments, Argument, JavaVersion, LibraryRule, LibraryRuleAction, LibraryRuleOs, LibraryRuleOsName};
pub use library::evaluate_library_rules;
pub use inheritance::resolve_inheritance;