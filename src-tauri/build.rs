// Tauri build script
use tauri_build::{BuildContext, WindowsAttributes, LinuxAttributes, MacOSAttributes};

fn main() {
    tauri_build::try_build(
        BuildContext::new()
            .windows_attributes(WindowsAttributes::new().app_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo>
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
  <compatibility>
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}" /> <!-- Windows 10 -->
      <supportedOS Id="{1f676c76-80e1-4239-95bb-83d0f6d0da78}" /> <!-- Windows 8.1 -->
      <supportedOS Id="{4a2f28e3-53b9-4441-ba9c-d69d4a4a6e38}" /> <!-- Windows 8 -->
    </application>
  </compatibility>
</assembly>
"#))
            .linux_attributes(LinuxAttributes::new().desktop_entry(r#"
[Desktop Entry]
Type=Application
Name=OfflineLauncher
Comment=Cross-platform Minecraft Java Edition launcher with offline account support
Exec=offline-launcher
Icon=offline-launcher
Categories=Game;
StartupNotify=true
"#))
            .macos_attributes(MacOSAttributes::new().info_plist(r#"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDisplayName</key>
    <string>OfflineLauncher</string>
    <key>CFBundleIdentifier</key>
    <string>com.offlinelauncher.app</string>
    <key>CFBundleName</key>
    <string>OfflineLauncher</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
</dict>
</plist>
"#))
    ).expect("Failed to build Tauri app");
}