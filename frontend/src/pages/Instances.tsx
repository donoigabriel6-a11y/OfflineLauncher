// Instances page
import { useEffect, useState } from 'react';
import { Header } from '@/components/layout/Header';
import { PageLayout } from '@/components/layout/Layout';
import { Button } from '@/components/ui/Button';
import { Card, CardContent } from '@/components/ui/Card';
import { Modal } from '@/components/ui/Modal';
import { Input } from '@/components/ui/Input';
import { Select } from '@/components/ui/Select';
import { useStore } from '@/store/useStore';
import { useToast } from '@/components/ui/Toaster';
import { instancesApi, versionsApi } from '@/services/api';
import { eventManager } from '@/services/events';
import type { Instance, ModLoader } from '@/types';
import { Plus, Trash2, FolderOpen, Settings, Loader2 } from 'lucide-react';
import { cn } from '@/lib/utils';

export function Instances() {
  const {
    instances,
    setInstances,
    versions,
    selectedInstanceId,
    setSelectedInstanceId,
    isLoading,
    setIsLoading,
  } = useStore();
  const { toast } = useToast();
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);
  const [showCloneModal, setShowCloneModal] = useState(false);
  const [newInstanceName, setNewInstanceName] = useState('');
  const [newInstanceVersion, setNewInstanceVersion] = useState('');
  const [editingInstance, setEditingInstance] = useState<Instance | null>(null);
  const [editName, setEditName] = useState('');
  const [editVersion, setEditVersion] = useState('');
  const [editLoader, setEditLoader] = useState<ModLoader>('none');
  const [editLoaderVersion, setEditLoaderVersion] = useState('');
  const [editMemoryMin, setEditMemoryMin] = useState(512);
  const [editMemoryMax, setEditMemoryMax] = useState(2048);
  const [editJvmArgs, setEditJvmArgs] = useState('');
  const [editGameArgs, setEditGameArgs] = useState('');
  const [deletingInstanceId, setDeletingInstanceId] = useState<string | null>(null);
  const [cloningInstance, setCloningInstance] = useState<Instance | null>(null);
  const [cloneName, setCloneName] = useState('');

  // Load instances and versions on mount
  useEffect(() => {
    const loadData = async () => {
      try {
        setIsLoading(true);
        const [instancesData, versionsData] = await Promise.all([
          instancesApi.list(),
          versionsApi.list(),
        ]);
        useStore.getState().setInstances(instancesData);
        useStore.getState().setVersions(versionsData);
      } catch (error) {
        toast({ type: 'error', title: 'Error', message: 'Failed to load data' });
      } finally {
        setIsLoading(false);
      }
    };

    loadData();

    eventManager.setupInstancesChanged((event) => {
      if (
        event.action === 'created' ||
        event.action === 'updated' ||
        event.action === 'deleted' ||
        event.action === 'cloned'
      ) {
        loadData();
      }
    });

    return () => eventManager.cleanup();
  }, [setInstances, setIsLoading, toast]);

  const handleCreate = async () => {
    if (!newInstanceName.trim() || !newInstanceVersion) {
      toast({ type: 'error', title: 'Error', message: 'Please enter a name and select a version' });
      return;
    }

    try {
      const instance = await instancesApi.create(newInstanceName.trim(), newInstanceVersion);
      setShowCreateModal(false);
      setNewInstanceName('');
      setNewInstanceVersion('');
      setSelectedInstanceId(instance.id);
      toast({ type: 'success', title: 'Instance Created', message: `${instance.name} is ready` });
    } catch (error) {
      toast({
        type: 'error',
        title: 'Failed to Create',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  };

  const handleUpdate = async () => {
    if (!editingInstance || !editName.trim()) {
      toast({ type: 'error', title: 'Error', message: 'Please enter a name' });
      return;
    }

    try {
      const updatedInstance = {
        ...editingInstance,
        name: editName.trim(),
        versionId: editVersion,
        loader: editLoader,
        loaderVersion: editLoaderVersion || undefined,
        memory: { minMb: editMemoryMin, maxMb: editMemoryMax },
        jvmArgs: editJvmArgs,
        gameArgs: editGameArgs,
      };
      await instancesApi.update(updatedInstance);
      setShowEditModal(false);
      setEditingInstance(null);
      toast({ type: 'success', title: 'Instance Updated', message: 'Changes saved' });
    } catch (error) {
      toast({
        type: 'error',
        title: 'Failed to Update',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  };

  const handleDelete = async () => {
    if (!deletingInstanceId) return;

    try {
      await instancesApi.delete(deletingInstanceId);
      setShowDeleteModal(false);
      setDeletingInstanceId(null);
      if (selectedInstanceId === deletingInstanceId) {
        setSelectedInstanceId(null);
      }
      toast({ type: 'success', title: 'Instance Deleted', message: 'Instance removed' });
    } catch (error) {
      toast({
        type: 'error',
        title: 'Failed to Delete',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  };

  const handleClone = async () => {
    if (!cloningInstance || !cloneName.trim()) {
      toast({ type: 'error', title: 'Error', message: 'Please enter a name for the clone' });
      return;
    }

    try {
      const cloned = await instancesApi.clone(cloningInstance.id, cloneName.trim());
      setShowCloneModal(false);
      setCloningInstance(null);
      setCloneName('');
      setSelectedInstanceId(cloned.id);
      toast({ type: 'success', title: 'Instance Cloned', message: `${cloned.name} created` });
    } catch (error) {
      toast({
        type: 'error',
        title: 'Failed to Clone',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  };

  const handleOpenFolder = async (instanceId: string) => {
    try {
      await instancesApi.openFolder(instanceId);
    } catch (error) {
      toast({ type: 'error', title: 'Error', message: 'Failed to open folder' });
    }
  };

  const openEditModal = (instance: Instance) => {
    setEditingInstance(instance);
    setEditName(instance.name);
    setEditVersion(instance.versionId);
    setEditLoader(instance.loader);
    setEditLoaderVersion(instance.loaderVersion || '');
    setEditMemoryMin(instance.memory.minMb);
    setEditMemoryMax(instance.memory.maxMb);
    setEditJvmArgs(instance.jvmArgs);
    setEditGameArgs(instance.gameArgs);
    setShowEditModal(true);
  };

  return (
    <div className="flex h-full flex-col">
      <Header
        title="Instances"
        subtitle="Manage your Minecraft instances (profiles)"
        actions={
          <Button onClick={() => setShowCreateModal(true)} leftIcon={<Plus className="h-4 w-4" />}>
            New Instance
          </Button>
        }
      />

      <PageLayout>
        <div className="mx-auto w-full max-w-5xl space-y-4">
          {isLoading ? (
            <Card>
              <CardContent className="py-12 text-center">
                <Loader2 className="mx-auto mb-4 h-8 w-8 animate-spin text-primary-500" />
                <p className="text-surface-500">Loading instances...</p>
              </CardContent>
            </Card>
          ) : instances.length === 0 ? (
            <Card variant="outlined">
              <CardContent className="py-12 text-center">
                <p className="mb-4 text-surface-500">No instances yet</p>
                <Button
                  onClick={() => setShowCreateModal(true)}
                  leftIcon={<Plus className="h-4 w-4" />}
                >
                  Create Your First Instance
                </Button>
              </CardContent>
            </Card>
          ) : (
            <div
              className="grid gap-4"
              style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(320px, 1fr))' }}
            >
              {instances.map((instance) => (
                <Card
                  key={instance.id}
                  hover
                  className={cn(selectedInstanceId === instance.id && 'border-primary-500/50')}
                >
                  <CardContent className="pt-0">
                    <div className="p-4">
                      <div className="mb-3 flex items-start justify-between">
                        <div className="min-w-0 flex-1">
                          <h3 className="truncate pr-4 font-medium text-surface-100">
                            {instance.name}
                          </h3>
                          <p className="mt-1 truncate text-sm text-surface-500">
                            {instance.versionId}
                          </p>
                        </div>
                        <div className="flex items-center gap-1">
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => handleOpenFolder(instance.id)}
                            className="h-8 w-8"
                            title="Open folder"
                          >
                            <FolderOpen className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => {
                              setCloningInstance(instance);
                              setCloneName(`${instance.name} (copy)`);
                              setShowCloneModal(true);
                            }}
                            className="h-8 w-8"
                            title="Clone"
                          >
                            <Settings className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => openEditModal(instance)}
                            className="h-8 w-8"
                            title="Edit"
                          >
                            <Settings className="h-4 w-4" />
                          </Button>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => {
                              setDeletingInstanceId(instance.id);
                              setShowDeleteModal(true);
                            }}
                            className="h-8 w-8 text-red-500 hover:bg-red-500/10 hover:text-red-400"
                            title="Delete"
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </div>
                      </div>

                      <div className="flex flex-wrap items-center gap-3 text-xs text-surface-500">
                        <span
                          className={cn(
                            'rounded-full border px-2 py-0.5',
                            instance.loader === 'fabric' &&
                              'border-purple-500/30 bg-purple-500/20 text-purple-400',
                            instance.loader === 'quilt' &&
                              'border-orange-500/30 bg-orange-500/20 text-orange-400',
                            instance.loader === 'none' &&
                              'border-surface-600 bg-surface-700 text-surface-400'
                          )}
                        >
                          {instance.loader === 'none' ? 'Vanilla' : instance.loader}
                          {instance.loaderVersion && ` ${instance.loaderVersion}`}
                        </span>
                        <span className="rounded-full bg-surface-800 px-2 py-0.5">
                          {instance.memory.maxMb}MB RAM
                        </span>
                        {instance.resolution && (
                          <span className="rounded-full bg-surface-800 px-2 py-0.5">
                            {instance.resolution.width}x{instance.resolution.height}
                          </span>
                        )}
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </div>
      </PageLayout>

      {/* Create Instance Modal */}
      <Modal
        isOpen={showCreateModal}
        onClose={() => {
          setShowCreateModal(false);
          setNewInstanceName('');
          setNewInstanceVersion('');
        }}
        title="Create New Instance"
        description="Enter a name and select a Minecraft version"
        size="md"
      >
        <div className="space-y-4">
          <Input
            label="Instance Name"
            placeholder="My Survival World"
            value={newInstanceName}
            onChange={(e) => setNewInstanceName(e.target.value)}
            autoFocus
          />
          <Select
            label="Minecraft Version"
            placeholder="Select version"
            options={versions.map((v) => ({ value: v.id, label: `${v.id} (${v.type})` }))}
            value={newInstanceVersion}
            onChange={(e) => setNewInstanceVersion(e.target.value)}
          />
          <div className="flex justify-end gap-2 pt-4">
            <Button
              variant="secondary"
              onClick={() => {
                setShowCreateModal(false);
                setNewInstanceName('');
                setNewInstanceVersion('');
              }}
            >
              Cancel
            </Button>
            <Button
              onClick={handleCreate}
              disabled={!newInstanceName.trim() || !newInstanceVersion}
            >
              Create Instance
            </Button>
          </div>
        </div>
      </Modal>

      {/* Edit Instance Modal */}
      <Modal
        isOpen={showEditModal}
        onClose={() => {
          setShowEditModal(false);
          setEditingInstance(null);
        }}
        title="Edit Instance"
        description="Modify instance settings"
        size="lg"
      >
        {editingInstance && (
          <div className="space-y-4">
            <div className="grid gap-4 sm:grid-cols-2">
              <Input
                label="Instance Name"
                value={editName}
                onChange={(e) => setEditName(e.target.value)}
              />
              <Select
                label="Minecraft Version"
                options={versions.map((v) => ({ value: v.id, label: `${v.id} (${v.type})` }))}
                value={editVersion}
                onChange={(e) => setEditVersion(e.target.value)}
              />
              <Select
                label="Mod Loader"
                options={[
                  { value: 'none', label: 'Vanilla' },
                  { value: 'fabric', label: 'Fabric' },
                  { value: 'quilt', label: 'Quilt' },
                ]}
                value={editLoader}
                onChange={(e) => setEditLoader(e.target.value as ModLoader)}
              />
              <Input
                label="Loader Version (optional)"
                placeholder="e.g., 0.14.21"
                value={editLoaderVersion}
                onChange={(e) => setEditLoaderVersion(e.target.value)}
              />
              <Input
                label="Min Memory (MB)"
                type="number"
                value={editMemoryMin}
                onChange={(e) => setEditMemoryMin(parseInt(e.target.value) || 512)}
              />
              <Input
                label="Max Memory (MB)"
                type="number"
                value={editMemoryMax}
                onChange={(e) => setEditMemoryMax(parseInt(e.target.value) || 2048)}
              />
            </div>
            <div>
              <label className="label">JVM Arguments</label>
              <textarea
                className="input h-24 resize-y font-mono text-sm"
                value={editJvmArgs}
                onChange={(e) => setEditJvmArgs(e.target.value)}
                placeholder="Additional JVM arguments..."
              />
            </div>
            <div>
              <label className="label">Game Arguments</label>
              <textarea
                className="input h-20 resize-y font-mono text-sm"
                value={editGameArgs}
                onChange={(e) => setEditGameArgs(e.target.value)}
                placeholder="Additional game arguments..."
              />
            </div>
            <div className="flex justify-end gap-2 pt-4">
              <Button
                variant="secondary"
                onClick={() => {
                  setShowEditModal(false);
                  setEditingInstance(null);
                }}
              >
                Cancel
              </Button>
              <Button onClick={handleUpdate}>Save Changes</Button>
            </div>
          </div>
        )}
      </Modal>

      {/* Delete Instance Modal */}
      <Modal
        isOpen={showDeleteModal}
        onClose={() => {
          setShowDeleteModal(false);
          setDeletingInstanceId(null);
        }}
        title="Delete Instance"
        description="This action cannot be undone. The instance and its game data will be permanently removed."
      >
        <div className="space-y-4">
          <div className="rounded-lg border border-red-500/20 bg-red-500/10 p-4">
            <p className="text-red-400">
              Are you sure you want to delete this instance? This will remove the instance
              configuration. The game directory will not be deleted automatically.
            </p>
          </div>
          <div className="flex justify-end gap-2 pt-4">
            <Button
              variant="secondary"
              onClick={() => {
                setShowDeleteModal(false);
                setDeletingInstanceId(null);
              }}
            >
              Cancel
            </Button>
            <Button variant="danger" onClick={handleDelete}>
              Delete Instance
            </Button>
          </div>
        </div>
      </Modal>

      {/* Clone Instance Modal */}
      <Modal
        isOpen={showCloneModal}
        onClose={() => {
          setShowCloneModal(false);
          setCloningInstance(null);
          setCloneName('');
        }}
        title="Clone Instance"
        description="Create a copy of this instance with a new name"
      >
        <div className="space-y-4">
          <Input
            label="New Instance Name"
            placeholder="My Survival World (copy)"
            value={cloneName}
            onChange={(e) => setCloneName(e.target.value)}
            autoFocus
          />
          <div className="flex justify-end gap-2 pt-4">
            <Button
              variant="secondary"
              onClick={() => {
                setShowCloneModal(false);
                setCloningInstance(null);
                setCloneName('');
              }}
            >
              Cancel
            </Button>
            <Button onClick={handleClone} disabled={!cloneName.trim()}>
              Clone Instance
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
