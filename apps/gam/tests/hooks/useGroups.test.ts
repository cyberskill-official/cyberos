import { act, renderHook, waitFor } from '@testing-library/react';

import { useGroups } from '#/hooks/useGroups';

vi.mock('#/lib/tauri', () => ({
    isTauri: false,
    tauriAPI: {},
}));

describe('useGroups', () => {
    it('starts with empty groups and assignments', async () => {
        const { result } = renderHook(() => useGroups());

        await waitFor(() => {
            expect(result.current.loading).toBe(false);
        });

        expect(result.current.groups).toEqual([]);
        expect(result.current.assignments).toEqual({});
    });

    it('defaults activeGroupId to null', () => {
        const { result } = renderHook(() => useGroups());
        expect(result.current.activeGroupId).toBeNull();
    });

    it('setActiveGroupId changes activeGroupId', () => {
        const { result } = renderHook(() => useGroups());

        act(() => {
            result.current.setActiveGroupId('g-123');
        });

        expect(result.current.activeGroupId).toBe('g-123');
    });

    it('setActiveGroupId to null clears filter', () => {
        const { result } = renderHook(() => useGroups());

        act(() => {
            result.current.setActiveGroupId('g-123');
        });

        act(() => {
            result.current.setActiveGroupId(null);
        });

        expect(result.current.activeGroupId).toBeNull();
    });

    it('createGroup returns null when not in Tauri', async () => {
        const { result } = renderHook(() => useGroups());

        let group: unknown;
        await act(async () => {
            group = await result.current.createGroup('Test', '#ff0000');
        });

        expect(group).toBeNull();
    });

    it('getAliasGroupIds returns empty array for unknown alias', () => {
        const { result } = renderHook(() => useGroups());
        expect(result.current.getAliasGroupIds('nonexistent')).toEqual([]);
    });

    it('exposes all required functions', () => {
        const { result } = renderHook(() => useGroups());

        expect(typeof result.current.createGroup).toBe('function');
        expect(typeof result.current.renameGroup).toBe('function');
        expect(typeof result.current.deleteGroup).toBe('function');
        expect(typeof result.current.setAliasGroups).toBe('function');
        expect(typeof result.current.fetchGroups).toBe('function');
    });
});
