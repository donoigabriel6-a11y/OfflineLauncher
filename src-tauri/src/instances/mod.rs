//! Instance (profile) management

pub mod model;
pub mod storage;

pub use model::{Instance, JavaSettings, MemorySettings, ModLoader, Resolution};
pub use storage::InstanceStorage;
