//! Version management module
//!
//! Handles fetching version manifests, parsing version JSONs,
//! library resolution, and inheritance.

pub mod manifest;
pub mod model;
pub mod library;
pub mod inheritance;

pub use manifest::{fetch_version_manifest, VersionManifest, VersionEntry};
pub use model::{VersionJson, Library, AssetIndex, Downloads, Arguments, Argument, ArgumentRule, JavaVersion};
pub use library::{evaluate_library_rules, LibraryRule, LibraryRuleAction, LibraryRuleOs};
pub use inheritance::resolve_inheritance;