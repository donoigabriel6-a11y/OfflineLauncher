//! OfflineLauncher - Main entry point

use offline_launcher_lib::run;

#[cfg_attr(windows, windows_subsystem = "windows")]
fn main() {
    run();
}