// Versions page
import { useEffect, useState, useMemo } from 'react';
import { Header } from '@/components/layout/Header';
import { PageLayout } from '@/components/layout/Layout';
import { Button } from '@/components/ui/Button';
import { Card, CardContent } from '@/components/ui/Card';
import { Input } from '@/components/ui/Input';
import { Select } from '@/components/ui/Select';
import { useStore } from '@/store/useStore';
import { useToast } from '@/components/ui/Toaster';
import { versionsApi } from '@/services/api';
import { eventManager } from '@/services/events';
import {
  Download,
  RefreshCw,
  Filter,
  Loader2,
  FolderOpen,
} from 'lucide-react';
import { cn } from '@/lib/utils';

export function Versions() {
  const {
    versions,
    setVersions,
    versionFilter,
    setVersionFilter,
    downloads,
    selectedVersionId,
  } = useStore();
  const { toast } = useToast();
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(false);
  const [refreshing, setRefreshing] = useState(false);

  // Load versions on mount
  useEffect(() => {
    const loadVersions = async () => {
      setLoading(true);
      try {
        const manifest = await versionsApi.fetchManifest();
        setVersions(manifest.versions);
      } catch (error) {
        toast({ type: 'error', title: 'Error', message: 'Failed to load version manifest' });
      } finally {
        setLoading(false);
      }
    };

    loadVersions();

    // Listen for download progress
    eventManager.setupDownloadProgress((progress) => {
      useStore.getState().setDownloadProgress(progress);
    });

    eventManager.setupDownloadComplete((event) => {
      useStore.getState().removeDownload(event.versionId);
      if (event.success) {
        toast({ type: 'success', title: 'Download Complete', message: `Version ${event.versionId} is ready` });
      } else {
        toast({ type: 'error', title: 'Download Failed', message: event.error || 'Unknown error' });
      }
    });

    return () => eventManager.cleanup();
  }, [setVersions, toast]);

  const handleRefresh = async () => {
    setRefreshing(true);
    try {
      const manifest = await versionsApi.fetchManifest();
      setVersions(manifest.versions);
      toast({ type: 'success', title: 'Refreshed', message: 'Version list updated' });
    } catch (error) {
      toast({ type: 'error', title: 'Error', message: 'Failed to refresh versions' });
    } finally {
      setRefreshing(false);
    }
  };

  const handleDownload = async (versionId: string) => {
    try {
      await versionsApi.download(versionId);
    } catch (error) {
      toast({ type: 'error', title: 'Download Failed', message: error instanceof Error ? error.message : 'Unknown error' });
    }
  };

  const handleOpenFolder = async (_versionId: string) => {
    // Would open the version folder
    toast({ type: 'info', title: 'Opening Folder', message: 'Not implemented yet' });
  };

  // Filter versions
  const filteredVersions = useMemo(() => {
    return versions.filter((version) => {
      // Type filter
      if (versionFilter !== 'all' && version.type !== versionFilter) return false;
      // Search filter
      if (searchQuery && !version.id.toLowerCase().includes(searchQuery.toLowerCase())) return false;
      return true;
    });
  }, [versions, versionFilter, searchQuery]);

  // Sort: releases first, then snapshots, then by version (newest first)
  const sortedVersions = useMemo(() => {
    return [...filteredVersions].sort((a, b) => {
      const typeOrder = { release: 0, snapshot: 1, old_beta: 2, old_alpha: 3 };
      if (typeOrder[a.type] !== typeOrder[b.type]) {
        return typeOrder[a.type] - typeOrder[b.type];
      }
      return b.id.localeCompare(a.id, undefined, { numeric: true, sensitivity: 'base' });
    });
  }, [filteredVersions]);

  const downloadProgress = selectedVersionId ? downloads.get(selectedVersionId) : null;
  const isDownloading = downloadProgress && downloadProgress.stage !== 'complete';

  const typeLabels: Record<string, string> = {
    release: 'Release',
    snapshot: 'Snapshot',
    old_beta: 'Old Beta',
    old_alpha: 'Old Alpha',
  };

  const typeColors: Record<string, string> = {
    release: 'bg-green-500/20 text-green-400 border-green-500/30',
    snapshot: 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30',
    old_beta: 'bg-orange-500/20 text-orange-400 border-orange-500/30',
    old_alpha: 'bg-red-500/20 text-red-400 border-red-500/30',
  };

  return (
    <div className="flex flex-col h-full">
      <Header
        title="Versions"
        subtitle="Browse and download Minecraft versions"
        actions={
          <Button variant="secondary" onClick={handleRefresh} leftIcon={<RefreshCw className={cn('w-4 h-4', refreshing && 'animate-spin')} />}>
            Refresh
          </Button>
        }
      />

      <PageLayout>
        <div className="max-w-4xl mx-auto w-full space-y-6">
          {/* Filters */}
          <Card>
            <CardContent className="pt-0">
              <div className="flex flex-col sm:flex-row gap-4">
                <div className="relative flex-1">
                  <Filter className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-surface-500" />
                  <Input
                    placeholder="Search versions..."
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="pl-10"
                  />
                </div>
                <Select
                  label="Type"
                  options={[
                    { value: 'all', label: 'All Types' },
                    { value: 'release', label: 'Releases' },
                    { value: 'snapshot', label: 'Snapshots' },
                    { value: 'old_beta', label: 'Old Beta' },
                    { value: 'old_alpha', label: 'Old Alpha' },
                  ]}
                  value={versionFilter}
                  onChange={(e) => setVersionFilter(e.target.value as any)}
                  className="w-40"
                />
              </div>
            </CardContent>
          </Card>

          {/* Version List */}
          {loading ? (
            <Card>
              <CardContent className="py-12 text-center">
                <Loader2 className="w-8 h-8 animate-spin text-primary-500 mx-auto mb-4" />
                <p className="text-surface-500">Loading versions...</p>
              </CardContent>
            </Card>
          ) : sortedVersions.length === 0 ? (
            <Card variant="outlined">
              <CardContent className="py-12 text-center">
                <p className="text-surface-500">No versions match your filters</p>
              </CardContent>
            </Card>
          ) : (
            <div className="space-y-3">
              {sortedVersions.map((version) => {
                const progress = downloads.get(version.id);
                const isDownloaded = progress && progress.stage === 'complete' && progress.downloaded === progress.total;
                const isCurrentDownload = progress && progress.stage !== 'complete';

                return (
                  <Card key={version.id} hover className={cn(selectedVersionId === version.id && 'border-primary-500/50')}>
                    <CardContent className="pt-0">
                      <div className="flex items-center gap-4 p-4">
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-3 flex-wrap">
                            <h3 className="font-medium text-surface-100 truncate">{version.id}</h3>
                            <span className={cn(
                              'px-2 py-0.5 text-xs rounded-full border',
                              typeColors[version.type]
                            )}>
                              {typeLabels[version.type] || version.type}
                            </span>
                            {version.complianceLevel > 0 && (
                              <span className="px-2 py-0.5 text-xs bg-surface-800 text-surface-500 rounded-full">
                                Compliance: {version.complianceLevel}
                              </span>
                            )}
                          </div>
                          <div className="flex items-center gap-4 mt-2 text-sm text-surface-500">
                            <span>Released: {new Date(version.releaseTime).toLocaleDateString()}</span>
                            {version.sha1 && (
                              <span className="font-mono text-xs">SHA1: {version.sha1.slice(0, 12)}...</span>
                            )}
                          </div>

                          {/* Download Progress */}
                          {isCurrentDownload && (
                            <div className="mt-3 space-y-1">
                              <div className="flex items-center justify-between text-xs">
                                <span className="text-surface-400">{progress?.fileName || 'Downloading...'}</span>
                                <span className="text-primary-400 font-mono">
                                  {progress ? Math.round((progress.downloaded / progress.total) * 100) : 0}%
                                </span>
                              </div>
                              <div className="h-2 bg-surface-800 rounded-full overflow-hidden">
                                <div
                                  className="h-full bg-primary-500 transition-all duration-300"
                                  style={{ width: progress ? `${Math.round((progress.downloaded / progress.total) * 100)}%` : '0%' }}
                                />
                              </div>
                              {progress && progress.speedBps && (
                                <p className="text-xs text-surface-500">
                                  {progress.speedBps > 1024 * 1024
                                    ? `${(progress.speedBps / 1024 / 1024).toFixed(1)} MB/s`
                                    : `${(progress.speedBps / 1024).toFixed(1)} KB/s`}
                                  {progress.etaSeconds ? ` • ETA: ${Math.floor(progress.etaSeconds / 60)}m ${progress.etaSeconds % 60}s` : ''}
                                </p>
                              )}
                            </div>
                          )}
                        </div>
                        <div className="flex items-center gap-2 flex-shrink-0">
                          {isDownloaded ? (
                            <Button variant="ghost" size="sm" onClick={() => handleOpenFolder(version.id)} leftIcon={<FolderOpen className="w-4 h-4" />}>
                              Open
                            </Button>
                          ) : isDownloading ? (
                            <Button variant="ghost" size="sm" disabled>
                              <Loader2 className="w-4 h-4 animate-spin" />
                            </Button>
                          ) : (
                            <Button size="sm" onClick={() => handleDownload(version.id)} leftIcon={<Download className="w-4 h-4" />}>
                              Download
                            </Button>
                          )}
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                );
              })}
            </div>
          )}
        </div>
      </PageLayout>
    </div>
  );
}