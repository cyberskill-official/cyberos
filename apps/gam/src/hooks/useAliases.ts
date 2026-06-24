import { useCallback, useEffect, useState } from 'react';

import type { I_GitAlias, I_IpcResult } from '#/types';

import { isTauri, tauriAPI } from '#/lib/tauri';

type T_Scope = 'global' | 'local' | 'all';

// Mock API for browser-based development
const mockAliases: I_GitAlias[] = [];

const mockAPI = {
    getAliases: async (_scope: T_Scope): Promise<I_IpcResult<I_GitAlias[]>> => {
        return { success: true, data: mockAliases };
    },
    addAlias: async (
        name: string,
        command: string,
        scope: 'global' | 'local',
        localPath?: string,
    ): Promise<I_IpcResult> => {
        mockAliases.push({ name, command, scope, localPath });
        return { success: true };
    },
    updateAlias: async (
        oldName: string,
        name: string,
        command: string,
        scope: 'global' | 'local',
        localPath?: string,
    ): Promise<I_IpcResult> => {
        const idx = mockAliases.findIndex(a => a.name === oldName);
        if (idx !== -1) {
            mockAliases[idx] = { name, command, scope, localPath };
        }
        return { success: true };
    },
    deleteAlias: async (name: string, _scope: 'global' | 'local', _localPath?: string): Promise<I_IpcResult> => {
        const idx = mockAliases.findIndex(a => a.name === name);
        if (idx !== -1) {
            mockAliases.splice(idx, 1);
        }
        return { success: true };
    },
};

function getAPI() {
    return isTauri ? tauriAPI : mockAPI;
}

export function useAliases() {
    const [aliases, setAliases] = useState<I_GitAlias[]>([]);
    const [loading, setLoading] = useState(true);
    const [scope, setScope] = useState<T_Scope>('all');
    const [error, setError] = useState<string | null>(null);

    const fetchAliases = useCallback(async () => {
        setLoading(true);
        setError(null);

        try {
            const api = getAPI();
            const result = await api.getAliases(scope);

            if ('success' in result && result.success && result.data) {
                setAliases(result.data);
            }
            else if ('success' in result && !result.success) {
                setError(result.error || 'Failed to fetch aliases');
                setAliases([]);
            }
        }
        catch (err: any) {
            setError(err.message || 'Failed to fetch aliases');
            setAliases([]);
        }
        finally {
            setLoading(false);
        }
    }, [scope]);

    useEffect(() => {
        fetchAliases();
    }, [fetchAliases]);

    const addAlias = useCallback(
        async (name: string, command: string, aliasScope: 'global' | 'local', localPath?: string) => {
            // Optimistic: add to local state immediately
            const optimistic: I_GitAlias = { name, command, scope: aliasScope, localPath };
            setAliases(prev => [...prev, optimistic]);

            try {
                const api = getAPI();
                const result = await api.addAlias(name, command, aliasScope, localPath);

                if ('success' in result && !result.success) {
                    throw new Error(result.error || 'Failed to add alias');
                }
                // Background refresh to get server-confirmed state (scores, etc.)
                await fetchAliases();
            }
            catch (err) {
                // Rollback on failure
                await fetchAliases();
                throw err;
            }
        },
        [fetchAliases],
    );

    const updateAlias = useCallback(
        async (
            oldName: string,
            name: string,
            command: string,
            aliasScope: 'global' | 'local',
            localPath?: string,
        ) => {
            // Optimistic: patch alias in local state immediately
            setAliases(prev =>
                prev.map(a =>
                    a.name === oldName && a.scope === aliasScope
                        ? { ...a, name, command, scope: aliasScope, localPath }
                        : a,
                ),
            );

            try {
                const api = getAPI();
                const result = await api.updateAlias(oldName, name, command, aliasScope, localPath);

                if ('success' in result && !result.success) {
                    throw new Error(result.error || 'Failed to update alias');
                }
                await fetchAliases();
            }
            catch (err) {
                await fetchAliases();
                throw err;
            }
        },
        [fetchAliases],
    );

    const deleteAlias = useCallback(
        async (name: string, aliasScope: 'global' | 'local', localPath?: string) => {
            // Optimistic: remove from local state immediately
            const snapshot = aliases;
            setAliases(prev => prev.filter(a => !(a.name === name && a.scope === aliasScope)));

            try {
                const api = getAPI();
                const result = await api.deleteAlias(name, aliasScope, localPath);

                if ('success' in result && !result.success) {
                    throw new Error(result.error || 'Failed to delete alias');
                }
                await fetchAliases();
            }
            catch (err) {
                // Rollback: restore snapshot on failure
                setAliases(snapshot);
                throw err;
            }
        },
        [aliases, fetchAliases],
    );

    return {
        aliases,
        loading,
        error,
        scope,
        setScope,
        fetchAliases,
        addAlias,
        updateAlias,
        deleteAlias,
    };
}
