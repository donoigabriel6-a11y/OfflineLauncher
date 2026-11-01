# OfflineLauncher - Project Plan

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tauri 2 App                              │
├─────────────────────────────────────────────────────────────────┤
│  Frontend (React + TypeScript + Vite + Tailwind)               │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐  │
│  │ Home    │ │Accounts │ │Versions │ │Instances│ │Settings │  │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘  │
├─────────────────────────────────────────────────────────────────┤
│  Tauri IPC Bridge (Commands + Events)                          │
├─────────────────────────────────────────────────────────────────┤
│  Backend (Rust)                                                 │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │ accounts │ │ versions │ │ downloads│ │  java    │          │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │instances │ │  launch  │ │ loaders  │ │ config   │          │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

## Folder Structure

```
OfflineLauncher/
├── .github/
│   └── workflows/
│       └── ci.yml
├── src/
│   ├── main.rs                 # Tauri entry point
│   ├── lib.rs                  # Module declarations
│   ├── config.rs               # App configuration
│   ├── app_dirs.rs             # Platform-correct app data directories
│   ├── error.rs                # Error types
│   ├── events.rs               # Tauri event definitions
│   │
│   ├── accounts/
│   │   ├── mod.rs
│   │   ├── model.rs            # Account trait, OfflineAccount, AccountType
│   │   ├── storage.rs          # JSON file persistence
│   │   └── uuid.rs             # Offline UUID generation (v3 MD5)
│   │
│   ├── versions/
│   │   ├── mod.rs
│   │   ├── manifest.rs         # Version manifest fetching/parsing
│   │   ├── model.rs            # Version JSON structures
│   │   ├── library.rs          # Library rules, natives, classifiers
│   │   └── inheritance.rs      # inheritsFrom resolution
│   │
│   ├── downloads/
│   │   ├── mod.rs
│   │   ├── manager.rs          # Download queue, concurrency, retry
│   │   ├── hash.rs             # SHA1 verification
│   │   └── progress.rs         # Progress reporting
│   │
│   ├── java/
│   │   ├── mod.rs
│   │   ├── detection.rs        # System Java detection
│   │   ├── runtime.rs          # Runtime manifest (Mojang/Adoptium)
│   │   └── manager.rs          # Java selection & download
│   │
│   ├── instances/
│   │   ├── mod.rs
│   │   ├── model.rs            # Instance struct
│   │   └── storage.rs          # Instance persistence
│   │
│   ├── launch/
│   │   ├── mod.rs
│   │   ├── arguments.rs        # Placeholder substitution, argument parsing
│   │   ├── classpath.rs        # Classpath construction
│   │   └── process.rs          # Process spawning, stdio streaming
│   │
│   └── loaders/
│       ├── mod.rs
│       ├── fabric.rs           # Fabric API
│       └── quilt.rs            # Quilt API
│
├── src-tauri/ (Tauri config)
│   ├── tauri.conf.json
│   ├── build.rs
│   └── Cargo.toml
│
├── frontend/
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── styles/
│   │   │   └── globals.css
│   │   ├── components/
│   │   │   ├── layout/
│   │   │   ├── accounts/
│   │   │   ├── versions/
│   │   │   ├── instances/
│   │   │   ├── settings/
│   │   │   └── console/
│   │   ├── pages/
│   │   │   ├── Home.tsx
│   │   │   ├── Accounts.tsx
│   │   │   ├── Versions.tsx
│   │   │   ├── Instances.tsx
│   │   │   └── Settings.tsx
│   │   ├── hooks/
│   │   ├── services/
│   │   │   ├── api.ts          # Tauri invoke wrappers
│   │   │   └── events.ts       # Tauri event listeners
│   │   ├── store/
│   │   │   └── useStore.ts     # Zustand store
│   │   └── types/
│   │       └── index.ts        # Shared TypeScript types
│   ├── index.html
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   └── tailwind.config.js
│
├── tests/
│   ├── accounts_uuid_test.rs
│   ├── arguments_test.rs
│   ├── library_rules_test.rs
│   ├── hash_test.rs
│   └── integration_test.rs
│
├── README.md
├── Cargo.toml (workspace root)
└── .rustfmt.toml / clippy.toml / eslint.config.js / prettier.config.js
```

