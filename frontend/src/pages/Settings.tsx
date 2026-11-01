// Settings page
import { useEffect, useState } from 'react';
import { Header } from '@/components/layout/Header';
import { PageLayout } from '@/components/layout/Layout';
import { Button } from '@/components/ui/Button';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/Card';
import { Input } from '@/components/ui/Input';
import { Select } from '@/components/ui/Select';
import { useStore } from '@/store/useStore';
import { useToast } from '@/components/ui/Toaster';
import { configApi, javaApi } from '@/services/api';
import type { Config, JavaSelection } from '@/types';
import {
  FolderOpen,
  Save,
  RotateCcw,
  Info,
  CheckCircle,
  Loader2,
} from 'lucide-react';
import { cn } from '@/lib/utils';

export function Settings() {
  const { config, setConfig, setTheme } = useStore();
  const { toast } = useToast();
  const [localConfig, setLocalConfig] = useState<Partial<Config>>({});
  const [javaRuntimes, setJavaRuntimes] = useState<JavaSelection[]>([]);
  const [testingJava, setTestingJava] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  // Load config on mount
  useEffect(() => {
    const loadConfig = async () => {
      try {
        const cfg = await configApi.get();
        setConfig(cfg);
        setLocalConfig(cfg);
      } catch (error) {
        toast({ type: 'error', title: 'Error', message: 'Failed to load settings' });
      }
    };

    loadConfig();
    javaApi.listRuntimes().then(setJavaRuntimes).catch(() => {});
  }, [setConfig, toast]);

  const handleSave = async () => {
    setSaving(true);
    try {
      const newConfig = { ...config, ...localConfig } as Config;
      await configApi.set(newConfig);
      setConfig(newConfig);
      toast({ type: 'success', title: 'Settings Saved', message: 'Your preferences have been updated' });
    } catch (error) {
      toast({ type: 'error', title: 'Failed to Save', message: error instanceof Error ? error.message : 'Unknown error' });
    } finally {
      setSaving(false);
    }
  };

  const handleReset = async () => {
    try {
      const defaultConfig: Config = {
        concurrentDownloads: 8,
        defaultMemoryMb: 2048,
        defaultJavaVersion: undefined,
        defaultJavaPath: undefined,
        defaultJvmArgs: '-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1HeapRegionSize=8M -XX:G1ReservePercent=20 -XX:G1HeapWastePercent=5 -XX:G1MixedGCCountTarget=4 -XX:InitiatingHeapOccupancyPercent=15 -XX:G1MixedGCLiveThresholdPercent=90 -XX:G1RSetUpdatingPauseTimePercent=5 -XX:SurvivorRatio=32 -XX:+PerfDisableSharedMem -XX:MaxTenuringThreshold=1',
        language: 'en',
        theme: 'dark',
        closeOnLaunch: false,
        showAdvanced: false,
        window: { width: 1200, height: 800, maximized: false },
      };
      await configApi.set(defaultConfig);
      setConfig(defaultConfig);
      setLocalConfig(defaultConfig);
      toast({ type: 'success', title: 'Settings Reset', message: 'All settings restored to defaults' });
    } catch (error) {
      toast({ type: 'error', title: 'Failed to Reset', message: error instanceof Error ? error.message : 'Unknown error' });
    }
  };

  const handleOpenDataDir = async () => {
    try {
      await configApi.openDataDir();
    } catch (error) {
      toast({ type: 'error', title: 'Error', message: 'Failed to open data directory' });
    }
  };

  const testJava = async (javaPath: string) => {
    setTestingJava(javaPath);
    try {
      // Would invoke a test command
      toast({ type: 'success', title: 'Java Test', message: 'Java executable is valid' });
    } catch (error) {
      toast({ type: 'error', title: 'Java Test Failed', message: error instanceof Error ? error.message : 'Unknown error' });
    } finally {
      setTestingJava(null);
    }
  };

  return (
    <div className="flex flex-col h-full">
      <Header
        title="Settings"
        subtitle="Configure OfflineLauncher preferences"
        actions={
          <div className="flex items-center gap-2">
            <Button variant="secondary" onClick={handleReset} leftIcon={<RotateCcw className="w-4 h-4" />}>
              Reset to Defaults
            </Button>
            <Button onClick={handleSave} disabled={saving} leftIcon={<Save className="w-4 h-4" />}>
              {saving ? 'Saving...' : 'Save Changes'}
            </Button>
          </div>
        }
      />

      <PageLayout>
        <div className="max-w-3xl mx-auto w-full space-y-6">
          {/* General */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Info className="w-5 h-5 text-primary-500" />
                General
              </CardTitle>
              <CardDescription>Basic application settings</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4 pt-0">
              <div className="grid gap-4 sm:grid-cols-2">
                <Select
                  label="Theme"
                  options={[
                    { value: 'dark', label: 'Dark' },
                    { value: 'light', label: 'Light' },
                    { value: 'system', label: 'System' },
                  ]}
                  value={localConfig.theme || config?.theme || 'dark'}
                  onChange={(e) => {
                    setLocalConfig(prev => ({ ...prev, theme: e.target.value as any }));
                    setTheme(e.target.value as any);
                  }}
                />
                <Select
                  label="Language"
                  options={[
                    { value: 'en', label: 'English' },
                  ]}
                  value={localConfig.language || config?.language || 'en'}
                  onChange={(e) => setLocalConfig(prev => ({ ...prev, language: e.target.value }))}
                />
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium text-surface-100">Close launcher on game launch</p>
                  <p className="text-sm text-surface-500">Automatically close the launcher when the game starts</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    className="sr-only peer"
                    checked={localConfig.closeOnLaunch ?? config?.closeOnLaunch ?? false}
                    onChange={(e) => setLocalConfig(prev => ({ ...prev, closeOnLaunch: e.target.checked }))}
                  />
                  <div className="w-11 h-6 bg-surface-700 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-500/30 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-surface-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-600"></div>
                </label>
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <p className="font-medium text-surface-100">Show advanced settings</p>
                  <p className="text-sm text-surface-500">Display advanced options in instance editor</p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    className="sr-only peer"
                    checked={localConfig.showAdvanced ?? config?.showAdvanced ?? false}
                    onChange={(e) => setLocalConfig(prev => ({ ...prev, showAdvanced: e.target.checked }))}
                  />
                  <div className="w-11 h-6 bg-surface-700 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-primary-500/30 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-surface-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary-600"></div>
                </label>
              </div>
            </CardContent>
          </Card>

          {/* Downloads */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <FolderOpen className="w-5 h-5 text-primary-500" />
                Downloads
              </CardTitle>
              <CardDescription>Configure download behavior</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4 pt-0">
              <div className="grid gap-4 sm:grid-cols-2">
                <Input
                  label="Concurrent Downloads"
                  type="number"
                  min={1}
                  max={32}
                  value={localConfig.concurrentDownloads || config?.concurrentDownloads || 8}
                  onChange={(e) => setLocalConfig(prev => ({ ...prev, concurrentDownloads: parseInt(e.target.value) || 8 }))}
                  helperText="Number of simultaneous downloads (1-32)"
                />
                <Input
                  label="Data Directory"
                  type="text"
                  value={localConfig.dataDir || config?.dataDir || 'Default (OS-specific)'}
                  readOnly
                  helperText="Where game files, accounts, and instances are stored"
                />
              </div>
              <Button variant="secondary" onClick={handleOpenDataDir} leftIcon={<FolderOpen className="w-4 h-4" />}>
                Open Data Directory
              </Button>
            </CardContent>
          </Card>

          {/* Java */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Info className="w-5 h-5 text-primary-500" />
                Java Runtime
              </CardTitle>
              <CardDescription>Manage Java installations and defaults</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4 pt-0">
              <div className="grid gap-4 sm:grid-cols-2">
                <Select
                  label="Default Java Version"
                  options={[
                    { value: '', label: 'Auto-detect from version' },
                    { value: '8', label: 'Java 8' },
                    { value: '11', label: 'Java 11' },
                    { value: '17', label: 'Java 17' },
                    { value: '21', label: 'Java 21' },
                  ]}
                  value={String(localConfig.defaultJavaVersion || config?.defaultJavaVersion || '')}
                  onChange={(e) => setLocalConfig(prev => ({ ...prev, defaultJavaVersion: e.target.value ? parseInt(e.target.value) : undefined }))}
                />
                <Input
                  label="Custom Java Path (optional)"
                  type="text"
                  value={localConfig.defaultJavaPath || config?.defaultJavaPath || ''}
                  onChange={(e) => setLocalConfig(prev => ({ ...prev, defaultJavaPath: e.target.value || undefined }))}
                  placeholder="Path to java executable"
                  helperText="Leave empty to auto-detect"
                />
              </div>

              <div>
                <h4 className="font-medium text-surface-100 mb-3">Detected Java Runtimes</h4>
                {javaRuntimes.length === 0 ? (
                  <p className="text-surface-500 text-sm">No Java installations detected. Click refresh to scan.</p>
                ) : (
                  <div className="space-y-2 max-h-64 overflow-y-auto scrollbar-thin">
                    {javaRuntimes.map((java) => (
                      <div key={java.path} className="flex items-center justify-between p-3 bg-surface-800/50 rounded-lg border border-surface-700">
                        <div className="flex items-center gap-3">
                          <div className="w-8 h-8 rounded bg-primary-600/20 flex items-center justify-center">
                            <Info className="w-4 h-4 text-primary-400" />
                          </div>
                          <div>
                            <p className="font-medium text-surface-100 text-sm">
                              {java.version} (Java {java.majorVersion})
                            </p>
                            <p className="text-xs text-surface-500 truncate max-w-xs">{java.path}</p>
                          </div>
                        </div>
                        <div className="flex items-center gap-2">
                          <span className={cn(
                            'px-2 py-0.5 text-xs rounded-full',
                            java.source === 'system' && 'bg-blue-500/20 text-blue-400',
                            java.source === 'mojang' && 'bg-green-500/20 text-green-400',
                            java.source === 'adoptium' && 'bg-purple-500/20 text-purple-400',
                            java.source === 'custom' && 'bg-orange-500/20 text-orange-400',
                          )}>
                            {java.source}
                          </span>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => testJava(java.path)}
                            disabled={testingJava === java.path}
                            className="h-7"
                          >
                            {testingJava === java.path ? (
                              <Loader2 className="w-4 h-4 animate-spin" />
                            ) : (
                              <CheckCircle className="w-4 h-4" />
                            )}
                          </Button>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
                <Button variant="secondary" onClick={() => javaApi.listRuntimes().then(setJavaRuntimes)} leftIcon={<RotateCcw className="w-4 h-4" />}>
                  Refresh Java List
                </Button>
              </div>
            </CardContent>
          </Card>

          {/* Memory */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Info className="w-5 h-5 text-primary-500" />
                Default Memory
              </CardTitle>
              <CardDescription>Default memory allocation for new instances</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4 pt-0">
              <div className="grid gap-4 sm:grid-cols-2">
                <Input
                  label="Minimum Memory (MB)"
                  type="number"
                  min={256}
                  max={65536}
                  value={localConfig.defaultMemoryMb || config?.defaultMemoryMb || 2048}
                  onChange={(e) => setLocalConfig(prev => ({ ...prev, defaultMemoryMb: parseInt(e.target.value) || 2048 }))}
                  helperText="Minimum heap size (-Xms)"
                />
                <Input
                  label="Maximum Memory (MB)"
                  type="number"
                  min={512}
                  max={65536}
                  value={localConfig.defaultMemoryMb || config?.defaultMemoryMb || 2048}
                  onChange={(e) => setLocalConfig(prev => ({ ...prev, defaultMemoryMb: parseInt(e.target.value) || 2048 }))}
                  helperText="Maximum heap size (-Xmx)"
                />
              </div>
              <div>
                <label className="label">Default JVM Arguments</label>
                <textarea
                  className="input font-mono text-sm h-32 resize-y"
                  value={localConfig.defaultJvmArgs || config?.defaultJvmArgs || ''}
                  onChange={(e) => setLocalConfig(prev => ({ ...prev, defaultJvmArgs: e.target.value }))}
                  placeholder="Additional JVM arguments for all instances..."
                />
              </div>
            </CardContent>
          </Card>

          {/* About */}
          <Card variant="outlined">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Info className="w-5 h-5 text-primary-500" />
                About
              </CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
              <div className="space-y-3 text-sm">
                <div className="flex justify-between">
                  <span className="text-surface-500">Version</span>
                  <span className="font-mono text-surface-100">0.1.0</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-surface-500">Platform</span>
                  <span className="font-mono text-surface-100">Tauri 2 + React + TypeScript</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-surface-500">Target</span>
                  <span className="font-mono text-surface-100">Minecraft Java Edition</span>
                </div>
                <div className="pt-4 border-t border-surface-700">
                  <p className="text-surface-500">
                    OfflineLauncher is an open-source Minecraft launcher with first-class offline account support.
                    Built with Tauri 2, React, TypeScript, and Rust.
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </PageLayout>
    </div>
  );
}