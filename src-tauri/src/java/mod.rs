//! Java runtime detection and management

pub mod detection;
pub mod runtime;
pub mod manager;

pub use detection::{detect_system_java, JavaInstallation};
pub use runtime::{fetch_java_runtimes, JavaRuntime, JavaRuntimeManifest};
pub use manager::{JavaManager, JavaSelection};