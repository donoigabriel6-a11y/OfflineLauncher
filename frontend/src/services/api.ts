// Tauri API invoke wrappers
import { invoke } from '@tauri-apps/api/core';
import type {
  Account,
  VersionEntry,
  VersionManifest,
  VersionJson,
  Instance,
  JavaInstallation,
  JavaSelection,
  Config,
  FabricVersion,
  FabricLoaderVersion,
  QuiltVersion,
  QuiltLoaderVersion,
} from '@/types';

// Config
export const configApi = {
  get: () => invoke<Config>('config_get'),
  set: (config: Config) => invoke('config_set', { config }),
  openDataDir: () => invoke<string>('config_open_data_dir'),
};

// Accounts
export const accountsApi = {
  list: () => invoke<Account[]>('accounts_list'),
  create: (username: string) => invoke<Account>('accounts_create', { username }),
  update: (id: string, username?: string, skinPath?: string) =>
    invoke<Account>('accounts_update', { id, username, skinPath }),
  delete: (id: string) => invoke('accounts_delete', { id }),
  generateUuid: (username: string) => invoke<string>('accounts_generate_uuid', { username }),
};

// Versions
export const versionsApi = {
  fetchManifest: () => invoke<VersionManifest>('versions_fetch_manifest'),
  list: (versionType?: string) => invoke<VersionEntry[]>('versions_list', { versionType }),
  download: (versionId: string) => invoke('versions_download', { versionId }),
  getVersionJson: (versionId: string) =>
    invoke<VersionJson>('versions_get_version_json', { versionId }),
};

// Java
export const javaApi = {
  detect: () => invoke<JavaInstallation[]>('java_detect'),
  listRuntimes: () => invoke<JavaSelection[]>('java_list_runtimes'),
  resolve: (requiredMajor: number, overridePath?: string) =>
    invoke<JavaSelection>('java_resolve', { requiredMajor, overridePath }),
};

// Instances
export const instancesApi = {
  list: () => invoke<Instance[]>('instances_list'),
  create: (name: string, versionId: string) =>
    invoke<Instance>('instances_create', { name, versionId }),
  update: (instance: Instance) => invoke<Instance>('instances_update', { instance }),
  delete: (id: string) => invoke('instances_delete', { id }),
  clone: (id: string, newName: string) => invoke<Instance>('instances_clone', { id, newName }),
  openFolder: (id: string) => invoke('instances_open_folder', { id }),
};

// Launch
export const launchApi = {
  start: (instanceId: string, accountId: string) =>
    invoke('launch_start', { instanceId, accountId }),
  kill: (instanceId: string) => invoke('launch_kill', { instanceId }),
};

// Loaders
export const loadersApi = {
  fabricVersions: () => invoke<FabricVersion[]>('loaders_fabric_versions'),
  fabricLoaderVersions: () => invoke<FabricLoaderVersion[]>('loaders_fabric_loader_versions'),
  quiltVersions: () => invoke<QuiltVersion[]>('loaders_quilt_versions'),
  quiltLoaderVersions: () => invoke<QuiltLoaderVersion[]>('loaders_quilt_loader_versions'),
};
