// Tauri 2 build script
fn main() {
    tauri_build::build();
    // Set Windows subsystem to GUI (no console window)
    // Use try_windows_subsystem which is available in tauri-build 2
    let _ = tauri_build::try_windows_subsystem("windows");
}
