// Accounts page
import { useEffect, useState } from 'react';
import { Header } from '@/components/layout/Header';
import { PageLayout } from '@/components/layout/Layout';
import { Button } from '@/components/ui/Button';
import { Card, CardContent } from '@/components/ui/Card';
import { Modal } from '@/components/ui/Modal';
import { Input } from '@/components/ui/Input';
import { useStore } from '@/store/useStore';
import { useToast } from '@/components/ui/Toaster';
import { accountsApi } from '@/services/api';
import { eventManager } from '@/services/events';
import type { Account } from '@/types';
import { Plus, Trash2, Edit, Copy, AlertTriangle } from 'lucide-react';
import { cn } from '@/lib/utils';

export function Accounts() {
  const { accounts, setAccounts, selectedAccountId } = useStore();
  const { toast } = useToast();
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showEditModal, setShowEditModal] = useState(false);
  const [showDeleteModal, setShowDeleteModal] = useState(false);
  const [newUsername, setNewUsername] = useState('');
  const [editingAccount, setEditingAccount] = useState<Account | null>(null);
  const [editUsername, setEditUsername] = useState('');
  const [deletingAccountId, setDeletingAccountId] = useState<string | null>(null);
  const [usernameError, setUsernameError] = useState('');

  // Load accounts on mount and listen for changes
  useEffect(() => {
    const loadAccounts = async () => {
      try {
        const data = await accountsApi.list();
        setAccounts(data);
      } catch (error) {
        toast({ type: 'error', title: 'Error', message: 'Failed to load accounts' });
      }
    };

    loadAccounts();

    eventManager.setupAccountsChanged((event) => {
      if (event.action === 'created') {
        // Account added via another window
        loadAccounts();
      } else if (event.action === 'updated') {
        loadAccounts();
      } else if (event.action === 'deleted') {
        loadAccounts();
      }
    });

    return () => eventManager.cleanup();
  }, [setAccounts, toast]);

  const validateUsername = (username: string): boolean => {
    if (username.length < 3 || username.length > 16) {
      setUsernameError('Username must be 3-16 characters');
      return false;
    }
    if (!/^[A-Za-z0-9_]+$/.test(username)) {
      setUsernameError('Username can only contain letters, numbers, and underscores');
      return false;
    }
    // Check if username already exists (excluding the one being edited)
    const exists = accounts.some(
      (a) => a.username.toLowerCase() === username.toLowerCase() && a.id !== editingAccount?.id
    );
    if (exists) {
      setUsernameError('Username already exists');
      return false;
    }
    setUsernameError('');
    return true;
  };

  const handleCreate = async () => {
    if (!validateUsername(newUsername.trim())) return;

    try {
      await accountsApi.create(newUsername.trim());
      setShowCreateModal(false);
      setNewUsername('');
      toast({
        type: 'success',
        title: 'Account Created',
        message: `Welcome, ${newUsername.trim()}!`,
      });
    } catch (error) {
      toast({
        type: 'error',
        title: 'Failed to Create',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  };

  const handleUpdate = async () => {
    if (!editingAccount || !validateUsername(editUsername.trim())) return;

    try {
      await accountsApi.update(editingAccount.id, editUsername.trim());
      setShowEditModal(false);
      setEditingAccount(null);
      setEditUsername('');
      toast({ type: 'success', title: 'Account Updated', message: 'Changes saved' });
    } catch (error) {
      toast({
        type: 'error',
        title: 'Failed to Update',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  };

  const handleDelete = async () => {
    if (!deletingAccountId) return;

    try {
      await accountsApi.delete(deletingAccountId);
      setShowDeleteModal(false);
      setDeletingAccountId(null);
      toast({ type: 'success', title: 'Account Deleted', message: 'Account removed' });
    } catch (error) {
      toast({
        type: 'error',
        title: 'Failed to Delete',
        message: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  };

  const copyUsername = (username: string) => {
    navigator.clipboard.writeText(username);
    toast({ type: 'success', title: 'Copied', message: 'Username copied to clipboard' });
  };

  const copyUuid = (uuid: string) => {
    navigator.clipboard.writeText(uuid);
    toast({ type: 'success', title: 'Copied', message: 'UUID copied to clipboard' });
  };

  return (
    <div className="flex h-full flex-col">
      <Header
        title="Accounts"
        subtitle="Manage your offline accounts"
        actions={
          <Button onClick={() => setShowCreateModal(true)} leftIcon={<Plus className="h-4 w-4" />}>
            Add Account
          </Button>
        }
      />

      <PageLayout>
        <div className="mx-auto w-full max-w-3xl space-y-4">
          {accounts.length === 0 ? (
            <Card variant="outlined">
              <CardContent className="py-12 text-center">
                <AlertTriangle className="mx-auto mb-4 h-16 w-16 text-surface-600" />
                <h3 className="mb-2 text-lg font-medium text-surface-100">No accounts yet</h3>
                <p className="mb-6 text-surface-500">Create an offline account to start playing</p>
                <Button
                  onClick={() => setShowCreateModal(true)}
                  leftIcon={<Plus className="h-4 w-4" />}
                >
                  Create Your First Account
                </Button>
              </CardContent>
            </Card>
          ) : (
            <div className="space-y-3">
              {accounts.map((account) => (
                <Card
                  key={account.id}
                  hover
                  className={cn(selectedAccountId === account.id && 'border-primary-500/50')}
                >
                  <CardContent className="pt-0">
                    <div className="flex items-center gap-4 p-4">
                      <div className="flex h-12 w-12 flex-shrink-0 items-center justify-center rounded-full bg-primary-600/20">
                        <span className="text-xl font-bold text-primary-400">
                          {account.username.charAt(0).toUpperCase()}
                        </span>
                      </div>
                      <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2">
                          <h3 className="truncate font-medium text-surface-100">
                            {account.username}
                          </h3>
                          <span className="rounded-full bg-surface-800 px-2 py-0.5 text-xs uppercase text-surface-400">
                            offline
                          </span>
                        </div>
                        <div className="mt-1 flex items-center gap-4 text-sm text-surface-500">
                          <span className="flex-1 truncate font-mono" title={account.uuid}>
                            {account.uuid}
                          </span>
                          <Button
                            variant="ghost"
                            size="sm"
                            onClick={() => copyUuid(account.uuid)}
                            className="h-6 px-2"
                          >
                            <Copy className="h-3 w-3" />
                          </Button>
                        </div>
                      </div>
                      <div className="flex items-center gap-2">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => copyUsername(account.username)}
                          className="h-8 w-8"
                        >
                          <Copy className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => {
                            setEditingAccount(account);
                            setEditUsername(account.username);
                            setShowEditModal(true);
                          }}
                          className="h-8 w-8"
                        >
                          <Edit className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => {
                            setDeletingAccountId(account.id);
                            setShowDeleteModal(true);
                          }}
                          className="h-8 w-8 text-red-500 hover:bg-red-500/10 hover:text-red-400"
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </div>
      </PageLayout>

      {/* Create Account Modal */}
      <Modal
        isOpen={showCreateModal}
        onClose={() => {
          setShowCreateModal(false);
          setNewUsername('');
          setUsernameError('');
        }}
        title="Create Offline Account"
        description="Enter a username (3-16 characters, letters, numbers, underscores only)"
      >
        <div className="space-y-4">
          <Input
            label="Username"
            placeholder="Steve"
            value={newUsername}
            onChange={(e) => {
              setNewUsername(e.target.value);
              validateUsername(e.target.value);
            }}
            error={usernameError}
            autoFocus
            helperText="This will be your in-game name. Must be 3-16 characters, alphanumeric and underscores only."
          />
          <div className="flex justify-end gap-2 pt-4">
            <Button
              variant="secondary"
              onClick={() => {
                setShowCreateModal(false);
                setNewUsername('');
                setUsernameError('');
              }}
            >
              Cancel
            </Button>
            <Button onClick={handleCreate} disabled={!validateUsername(newUsername)}>
              Create Account
            </Button>
          </div>
        </div>
      </Modal>

      {/* Edit Account Modal */}
      <Modal
        isOpen={showEditModal}
        onClose={() => {
          setShowEditModal(false);
          setEditingAccount(null);
          setEditUsername('');
          setUsernameError('');
        }}
        title="Edit Account"
        description="Change the username for this account"
      >
        <div className="space-y-4">
          <Input
            label="Username"
            placeholder="Steve"
            value={editUsername}
            onChange={(e) => {
              setEditUsername(e.target.value);
              validateUsername(e.target.value);
            }}
            error={usernameError}
            autoFocus
            helperText="Must be 3-16 characters, alphanumeric and underscores only."
          />
          <div className="flex justify-end gap-2 pt-4">
            <Button
              variant="secondary"
              onClick={() => {
                setShowEditModal(false);
                setEditingAccount(null);
                setEditUsername('');
                setUsernameError('');
              }}
            >
              Cancel
            </Button>
            <Button onClick={handleUpdate} disabled={!validateUsername(editUsername)}>
              Save Changes
            </Button>
          </div>
        </div>
      </Modal>

      {/* Delete Account Modal */}
      <Modal
        isOpen={showDeleteModal}
        onClose={() => {
          setShowDeleteModal(false);
          setDeletingAccountId(null);
        }}
        title="Delete Account"
        description="This action cannot be undone. The account will be permanently removed."
      >
        <div className="space-y-4">
          <div className="rounded-lg border border-red-500/20 bg-red-500/10 p-4">
            <p className="text-red-400">
              Are you sure you want to delete this account? This will remove it from the launcher.
            </p>
          </div>
          <div className="flex justify-end gap-2 pt-4">
            <Button
              variant="secondary"
              onClick={() => {
                setShowDeleteModal(false);
                setDeletingAccountId(null);
              }}
            >
              Cancel
            </Button>
            <Button variant="danger" onClick={handleDelete}>
              Delete Account
            </Button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
