// Tauri event listeners
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import type { DownloadProgress, LaunchOutput, LaunchStatus } from '@/types';

type DownloadProgressCallback = (progress: DownloadProgress) => void;
type DownloadCompleteCallback = (event: {
  versionId: string;
  success: boolean;
  error?: string;
}) => void;
type LaunchOutputCallback = (output: LaunchOutput) => void;
type LaunchStatusCallback = (status: LaunchStatus) => void;
type AccountsChangedCallback = (event: { action: string; account: Account }) => void;
type InstancesChangedCallback = (event: { action: string; instance: Instance }) => void;

interface Account {
  id: string;
  username: string;
  uuid: string;
  accountType: string;
}

interface Instance {
  id: string;
  name: string;
  versionId: string;
  loader?: string;
}

class EventManager {
  private unlistenFns: UnlistenFn[] = [];

  async setupDownloadProgress(callback: DownloadProgressCallback): Promise<void> {
    const unlisten = await listen('download_progress', (event) => {
      callback(event.payload as DownloadProgress);
    });
    this.unlistenFns.push(unlisten);
  }

  async setupDownloadComplete(callback: DownloadCompleteCallback): Promise<void> {
    const unlisten = await listen('download_complete', (event) => {
      callback(event.payload as { versionId: string; success: boolean; error?: string });
    });
    this.unlistenFns.push(unlisten);
  }

  async setupLaunchOutput(callback: LaunchOutputCallback): Promise<void> {
    const unlisten = await listen('launch_output', (event) => {
      callback(event.payload as LaunchOutput);
    });
    this.unlistenFns.push(unlisten);
  }

  async setupLaunchStatus(callback: LaunchStatusCallback): Promise<void> {
    const unlisten = await listen('launch_status', (event) => {
      callback(event.payload as LaunchStatus);
    });
    this.unlistenFns.push(unlisten);
  }

  async setupAccountsChanged(callback: AccountsChangedCallback): Promise<void> {
    const unlisten = await listen('accounts_changed', (event) => {
      callback(event.payload as { action: string; account: Account });
    });
    this.unlistenFns.push(unlisten);
  }

  async setupInstancesChanged(callback: InstancesChangedCallback): Promise<void> {
    const unlisten = await listen('instances_changed', (event) => {
      callback(event.payload as { action: string; instance: Instance });
    });
    this.unlistenFns.push(unlisten);
  }

  cleanup(): void {
    this.unlistenFns.forEach((unlisten) => unlisten());
    this.unlistenFns = [];
  }
}

export const eventManager = new EventManager();
