// Zustand store for global state
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import type {
  Account,
  Instance,
  VersionEntry,
  Config,
  JavaSelection,
  DownloadProgress,
  LaunchOutput,
  LaunchStatus,
} from '@/types';

interface AppState {
  // Theme
  theme: 'dark' | 'light' | 'system';
  setTheme: (theme: 'dark' | 'light' | 'system') => void;

  // Sidebar
  sidebarOpen: boolean;
  toggleSidebar: () => void;
  setSidebarOpen: (open: boolean) => void;

  // Current page
  currentPage: string;
  setCurrentPage: (page: string) => void;

  // Accounts
  accounts: Account[];
  setAccounts: (accounts: Account[]) => void;
  addAccount: (account: Account) => void;
  updateAccount: (account: Account) => void;
  removeAccount: (id: string) => void;
  selectedAccountId: string | null;
  setSelectedAccountId: (id: string | null) => void;

  // Instances
  instances: Instance[];
  setInstances: (instances: Instance[]) => void;
  addInstance: (instance: Instance) => void;
  updateInstance: (instance: Instance) => void;
  removeInstance: (id: string) => void;
  selectedInstanceId: string | null;
  setSelectedInstanceId: (id: string | null) => void;

  // Versions
  versions: VersionEntry[];
  setVersions: (versions: VersionEntry[]) => void;
  versionFilter: 'all' | 'release' | 'snapshot' | 'old_beta' | 'old_alpha';
  setVersionFilter: (filter: VersionEntry['type']) => void;
  selectedVersionId: string | null;
  setSelectedVersionId: (id: string | null) => void;

  // Downloads
  downloads: Map<string, DownloadProgress>;
  setDownloadProgress: (progress: DownloadProgress) => void;
  removeDownload: (versionId: string) => void;

  // Launch
  launchOutputs: Map<string, LaunchOutput[]>;
  addLaunchOutput: (instanceId: string, output: LaunchOutput) => void;
  clearLaunchOutputs: (instanceId: string) => void;
  launchStatus: Map<string, LaunchStatus>;
  setLaunchStatus: (status: LaunchStatus) => void;

  // Java
  javaInstallations: JavaSelection[];
  setJavaInstallations: (installations: JavaSelection[]) => void;
  selectedJavaId: string | null;
  setSelectedJavaId: (id: string | null) => void;

  // Config
  config: Config | null;
  setConfig: (config: Config) => void;

  // UI state
  isLoading: boolean;
  setIsLoading: (loading: boolean) => void;
  error: string | null;
  setError: (error: string | null) => void;
  success: string | null;
  setSuccess: (success: string | null) => void;
}

export const useStore = create<AppState>()(
  persist(
    (set, _get) => ({
      // Theme
      theme: 'dark',
      setTheme: (theme) => set({ theme }),

      // Sidebar
      sidebarOpen: true,
      toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),
      setSidebarOpen: (open) => set({ sidebarOpen: open }),

      // Current page
      currentPage: 'home',
      setCurrentPage: (page) => set({ currentPage: page }),

      // Accounts
      accounts: [],
      setAccounts: (accounts) => set({ accounts }),
      addAccount: (account) => set((state) => ({ accounts: [...state.accounts, account] })),
      updateAccount: (account) =>
        set((state) => ({
          accounts: state.accounts.map((a) => (a.id === account.id ? account : a)),
        })),
      removeAccount: (id) =>
        set((state) => ({
          accounts: state.accounts.filter((a) => a.id !== id),
        })),
      selectedAccountId: null,
      setSelectedAccountId: (id) => set({ selectedAccountId: id }),

      // Instances
      instances: [],
      setInstances: (instances) => set({ instances }),
      addInstance: (instance) => set((state) => ({ instances: [...state.instances, instance] })),
      updateInstance: (instance) =>
        set((state) => ({
          instances: state.instances.map((i) => (i.id === instance.id ? instance : i)),
        })),
      removeInstance: (id) =>
        set((state) => ({
          instances: state.instances.filter((i) => i.id !== id),
        })),
      selectedInstanceId: null,
      setSelectedInstanceId: (id) => set({ selectedInstanceId: id }),

      // Versions
      versions: [],
      setVersions: (versions) => set({ versions }),
      versionFilter: 'all',
      setVersionFilter: (filter) => set({ versionFilter: filter }),
      selectedVersionId: null,
      setSelectedVersionId: (id) => set({ selectedVersionId: id }),

      // Downloads
      downloads: new Map(),
      setDownloadProgress: (progress) =>
        set((state) => {
          const newDownloads = new Map(state.downloads);
          newDownloads.set(progress.versionId, progress);
          return { downloads: newDownloads };
        }),
      removeDownload: (versionId) =>
        set((state) => {
          const newDownloads = new Map(state.downloads);
          newDownloads.delete(versionId);
          return { downloads: newDownloads };
        }),

      // Launch
      launchOutputs: new Map(),
      addLaunchOutput: (instanceId, output) =>
        set((state) => {
          const newOutputs = new Map(state.launchOutputs);
          const existing = newOutputs.get(instanceId) || [];
          newOutputs.set(instanceId, [...existing, output]);
          return { launchOutputs: newOutputs };
        }),
      clearLaunchOutputs: (instanceId) =>
        set((state) => {
          const newOutputs = new Map(state.launchOutputs);
          newOutputs.delete(instanceId);
          return { launchOutputs: newOutputs };
        }),
      launchStatus: new Map(),
      setLaunchStatus: (status) =>
        set((state) => {
          const newStatus = new Map(state.launchStatus);
          newStatus.set(status.instanceId, status);
          return { launchStatus: newStatus };
        }),

      // Java
      javaInstallations: [],
      setJavaInstallations: (installations) => set({ javaInstallations: installations }),
      selectedJavaId: null,
      setSelectedJavaId: (id) => set({ selectedJavaId: id }),

      // Config
      config: null,
      setConfig: (config) => set({ config }),

      // UI state
      isLoading: false,
      setIsLoading: (loading) => set({ isLoading: loading }),
      error: null,
      setError: (error) => set({ error }),
      success: null,
      setSuccess: (success) => set({ success }),
    }),
    {
      name: 'offline-launcher-store',
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        theme: state.theme,
        sidebarOpen: state.sidebarOpen,
        versionFilter: state.versionFilter,
        selectedAccountId: state.selectedAccountId,
        selectedInstanceId: state.selectedInstanceId,
        selectedVersionId: state.selectedVersionId,
      }),
    }
  )
);
