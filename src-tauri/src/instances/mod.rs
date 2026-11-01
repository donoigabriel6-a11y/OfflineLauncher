//! Instance (profile) management

pub mod model;
pub mod storage;

pub use model::{Instance, ModLoader, MemorySettings, JavaSettings, Resolution};
pub use storage::InstanceStorage;