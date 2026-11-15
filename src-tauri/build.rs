// Tauri 2 build script
fn main() {
    tauri_build::build();
    tauri_build::windows_subsystem("windows");
}
