//! Mod loader support (Fabric, Quilt)

pub mod fabric;
pub mod quilt;

pub use fabric::{fetch_fabric_versions, fetch_fabric_loader_versions, install_fabric};
pub use quilt::{fetch_quilt_versions, fetch_quilt_loader_versions, install_quilt};