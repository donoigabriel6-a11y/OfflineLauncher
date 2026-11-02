// Home / Play page
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Header } from '@/components/layout/Header';
import { PageLayout } from '@/components/layout/Layout';
import { Button } from '@/components/ui/Button';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/Card';
import { Modal } from '@/components/ui/Modal';
import { Input } from '@/components/ui/Input';
import { Select } from '@/components/ui/Select';
import { useStore } from '@/store/useStore';
import { useToast } from '@/components/ui/Toaster';
import { accountsApi, instancesApi, launchApi, javaApi } from '@/services/api';
import { eventManager } from '@/services/events';
import {
  Play,
  Plus,
  Settings,
  FolderOpen,
  AlertTriangle,
  CheckCircle,
  Loader2,
} from 'lucide-react';
import { cn } from '@/lib/utils';

export function Home() {
  const navigate = useNavigate();
  const {
    instances,
    accounts,
    selectedInstanceId,
    setSelectedInstanceId,
    selectedAccountId,
    setSelectedAccountId,
    launchOutputs,
    launchStatus,
    setLaunchStatus,
    addLaunchOutput,
    isLoading,
    setIsLoading,
    setError,
  } = useStore();

  const { toast } = useToast();
  const [showCreateInstance, setShowCreateInstance] = useState(false);
  const [newInstanceName, setNewInstanceName] = useState('');
  const [newInstanceVersion, setNewInstanceVersion] = useState('');
  const [launching, setLaunching] = useState(false);

  const selectedInstance = instances.find((i) => i.id === selectedInstanceId);
  const selectedAccount = accounts.find((a) => a.id === selectedAccountId);
  const currentLaunchStatus = selectedInstanceId ? launchStatus.get(selectedInstanceId) : null;
  const currentOutputs = selectedInstanceId ? launchOutputs.get(selectedInstanceId) || [] : [];

  // Load instances and accounts on mount
  useEffect(() => {
    const loadData = async () => {
      try {
        setIsLoading(true);
        const [instancesData, accountsData, javaData] = await Promise.all([
          instancesApi.list(),
          accountsApi.list(),
          javaApi.listRuntimes(),
        ]);
        useStore.getState().setInstances(instancesData);
        useStore.getState().setAccounts(accountsData);
        useStore.getState().setJavaInstallations(javaData);

        // Auto-select first instance/account if none selected
        if (!selectedInstanceId && instancesData.length > 0) {
          setSelectedInstanceId(instancesData[0].id);
        }
        if (!selectedAccountId && accountsData.length > 0) {
          setSelectedAccountId(accountsData[0].id);
        }
      } catch (error) {
        setError('Failed to load data');
        toast({ type: 'error', title: 'Error', message: 'Failed to load instances and accounts' });
      } finally {
        setIsLoading(false);
      }
    };

    loadData();
  }, [selectedInstanceId, selectedAccountId, setIsLoading, setError, setSelectedInstanceId, setSelectedAccountId, toast]);

  // Listen for launch events
  useEffect(() => {
    eventManager.setupLaunchOutput((output) => {
      addLaunchOutput(output.instanceId, output);
    });

    eventManager.setupLaunchStatus((status) => {
      setLaunchStatus(status);
      if (status.status === 'exited' || status.status === 'crashed') {
        setLaunching(false);
        if (status.status === 'crashed') {
          toast({
            type: 'error',
            title: 'Game Crashed',
            message: `Exit code: ${status.exitCode}`,
          });
        }
      }
    });

    return () => {
      eventManager.cleanup();
    };
  }, [addLaunchOutput, setLaunchStatus, toast]);

  const handleLaunch = async () => {
    if (!selectedInstance || !selectedAccount) {
      toast({ type: 'error', title: 'Error', message: 'Please select an instance and account' });
      return;
    }

    setLaunching(true);
    setError(null);

    try {
      await launchApi.start(selectedInstance.id, selectedAccount.id);
      toast({ type: 'success', title: 'Launching', message: 'Game is starting...' });
    } catch (error) {
      setLaunching(false);
      toast({
        type: 'error',
        title: 'Launch Failed',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  };

  const handleCreateInstance = async () => {
    if (!newInstanceName.trim() || !newInstanceVersion) {
      toast({ type: 'error', title: 'Error', message: 'Please enter a name and select a version' });
      return;
    }

    try {
      const instance = await instancesApi.create(newInstanceName.trim(), newInstanceVersion);
      setShowCreateInstance(false);
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

  const handleOpenFolder = async () => {
    if (!selectedInstance) return;
    try {
      await instancesApi.openFolder(selectedInstance.id);
    } catch (error) {
      toast({ type: 'error', title: 'Error', message: 'Failed to open folder' });
    }
  };

  const canLaunch = selectedInstance && selectedAccount && !launching;

  return (
    <div className="flex h-full flex-col">
      <Header
        title="Play"
        subtitle="Select an instance and account to start playing"
        actions={
          <Button
            variant="secondary"
            size="sm"
            onClick={() => setShowCreateInstance(true)}
            leftIcon={<Plus className="h-4 w-4" />}
          >
            New Instance
          </Button>
        }
      />

      <PageLayout className="flex flex-1" padding="lg">
        <div className="mx-auto w-full max-w-4xl flex-1 space-y-6">
          {/* Instance Selector */}
          <Card>
            <CardContent className="pt-0">
              <div className="mb-4 flex items-center justify-between">
                <h2 className="text-lg font-semibold text-surface-100">Instance</h2>
                <Button variant="ghost" size="sm" onClick={() => navigate('/instances')}>
                  <Settings className="h-4 w-4" />
                  <span className="hidden sm:inline">Manage</span>
                </Button>
              </div>

              {isLoading ? (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin text-primary-500" />
                  <span className="ml-2 text-surface-500">Loading instances...</span>
                </div>
              ) : instances.length === 0 ? (
                <div className="py-8 text-center text-surface-500">
                  <FolderOpen className="mx-auto mb-4 h-12 w-12 opacity-50" />
                  <p>No instances yet. Create one to get started!</p>
                </div>
              ) : (
                <div
                  className="grid gap-3"
                  style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))' }}
                >
                  {instances.map((instance) => (
                    <button
                      key={instance.id}
                      onClick={() => setSelectedInstanceId(instance.id)}
                      className={cn(
                        'group relative rounded-xl border-2 p-4 text-left transition-all duration-200',
                        selectedInstanceId === instance.id
                          ? 'border-primary-500 bg-primary-500/10'
                          : 'border-surface-700 bg-surface-900/50 hover:border-surface-600'
                      )}
                    >
                      <div className="flex items-start justify-between">
                        <div>
                          <h3 className="font-medium text-surface-100 transition-colors group-hover:text-primary-400">
                            {instance.name}
                          </h3>
                          <p className="mt-1 text-sm text-surface-500">
                            {instance.versionId}{' '}
                            {instance.loader !== 'none' && `• ${instance.loader}`}
                          </p>
                          <p className="mt-1 text-xs text-surface-600">
                            {instance.memory.maxMb}MB RAM
                          </p>
                        </div>
                        {selectedInstanceId === instance.id && (
                          <CheckCircle className="h-5 w-5 text-primary-500" />
                        )}
                      </div>
                    </button>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>

          {/* Account Selector */}
          <Card>
            <CardContent className="pt-0">
              <div className="mb-4 flex items-center justify-between">
                <h2 className="text-lg font-semibold text-surface-100">Account</h2>
                <Button variant="ghost" size="sm" onClick={() => navigate('/accounts')}>
                  <Settings className="h-4 w-4" />
                  <span className="hidden sm:inline">Manage</span>
                </Button>
              </div>

              {accounts.length === 0 ? (
                <div className="py-8 text-center text-surface-500">
                  <p>No accounts yet. Create one in the Accounts page!</p>
                </div>
              ) : (
                <div
                  className="grid gap-3"
                  style={{ gridTemplateColumns: 'repeat(auto-fill, minmax(150px, 1fr))' }}
                >
                  {accounts.map((account) => (
                    <button
                      key={account.id}
                      onClick={() => setSelectedAccountId(account.id)}
                      className={cn(
                        'group relative rounded-lg border-2 p-3 text-center transition-all duration-200',
                        selectedAccountId === account.id
                          ? 'border-primary-500 bg-primary-500/10'
                          : 'border-surface-700 bg-surface-900/50 hover:border-surface-600'
                      )}
                    >
                      <div className="mx-auto mb-2 flex h-10 w-10 items-center justify-center rounded-full bg-primary-600/20 transition-colors group-hover:bg-primary-600/30">
                        <span className="text-lg font-medium text-primary-400">
                          {account.username.charAt(0).toUpperCase()}
                        </span>
                      </div>
                      <p className="truncate text-sm font-medium text-surface-100">
                        {account.username}
                      </p>
                      <p className="truncate text-xs text-surface-600">
                        {account.uuid.slice(0, 8)}...
                      </p>
                      {selectedAccountId === account.id && (
                        <CheckCircle className="absolute right-2 top-2 h-4 w-4 text-primary-500" />
                      )}
                    </button>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>

          {/* Launch Button & Status */}
          <Card className={cn(selectedInstance && 'border-primary-500/30')}>
            <CardContent className="pt-0">
              {currentLaunchStatus && currentLaunchStatus.status !== 'exited' ? (
                <div className="space-y-4 p-6 text-center">
                  <Loader2 className="mx-auto h-12 w-12 animate-spin text-primary-500" />
                  <div>
                    <p className="text-lg font-medium capitalize text-surface-100">
                      {currentLaunchStatus.status === 'starting'
                        ? 'Starting game...'
                        : 'Game running'}
                    </p>
                    <p className="mt-1 text-sm text-surface-500">
                      PID: {currentLaunchStatus.pid || 'unknown'}
                    </p>
                  </div>
                  <Button
                    variant="danger"
                    onClick={() => launchApi.kill(selectedInstance!.id)}
                    leftIcon={<AlertTriangle className="h-4 w-4" />}
                  >
                    Stop Game
                  </Button>
                </div>
              ) : (
                <Button
                  onClick={handleLaunch}
                  disabled={!canLaunch}
                  className="w-full py-4 text-lg"
                  leftIcon={
                    launching ? (
                      <Loader2 className="h-5 w-5 animate-spin" />
                    ) : (
                      <Play className="h-5 w-5" />
                    )
                  }
                >
                  {launching ? 'Launching...' : 'Play'}
                </Button>
              )}

              {/* Quick Actions */}
              <div className="mt-4 flex items-center justify-center gap-3 border-t border-surface-700 pt-4">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleOpenFolder}
                  leftIcon={<FolderOpen className="h-4 w-4" />}
                >
                  Open Folder
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => navigate('/instances')}
                  leftIcon={<Settings className="h-4 w-4" />}
                >
                  Instance Settings
                </Button>
              </div>
            </CardContent>
          </Card>

          {/* Console Output */}
          {currentOutputs.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center justify-between">
                  Console Output
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => useStore.getState().clearLaunchOutputs(selectedInstanceId!)}
                  >
                    Clear
                  </Button>
                </CardTitle>
              </CardHeader>
              <CardContent className="pt-0">
                <div className="scrollbar-thin max-h-64 overflow-y-auto rounded-lg bg-surface-950 p-4 font-mono text-xs">
                  {currentOutputs.slice(-100).map((output, index) => (
                    <div
                      key={index}
                      className={cn(
                        'font-mono text-xs',
                        output.stream === 'stderr' ? 'text-red-400' : 'text-surface-300'
                      )}
                    >
                      {output.line}
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </PageLayout>

      {/* Create Instance Modal */}
      <Modal
        isOpen={showCreateInstance}
        onClose={() => setShowCreateInstance(false)}
        title="Create New Instance"
        description="Enter a name and select a Minecraft version"
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
            options={[]} // Would be populated from versions store
            value={newInstanceVersion}
            onChange={(e) => setNewInstanceVersion(e.target.value)}
          />
          <div className="flex justify-end gap-2 pt-4">
            <Button variant="secondary" onClick={() => setShowCreateInstance(false)}>
              Cancel
            </Button>
            <Button onClick={handleCreateInstance}>Create Instance</Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
