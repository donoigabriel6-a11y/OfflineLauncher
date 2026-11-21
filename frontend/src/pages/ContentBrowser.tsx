import { useCallback, useEffect, useMemo, useState } from 'react';
import { Header } from '@/components/layout/Header';
import { PageLayout } from '@/components/layout/Layout';
import { Button } from '@/components/ui/Button';
import { Card, CardContent } from '@/components/ui/Card';
import { Input } from '@/components/ui/Input';
import { Select } from '@/components/ui/Select';
import { useStore } from '@/store/useStore';
import { useToast } from '@/components/ui/Toaster';
import { contentApi } from '@/services/api';
import type { ContentCategory, ContentProject, ContentProvider, ContentVersion, InstalledContent } from '@/types';
import { AlertTriangle, Download, Loader2, PackageCheck, Search, Trash2 } from 'lucide-react';

const categories: { value: ContentCategory; label: string }[] = [
  { value: 'mods', label: 'Mods' },
  { value: 'modpacks', label: 'Modpacks' },
  { value: 'resourcepacks', label: 'Resource Packs' },
  { value: 'shaders', label: 'Shaders' },
];

export function ContentBrowser() {
  const { instances, setInstances } = useStore();
  const { toast } = useToast();
  const [provider, setProvider] = useState<ContentProvider>('modrinth');
  const [category, setCategory] = useState<ContentCategory>('mods');
  const [instanceId, setInstanceId] = useState('');
  const [query, setQuery] = useState('');
  const [projects, setProjects] = useState<ContentProject[]>([]);
  const [versions, setVersions] = useState<ContentVersion[]>([]);
  const [installed, setInstalled] = useState<InstalledContent[]>([]);
  const [selectedProject, setSelectedProject] = useState<ContentProject | null>(null);
  const [selectedVersionId, setSelectedVersionId] = useState('');
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState(false);
  const [searchError, setSearchError] = useState('');
  const selectedInstance = instances.find((instance) => instance.id === instanceId);

  useEffect(() => {
    if (instances.length === 0) {
      contentApi.listInstances().then(setInstances).catch(() => {});
    }
  }, [instances.length, setInstances]);

  const refreshInstalled = useCallback(async () => {
    if (!instanceId) {
      setInstalled([]);
      return;
    }
    try {
      setInstalled(await contentApi.listInstalled(instanceId));
    } catch {
      setInstalled([]);
    }
  }, [instanceId]);

  useEffect(() => {
    refreshInstalled();
  }, [refreshInstalled]);

  const search = useCallback(async () => {
    setLoading(true);
    setSearchError('');
    try {
      const result = await contentApi.search({
        provider,
        contentType: category,
        query: query.trim(),
        minecraftVersion: selectedInstance?.versionId ?? '',
        loader: selectedInstance?.loader === 'none' ? undefined : selectedInstance?.loader,
      });
      setProjects(result.projects);
    } catch (error) {
      setProjects([]);
      const message = error instanceof Error ? error.message : 'Search failed';
      setSearchError(message);
      toast({ type: 'error', title: 'Content Search Failed', message });
    } finally {
      setLoading(false);
    }
  }, [category, provider, query, selectedInstance, toast]);

  useEffect(() => {
    const timer = window.setTimeout(() => void search(), 250);
    return () => window.clearTimeout(timer);
  }, [search]);

  const compatibleVersions = useMemo(
    () => versions.filter((version) => !version.gameVersions?.length || version.gameVersions.includes(selectedInstance?.versionId ?? '')),
    [selectedInstance?.versionId, versions]
  );

  const openProject = async (project: ContentProject) => {
    setSelectedProject(project);
    setVersions([]);
    setSelectedVersionId('');
    if (!instanceId) return;
    setBusy(true);
    try {
      const result = await contentApi.versions({
        provider,
        projectId: project.id,
        minecraftVersion: selectedInstance?.versionId ?? '',
        loader: selectedInstance?.loader === 'none' ? undefined : selectedInstance?.loader,
      });
      setVersions(result);
      setSelectedVersionId(result[0]?.id ?? '');
    } catch (error) {
      toast({ type: 'error', title: 'Could Not Load Versions', message: error instanceof Error ? error.message : 'Unknown error' });
    } finally {
      setBusy(false);
    }
  };

  const install = async () => {
    if (!selectedProject || !selectedVersionId || !instanceId) return;
    setBusy(true);
    try {
      await contentApi.install({ instanceId, provider, contentType: category, projectId: selectedProject.id, versionId: selectedVersionId, minecraftVersion: selectedInstance?.versionId ?? '' });
      toast({ type: 'success', title: 'Installed', message: `${selectedProject.title} was added to ${selectedInstance?.name ?? 'the instance'}.` });
      setSelectedProject(null);
      await refreshInstalled();
    } catch (error) {
      toast({ type: 'error', title: 'Installation Failed', message: error instanceof Error ? error.message : 'Unknown error' });
    } finally {
      setBusy(false);
    }
  };

  const uninstall = async (item: InstalledContent) => {
    setBusy(true);
    try {
      await contentApi.uninstall(instanceId, item.installId);
      await refreshInstalled();
      toast({ type: 'success', title: 'Removed', message: `${item.title} was uninstalled.` });
    } catch (error) {
      toast({ type: 'error', title: 'Uninstall Failed', message: error instanceof Error ? error.message : 'Unknown error' });
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex h-full flex-col">
      <Header title="Content Browser" subtitle="Find and manage mods, modpacks, resource packs, and shaders" />
      <PageLayout className="flex flex-1 flex-col">
        <div className="mx-auto flex w-full max-w-6xl flex-1 flex-col gap-5">
          <Card>
            <div className="grid gap-4 md:grid-cols-3">
              <Select label="Install to instance" placeholder="Choose an instance" options={instances.map((item) => ({ value: item.id, label: `${item.name} — ${item.versionId} (${item.loader})` }))} value={instanceId} onChange={(event) => setInstanceId(event.target.value)} />
              <Select label="Provider" options={[{ value: 'modrinth', label: 'Modrinth' }, { value: 'curseforge', label: 'CurseForge' }]} value={provider} onChange={(event) => setProvider(event.target.value as ContentProvider)} />
              <Select label="Category" options={categories} value={category} onChange={(event) => setCategory(event.target.value as ContentCategory)} />
            </div>
            {!instanceId && <p className="mt-3 text-sm text-amber-400">Select an instance to see compatible results and install content.</p>}
          </Card>

          <div className="grid min-h-0 flex-1 gap-5 lg:grid-cols-[minmax(0,1fr)_minmax(300px,0.8fr)]">
            <section className="flex min-h-0 flex-col gap-3">
              <div className="flex gap-2">
                <Input aria-label="Search content" placeholder={`Search ${categories.find((item) => item.value === category)?.label.toLowerCase()}...`} value={query} onChange={(event) => setQuery(event.target.value)} onKeyDown={(event) => event.key === 'Enter' && void search()} />
                <Button onClick={() => void search()} aria-label="Search" leftIcon={<Search className="h-4 w-4" />}>Search</Button>
              </div>
              <div className="scrollbar-thin min-h-0 flex-1 space-y-3 overflow-y-auto pr-1">
                {loading ? <Card><CardContent className="py-10 text-center text-surface-400"><Loader2 className="mx-auto mb-2 h-6 w-6 animate-spin" />Searching…</CardContent></Card> : searchError ? <Card><CardContent className="py-8 text-center text-red-400">{searchError}</CardContent></Card> : projects.length === 0 ? <Card><CardContent className="py-8 text-center text-surface-500">No projects found. Try another search or provider.</CardContent></Card> : projects.map((project) => (
                  <button key={project.id} onClick={() => void openProject(project)} className="w-full text-left">
                    <Card hover className={selectedProject?.id === project.id ? 'border-primary-500/60' : ''}>
                      <div className="flex gap-4">
                        {project.iconUrl ? <img src={project.iconUrl} alt="" className="h-14 w-14 rounded-lg object-cover" /> : <div className="flex h-14 w-14 items-center justify-center rounded-lg bg-surface-800"><PackageCheck className="h-6 w-6 text-surface-500" /></div>}
                        <div className="min-w-0 flex-1"><div className="flex justify-between gap-2"><h3 className="truncate font-semibold text-surface-100">{project.title}</h3><span className="shrink-0 text-xs text-surface-500">{project.downloads.toLocaleString()} downloads</span></div><p className="mt-1 line-clamp-2 text-sm text-surface-400">{project.description}</p><p className="mt-2 text-xs text-surface-500">by {project.author}</p></div>
                      </div>
                    </Card>
                  </button>
                ))}
              </div>
            </section>

            <section className="flex min-h-0 flex-col gap-4">
              <Card className="flex-1">
                <h2 className="mb-3 text-lg font-semibold text-surface-100">Project details</h2>
                {!selectedProject ? <p className="text-sm text-surface-500">Choose a search result to review versions and compatibility.</p> : <div className="space-y-4">
                  <div className="flex items-center gap-3">{selectedProject.iconUrl && <img src={selectedProject.iconUrl} alt="" className="h-12 w-12 rounded-lg object-cover" />}<div><h3 className="font-semibold text-surface-100">{selectedProject.title}</h3><p className="text-sm text-surface-500">by {selectedProject.author}</p></div></div>
                  <p className="text-sm text-surface-300">{selectedProject.description}</p>
                  {selectedInstance && selectedProject.gameVersions?.length && !selectedProject.gameVersions.includes(selectedInstance.versionId) && <div className="flex gap-2 rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-amber-300"><AlertTriangle className="h-5 w-5 shrink-0" /><span>This project does not list Minecraft {selectedInstance.versionId} as compatible. Confirm compatibility before installing.</span></div>}
                  {!instanceId ? <p className="text-sm text-amber-400">Select an instance before loading versions.</p> : busy && versions.length === 0 ? <p className="flex items-center gap-2 text-sm text-surface-400"><Loader2 className="h-4 w-4 animate-spin" />Loading versions…</p> : versions.length === 0 ? <p className="text-sm text-surface-500">No versions available for this project.</p> : <>
                    {compatibleVersions.length === 0 && <p className="flex gap-2 rounded-lg border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-amber-300"><AlertTriangle className="h-5 w-5 shrink-0" />No listed versions match this instance. Installing a different version may cause issues.</p>}
                    <Select label="Version" options={versions.map((version) => ({ value: version.id, label: `${version.name} (${version.versionNumber})${version.gameVersions?.includes(selectedInstance?.versionId ?? '') ? ' · compatible' : ''}` }))} value={selectedVersionId} onChange={(event) => setSelectedVersionId(event.target.value)} />
                    <Button className="w-full" onClick={() => void install()} disabled={busy || !selectedVersionId} loading={busy} leftIcon={<Download className="h-4 w-4" />}>Install to {selectedInstance?.name ?? 'instance'}</Button>
                  </>}
                </div>}
              </Card>
              <Card>
                <h2 className="mb-3 text-lg font-semibold text-surface-100">Installed content {instanceId && <span className="text-sm font-normal text-surface-500">· {installed.length}</span>}</h2>
                {!instanceId ? <p className="text-sm text-surface-500">Choose an instance to see installed content.</p> : installed.length === 0 ? <p className="text-sm text-surface-500">No marketplace content installed for this instance.</p> : <div className="max-h-52 space-y-2 overflow-y-auto">{installed.map((item) => <div key={item.installId} className="flex items-center justify-between gap-3 rounded-lg bg-surface-800/70 p-3"><div className="min-w-0"><p className="truncate text-sm font-medium text-surface-100">{item.title}</p><p className="text-xs text-surface-500">{item.contentType} · {item.versionNumber}</p></div><Button size="sm" variant="ghost" disabled={busy} onClick={() => void uninstall(item)} aria-label={`Uninstall ${item.title}`}><Trash2 className="h-4 w-4 text-red-400" /></Button></div>)}</div>}
              </Card>
            </section>
          </div>
        </div>
      </PageLayout>
    </div>
  );
}
