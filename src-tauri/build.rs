// Tauri 2 build script
fn main() {
    tauri_build::build();

    // Set Windows subsystem to GUI (no console window) using winres
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_subsystem(winres::Subsystem::Windows);
        res.compile().expect("Failed to compile Windows resources");
    }
}
