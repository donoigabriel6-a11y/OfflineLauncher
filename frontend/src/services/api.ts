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

// Helper to convert Tauri string errors to Error objects
function handleTauriError<T>(promise: Promise<T>): Promise<T> {
  return promise.catch((error: unknown) => {
    if (typeof error === 'string') {
      throw new Error(error);
    }
    throw error;
  });
}

// Config
export const configApi = {
  get: () => handleTauriError(invoke<Config>('config_get')),
  set: (config: Config) => handleTauriError(invoke('config_set', { config })),
  openDataDir: () => handleTauriError(invoke<string>('config_open_data_dir')),
};

// Accounts
export const accountsApi = {
  list: () => handleTauriError(invoke<Account[]>('accounts_list')),
  create: (username: string) => handleTauriError(invoke<Account>('accounts_create', { username })),
  update: (id: string, username?: string, skinPath?: string) =>
    handleTauriError(invoke<Account>('accounts_update', { id, username, skinPath })),
  delete: (id: string) => handleTauriError(invoke('accounts_delete', { id })),
  generateUuid: (username: string) =>
    handleTauriError(invoke<string>('accounts_generate_uuid', { username })),
};

// Versions
export const versionsApi = {
  fetchManifest: () => handleTauriError(invoke<VersionManifest>('versions_fetch_manifest')),
  list: (versionType?: string) =>
    handleTauriError(invoke<VersionEntry[]>('versions_list', { versionType })),
  download: (versionId: string) => handleTauriError(invoke('versions_download', { versionId })),
  isInstalled: (versionId: string) =>
    handleTauriError(invoke<boolean>('versions_is_installed', { versionId })),
  getVersionJson: (versionId: string) =>
    handleTauriError(invoke<VersionJson>('versions_get_version_json', { versionId })),
};

// Java
export const javaApi = {
  detect: () => handleTauriError(invoke<JavaInstallation[]>('java_detect')),
  listRuntimes: () => handleTauriError(invoke<JavaSelection[]>('java_list_runtimes')),
  resolve: (requiredMajor: number, overridePath?: string) =>
    handleTauriError(invoke<JavaSelection>('java_resolve', { requiredMajor, overridePath })),
};

// Instances
export const instancesApi = {
  list: () => handleTauriError(invoke<Instance[]>('instances_list')),
  create: (name: string, versionId: string) =>
    handleTauriError(invoke<Instance>('instances_create', { name, versionId })),
  update: (instance: Instance) =>
    handleTauriError(invoke<Instance>('instances_update', { instance })),
  delete: (id: string) => handleTauriError(invoke('instances_delete', { id })),
  clone: (id: string, newName: string) =>
    handleTauriError(invoke<Instance>('instances_clone', { id, newName })),
  openFolder: (id: string) => handleTauriError(invoke('instances_open_folder', { id })),
};

// Launch
export const launchApi = {
  start: (instanceId: string, accountId: string) =>
    handleTauriError(invoke('launch_start', { instanceId, accountId })),
  kill: (instanceId: string) => handleTauriError(invoke('launch_kill', { instanceId })),
};

// Loaders
export const loadersApi = {
  fabricVersions: () => handleTauriError(invoke<FabricVersion[]>('loaders_fabric_versions')),
  fabricLoaderVersions: () =>
    handleTauriError(invoke<FabricLoaderVersion[]>('loaders_fabric_loader_versions')),
  quiltVersions: () => handleTauriError(invoke<QuiltVersion[]>('loaders_quilt_versions')),
  quiltLoaderVersions: (minecraftVersion: string) =>
    handleTauriError(
      invoke<QuiltLoaderVersion[]>('loaders_quilt_loader_versions', { minecraftVersion }),
    ),
};
