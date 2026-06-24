import { useCallback, useEffect, useState } from 'react';

import type { I_AliasGroup } from '#/types';

import { isTauri, tauriAPI } from '#/lib/tauri';

export function useGroups() {
    const [groups, setGroups] = useState<I_AliasGroup[]>([]);
    const [assignments, setAssignments] = useState<Record<string, string[]>>({});
    const [activeGroupId, setActiveGroupId] = useState<string | null>(null);
    const [loading, setLoading] = useState(true);

    const fetchGroups = useCallback(async () => {
        if (!isTauri) {
            setLoading(false);
            return;
        }

        try {
            const [groupsRes, assignmentsRes] = await Promise.all([
                tauriAPI.getGroups(),
                tauriAPI.getAllGroupAssignments(),
            ]);

            if (groupsRes.success && groupsRes.data) {
                setGroups(groupsRes.data);
            }
            if (assignmentsRes.success && assignmentsRes.data) {
                setAssignments(assignmentsRes.data);
            }
        }
        catch (err) {
            console.error('[GAM] Failed to fetch groups:', err);
        }
        finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        fetchGroups();
    }, [fetchGroups]);

    const createGroup = useCallback(async (name: string, color: string) => {
        if (!isTauri)
            return null;
        const res = await tauriAPI.createGroup(name, color);
        if (res.success && res.data) {
            setGroups(prev => [...prev, res.data!]);
            return res.data;
        }
        return null;
    }, []);

    const renameGroup = useCallback(async (groupId: string, newName: string) => {
        if (!isTauri)
            return;
        const res = await tauriAPI.renameGroup(groupId, newName);
        if (res.success) {
            setGroups(prev => prev.map(g => (g.id === groupId ? { ...g, name: newName } : g)));
        }
    }, []);

    const setGroupColor = useCallback(async (groupId: string, color: string) => {
        // Optimistic update — show new color immediately
        setGroups(prev => prev.map(g => (g.id === groupId ? { ...g, color } : g)));

        if (!isTauri)
            return;
        try {
            const res = await tauriAPI.setGroupColor(groupId, color);
            if (!res.success) {
                console.error('[GAM] Failed to set group color:', res.error);
            }
        }
        catch (err) {
            console.error('[GAM] Failed to set group color:', err);
        }
    }, []);

    const deleteGroup = useCallback(async (groupId: string) => {
        if (!isTauri)
            return;
        const res = await tauriAPI.deleteGroup(groupId);
        if (res.success) {
            setGroups(prev => prev.filter(g => g.id !== groupId));
            // Clean assignments
            setAssignments((prev) => {
                const next: Record<string, string[]> = {};
                for (const [alias, ids] of Object.entries(prev)) {
                    const filtered = ids.filter(id => id !== groupId);
                    if (filtered.length > 0)
                        next[alias] = filtered;
                }
                return next;
            });
            if (activeGroupId === groupId)
                setActiveGroupId(null);
        }
    }, [activeGroupId]);

    const setAliasGroups = useCallback(async (aliasName: string, groupIds: string[]) => {
        if (!isTauri)
            return;
        await tauriAPI.setAliasGroups(aliasName, groupIds);
        setAssignments((prev) => {
            if (groupIds.length === 0) {
                const { [aliasName]: _, ...rest } = prev;
                return rest;
            }
            return { ...prev, [aliasName]: groupIds };
        });
    }, []);

    const getAliasGroupIds = useCallback(
        (aliasName: string) => assignments[aliasName] ?? [],
        [assignments],
    );

    return {
        groups,
        assignments,
        activeGroupId,
        setActiveGroupId,
        loading,
        createGroup,
        renameGroup,
        setGroupColor,
        deleteGroup,
        setAliasGroups,
        getAliasGroupIds,
        fetchGroups,
    };
}
