import { useCallback } from 'react';

import type { I_GitAlias } from '#/types';

import { APP_VERSION } from '#/lib/constants';
import { isTauri, tauriAPI } from '#/lib/tauri';

import type { T_ToastType } from './useToast';

interface I_UseAliasActionsOptions {
    editingAlias: I_GitAlias | null;
    deletingAlias: I_GitAlias | null;
    aliases: I_GitAlias[];
    addAlias: (name: string, command: string, scope: 'global' | 'local', localPath?: string) => Promise<void>;
    updateAlias: (oldName: string, name: string, command: string, scope: 'global' | 'local', localPath?: string) => Promise<void>;
    deleteAlias: (name: string, scope: 'global' | 'local', localPath?: string) => Promise<void>;
    addToast: (type: T_ToastType, message: string) => void;
    clearDeletingAlias: () => void;
}

export function useAliasActions({
    editingAlias,
    deletingAlias,
    aliases,
    addAlias,
    updateAlias,
    deleteAlias,
    addToast,
    clearDeletingAlias,
}: I_UseAliasActionsOptions) {
    const handleSave = useCallback(
        async (name: string, command: string, aliasScope: 'global' | 'local', targetLocalPath?: string) => {
            if (editingAlias) {
                if (editingAlias.scope === 'global' && aliasScope === 'local') {
                    await addAlias(name, command, aliasScope, targetLocalPath);
                    addToast('success', `Created new local alias "${name}"`);
                }
                else {
                    await updateAlias(editingAlias.name, name, command, aliasScope, targetLocalPath);
                    addToast('success', `Alias "${name}" updated`);
                }
            }
            else {
                await addAlias(name, command, aliasScope, targetLocalPath);
                addToast('success', `Alias "${name}" created`);
            }
        },
        [editingAlias, addAlias, updateAlias, addToast],
    );

    const handleConfirmDelete = useCallback(async () => {
        if (deletingAlias) {
            try {
                await deleteAlias(deletingAlias.name, deletingAlias.scope, deletingAlias.localPath);
                addToast('success', `Alias "${deletingAlias.name}" deleted`);
            }
            catch (err: any) {
                addToast('error', err.message || 'Failed to delete alias');
            }
            clearDeletingAlias();
        }
    }, [deletingAlias, deleteAlias, addToast, clearDeletingAlias]);

    const handleImport = useCallback(async () => {
        if (!isTauri) {
            addToast('info', 'Import is available in the desktop app');

            return;
        }
        try {
            const result = await tauriAPI.importAliases();

            if (result.success && result.data) {
                let added = 0;

                for (const alias of result.data) {
                    try {
                        await addAlias(alias.name, alias.command, alias.scope || 'global');
                        added++;
                    }
                    catch {
                        // Skip duplicates
                    }
                }
                addToast('success', `Imported ${added} alias${added !== 1 ? 'es' : ''}`);
            }
            else if (result.error) {
                if (!result.error.includes('cancelled')) {
                    addToast('error', result.error);
                }
            }
        }
        catch (err: any) {
            if (!err.message?.includes('cancelled')) {
                addToast('error', err.message || 'Import failed');
            }
        }
    }, [addAlias, addToast]);

    const handleExport = useCallback(async () => {
        if (!isTauri) {
            const data = {
                version: APP_VERSION,
                exportedAt: new Date().toISOString(),
                aliases,
            };
            const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `gam-${new Date().toISOString()}.json`;
            a.click();
            URL.revokeObjectURL(url);
            addToast('success', `Exported ${aliases.length} aliases`);

            return;
        }
        try {
            const result = await tauriAPI.exportAliases(aliases);
            if (result.success) {
                addToast('success', `Exported ${aliases.length} aliases`);
            }
            else if (result.error && !result.error.includes('cancelled')) {
                addToast('error', result.error);
            }
        }
        catch (err: any) {
            if (!err.message?.includes('cancelled')) {
                addToast('error', err.message || 'Export failed');
            }
        }
    }, [aliases, addToast]);

    const handleOpenLocalFolder = useCallback(async (path: string) => {
        if (!isTauri) {
            addToast('info', 'Folder access is only available in the desktop app');

            return;
        }
        try {
            await tauriAPI.openLocalFolder(path);
        }
        catch (err: any) {
            addToast('error', err.message || 'Failed to open folder');
        }
    }, [addToast]);

    return { handleSave, handleConfirmDelete, handleImport, handleExport, handleOpenLocalFolder };
}
