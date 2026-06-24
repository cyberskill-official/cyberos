import { act, renderHook, waitFor } from '@testing-library/react';

import { useAliases } from '#/hooks/useAliases';

const mockGetAliases = vi.fn();
const mockAddAlias = vi.fn();
const mockUpdateAlias = vi.fn();
const mockDeleteAlias = vi.fn();

vi.mock('#/lib/tauri', () => ({
    isTauri: false,
    tauriAPI: {
        getAliases: (...args: unknown[]) => mockGetAliases(...args),
        addAlias: (...args: unknown[]) => mockAddAlias(...args),
        updateAlias: (...args: unknown[]) => mockUpdateAlias(...args),
        deleteAlias: (...args: unknown[]) => mockDeleteAlias(...args),
    },
}));

describe('useAliases', () => {
    beforeEach(() => {
        mockGetAliases.mockReset();
        mockAddAlias.mockReset();
        mockUpdateAlias.mockReset();
        mockDeleteAlias.mockReset();
    });

    it('starts with loading true and empty aliases', () => {
        const { result } = renderHook(() => useAliases());
        expect(result.current.loading).toBe(true);
        expect(result.current.aliases).toEqual([]);
    });

    it('defaults scope to all', () => {
        const { result } = renderHook(() => useAliases());
        expect(result.current.scope).toBe('all');
    });

    it('finishes loading after fetch (mock fallback)', async () => {
        const { result } = renderHook(() => useAliases());

        await waitFor(() => {
            expect(result.current.loading).toBe(false);
        });
    });

    it('setScope changes scope state', async () => {
        const { result } = renderHook(() => useAliases());

        await waitFor(() => {
            expect(result.current.loading).toBe(false);
        });

        act(() => {
            result.current.setScope('global');
        });

        expect(result.current.scope).toBe('global');
    });

    it('exposes addAlias, updateAlias, deleteAlias functions', () => {
        const { result } = renderHook(() => useAliases());
        expect(typeof result.current.addAlias).toBe('function');
        expect(typeof result.current.updateAlias).toBe('function');
        expect(typeof result.current.deleteAlias).toBe('function');
        expect(typeof result.current.fetchAliases).toBe('function');
    });

    it('error is null initially', () => {
        const { result } = renderHook(() => useAliases());
        expect(result.current.error).toBeNull();
    });
});
