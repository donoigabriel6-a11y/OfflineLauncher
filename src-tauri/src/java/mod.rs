//! Java runtime detection and management

pub mod detection;
pub mod manager;
pub mod runtime;

pub use detection::{detect_system_java, JavaInstallation};
pub use manager::{JavaManager, JavaSelection, JavaSource};
pub use runtime::{fetch_java_runtimes, JavaRuntime, JavaRuntimeManifest};
