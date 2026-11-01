// Shared TypeScript types matching Rust backend

export interface Account {
  id: string;
  username: string;
  uuid: string;
  accountType: 'offline' | 'microsoft';
  skinPath?: string;
  createdAt: string;
  updatedAt: string;
}

export interface OfflineAccount extends Account {
  accountType: 'offline';
  accessToken: '0';
  userType: 'legacy';
}

export interface VersionEntry {
  id: string;
  type: 'release' | 'snapshot' | 'old_beta' | 'old_alpha';
  url: string;
  time: string;
  releaseTime: string;
  sha1?: string;
  complianceLevel: number;
}

export interface VersionManifest {
  latest: {
    release: string;
    snapshot: string;
  };
  versions: VersionEntry[];
}

export interface VersionJson {
  id: string;
  inheritsFrom?: string;
  javaVersion: {
    component: string;
    majorVersion: number;
  };
  mainClass: string;
  assetIndex: {
    id: string;
    url: string;
    sha1: string;
    size: number;
    totalSize: number;
  };
  assets: string;
  libraries: Library[];
  arguments: Arguments;
  minecraftArguments?: string;
  downloads: {
    client: DownloadInfo;
    server?: DownloadInfo;
    serverMappings?: DownloadInfo;
  };
  logging?: Logging;
}

export interface Library {
  name: string;
  downloads?: {
    artifact?: DownloadInfo;
    classifiers?: Record<string, DownloadInfo>;
  };
  natives?: Record<string, string>;
  rules: LibraryRule[];
  extract?: {
    exclude: string[];
  };
  url?: string;
}

export interface DownloadInfo {
  url: string;
  sha1: string;
  size: number;
}

export interface LibraryRule {
  action: 'allow' | 'disallow';
  os?: {
    name: 'windows' | 'osx' | 'linux' | 'windows-arm' | 'osx-arm' | 'linux-arm' | 'linux-aarch64';
    version?: string;
    arch?: string;
  };
  features?: Record<string, boolean>;
}

export interface Arguments {
  game: Argument[];
  jvm: Argument[];
}

export type Argument = string | ConditionalArgument;

export interface ConditionalArgument {
  rules: LibraryRule[];
  value: string | string[];
}

export interface Logging {
  client: {
    argument: string;
    file: {
      id: string;
      sha1: string;
      size: number;
      url: string;
    };
    type: string;
  };
}

export interface Instance {
  id: string;
  name: string;
  icon?: string;
  versionId: string;
  loader: ModLoader;
  loaderVersion?: string;
  gameDir: string;
  java: JavaSettings;
  memory: MemorySettings;
  jvmArgs: string;
  gameArgs: string;
  resolution?: Resolution;
  lastPlayed?: string;
  createdAt: string;
  updatedAt: string;
}

export type ModLoader = 'none' | 'fabric' | 'quilt' | 'forge' | 'neoforge';

export interface JavaSettings {
  path?: string;
  majorVersion?: number;
  jvmArgs: string;
}

export interface MemorySettings {
  minMb: number;
  maxMb: number;
}

export interface Resolution {
  width: number;
  height: number;
  fullscreen: boolean;
}

export interface JavaInstallation {
  path: string;
  version: string;
  majorVersion: number;
  vendor: string;
  isJre: boolean;
}

export interface JavaSelection {
  path: string;
  version: string;
  majorVersion: number;
  source: 'system' | 'mojang' | 'adoptium' | 'custom';
}

export interface Config {
  dataDir?: string;
  concurrentDownloads: number;
  defaultMemoryMb: number;
  defaultJavaVersion?: number;
  defaultJavaPath?: string;
  defaultJvmArgs: string;
  language: string;
  theme: 'dark' | 'light' | 'system';
  closeOnLaunch: boolean;
  showAdvanced: boolean;
  lastInstanceId?: string;
  lastAccountId?: string;
  window: {
    width: number;
    height: number;
    maximized: boolean;
  };
}

export interface DownloadProgress {
  versionId: string;
  fileName: string;
  downloaded: number;
  total: number;
  stage: DownloadStage;
  speedBps?: number;
  etaSeconds?: number;
}

export type DownloadStage =
  | 'version_json'
  | 'client_jar'
  | 'libraries'
  | 'assets_index'
  | 'assets'
  | 'natives'
  | 'complete';

export interface LaunchOutput {
  instanceId: string;
  line: string;
  stream: 'stdout' | 'stderr';
}

export interface LaunchStatus {
  instanceId: string;
  status: 'starting' | 'running' | 'exited' | 'crashed' | 'killed';
  exitCode?: number;
  pid?: number;
}

export interface FabricVersion {
  version: string;
  stable: boolean;
}

export interface FabricLoaderVersion {
  version: string;
  stable: boolean;
  build: number;
}

export interface QuiltVersion {
  version: string;
  stable: boolean;
}

export interface QuiltLoaderVersion {
  version: string;
  build: number;
  stable: boolean;
}