## Data Models

### Account
```rust
pub trait Account: Send + Sync {
    fn id(&self) -> Uuid;
    fn username(&self) -> &str;
    fn uuid(&self) -> Uuid;           // Offline: UUID v3 of "OfflinePlayer:<username>"
    fn access_token(&self) -> &str;   // Offline: "0"
    fn user_type(&self) -> UserType;  // Offline: "legacy"
    fn account_type(&self) -> AccountType;
    fn skin_path(&self) -> Option<PathBuf>;
}

#[derive(Serialize, Deserialize)]
pub struct OfflineAccount {
    id: Uuid,
    username: String,
    uuid: Uuid,
    skin_path: Option<PathBuf>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

### Version
```rust
#[derive(Serialize, Deserialize)]
pub struct Version {
    id: String,
    type_: VersionType,  // release, snapshot, old_beta, old_alpha
    time: DateTime<Utc>,
    release_time: DateTime<Utc>,
    url: String,         // Version JSON URL
    sha1: Option<String>,
    compliance_level: u32,
}

#[derive(Serialize, Deserialize)]
pub struct VersionJson {
    id: String,
    inherits_from: Option<String>,
    java_version: JavaVersion,
    main_class: String,
    asset_index: AssetIndex,
    assets: String,
    libraries: Vec<Library>,
    arguments: Arguments,  // game + jvm with rules
    minecraft_arguments: Option<String>,  // Legacy
    downloads: Downloads,
}
```

### Instance
```rust
#[derive(Serialize, Deserialize)]
pub struct Instance {
    id: Uuid,
    name: String,
    icon: Option<String>,  // Base64 or path
    version_id: String,
    loader: Option<ModLoader>,  // Fabric/Quilt/Forge/NeoForge
    game_dir: PathBuf,
    java: JavaSettings,
    memory: MemorySettings,
    jvm_args: String,
    game_args: String,
    resolution: Option<Resolution>,
    last_played: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

### Java Settings
```rust
#[derive(Serialize, Deserialize)]
pub struct JavaSettings {
    path: Option<PathBuf>,  // Override, None = auto-detect
    major_version: Option<u8>,  // Override, None = from version JSON
    jvm_args: String,
}
```

## Milestones

### (a) Project Scaffold + Config/App Dirs
- [ ] Initialize Tauri 2 + React + TypeScript + Vite project
- [ ] Configure Tailwind CSS
- [ ] Set up Rust workspace with modules
- [ ] Implement platform-correct app data directories
- [ ] Config system (TOML/JSON) with defaults
- [ ] Basic Tauri commands/events infrastructure
- [ ] Lint/format configuration (clippy, rustfmt, eslint, prettier)
- [ ] GitHub Actions CI workflow (Windows, macOS, Linux)

### (b) Offline Accounts
- [ ] Account trait + OfflineAccount implementation
- [ ] UUID v3 generation (MD5 of "OfflinePlayer:<username>")
- [ ] Username validation (3-16 chars, [A-Za-z0-9_])
- [ ] JSON storage in app data dir
- [ ] CRUD operations (create, read, update, delete, list)
- [ ] Skin preview (local file)
- [ ] Tauri commands for account management
- [ ] React UI: Accounts page with list, add/edit/delete modal
- [ ] Unit tests: UUID generation, validation

### (c) Version Manifest + Downloader
- [ ] Fetch official version manifest
- [ ] Parse and cache version list (release, snapshot, old_beta, old_alpha)
- [ ] Version JSON download + parsing
- [ ] Library rules evaluation (OS/arch filtering)
- [ ] Natives handling (classifiers, extraction)
- [ ] inheritsFrom resolution
- [ ] Download manager: concurrent (8), resume, retry, SHA1 verify
- [ ] Progress events streamed to UI
- [ ] Tauri commands: fetch versions, download version, get progress
- [ ] React UI: Versions page with filter, download progress
- [ ] Unit tests: library rules, hash verification, inheritance

### (d) Java Detection/Download
- [ ] Detect system Javas (JAVA_HOME, PATH, common dirs)
- [ ] Parse java -version output
- [ ] Auto-select Java major version from version JSON
- [ ] Download runtime (Mojang manifest → Adoptium Temurin fallback)
- [ ] Store in launcher's runtimes dir
- [ ] Per-instance Java override
- [ ] Tauri commands: detect java, download java, list runtimes
- [ ] React UI: Java settings in instance editor
- [ ] Unit tests: version parsing, detection

### (e) Launch Pipeline + Working Vanilla Launch
- [ ] Placeholder substitution (all `${...}` vars)
- [ ] Argument parsing: legacy `minecraftArguments` + modern `arguments` with rules
- [ ] Classpath construction (libraries + client jar)
- [ ] Natives directory extraction
- [ ] Process spawning with stdio streaming
- [ ] Console UI with real-time output
- [ ] Running/exited state, exit code, kill button
- [ ] Crash detection + crash report viewer
- [ ] Tauri commands: launch, kill, get logs
- [ ] React UI: Home/Play page with instance/account picker, Play button, Console
- [ ] **Verification**: Launch vanilla 1.20.1 with offline account → main menu
- [ ] Integration test: download + prepare 1.20.1 (no launch)

### (f) Instances (Profiles)
- [ ] Instance model + storage (per-instance game dir)
- [ ] CRUD + clone + open folder
- [ ] Instance-specific settings (Java, memory, JVM args, game args, resolution)
- [ ] Last-played tracking
- [ ] Tauri commands for instance management
- [ ] React UI: Instances page (grid/list), create/edit modal
- [ ] Integration with launch pipeline

### (g) Full UI Polish
- [ ] Dark/light theme toggle (persisted)
- [ ] Responsive layout, sidebar navigation
- [ ] Home/Play: instance picker + account picker + big Play button
- [ ] Settings page: data dir, concurrent downloads, default memory, default Java, language scaffold, close-on-launch
- [ ] Loading states, error toasts, confirmations
- [ ] Keyboard shortcuts
- [ ] Accessibility basics

### (h) Fabric/Quilt Support
- [ ] Fabric API: versions, loader versions, installer JSON
- [ ] Quilt API: same
- [ ] Install loader into instance (merge via inheritsFrom)
- [ ] Loader version selection in instance editor
- [ ] Tauri commands: fetch loaders, install loader
- [ ] React UI: Loader selector

### (i) Tests/CI/README
- [ ] Unit tests: UUID, arguments, library rules, hash
- [ ] Integration test: download/prepare 1.20.1
- [ ] GitHub Actions: build + test on 3 OSes
- [ ] README: build/run, architecture, add account provider guide
- [ ] clippy + rustfmt + eslint + prettier in CI

## Tauri Commands (Planned)

| Module | Commands |
|--------|----------|
| accounts | `accounts_list`, `accounts_create`, `accounts_update`, `accounts_delete`, `accounts_get` |
| versions | `versions_fetch_manifest`, `versions_list`, `versions_download`, `versions_get_progress`, `versions_cancel` |
| java | `java_detect`, `java_list_runtimes`, `java_download`, `java_set_default` |
| instances | `instances_list`, `instances_create`, `instances_update`, `instances_delete`, `instances_clone`, `instances_open_folder` |
| launch | `launch_start`, `launch_kill`, `launch_get_status`, `launch_get_logs` |
| loaders | `loaders_fabric_versions`, `loaders_quilt_versions`, `loaders_install` |
| config | `config_get`, `config_set`, `config_open_data_dir` |

## Tauri Events (Planned)

| Event | Payload |
|-------|---------|
| `download_progress` | `{ version_id, file, downloaded, total, stage }` |
| `download_complete` | `{ version_id, success, error? }` |
| `launch_output` | `{ instance_id, line, stream: stdout|stderr }` |
| `launch_status` | `{ instance_id, status: starting|running|exited|crashed, exit_code? }` |
| `accounts_changed` | `{ action: created|updated|deleted, account }` |
| `instances_changed` | `{ action: created|updated|deleted, instance }` |

## Next Steps

Start with Milestone (a): Project Scaffold + Config/App Dirs