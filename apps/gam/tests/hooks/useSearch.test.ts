import { act, renderHook } from '@testing-library/react';

import type { I_GitAlias } from '#/types';

import { useSearch } from '#/hooks/useSearch';

vi.mock('#/lib/tauri', () => ({
    isTauri: false,
    tauriAPI: {},
}));

const mockAliases: I_GitAlias[] = [
    { name: 'co', command: 'checkout', scope: 'global' },
    { name: 'st', command: 'status -sb', scope: 'global' },
    { name: 'br', command: 'branch', scope: 'local' },
    { name: 'lg', command: 'log --oneline --graph', scope: 'global' },
];

describe('useSearch', () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it('returns all aliases when search is empty', () => {
        const { result } = renderHook(() => useSearch(mockAliases));
        expect(result.current.filteredAliases).toEqual(mockAliases);
        expect(result.current.searchQuery).toBe('');
    });

    it('filters by alias name after debounce', () => {
        const { result } = renderHook(() => useSearch(mockAliases));

        act(() => {
            result.current.setSearchQuery('co');
        });

        // Before debounce: still all aliases
        expect(result.current.filteredAliases).toEqual(mockAliases);

        // After debounce
        act(() => {
            vi.advanceTimersByTime(350);
        });

        expect(result.current.filteredAliases).toHaveLength(1);
        expect(result.current.filteredAliases[0].name).toBe('co');
    });

    it('filters by command content', () => {
        const { result } = renderHook(() => useSearch(mockAliases));

        act(() => {
            result.current.setSearchQuery('graph');
        });

        act(() => {
            vi.advanceTimersByTime(350);
        });

        expect(result.current.filteredAliases).toHaveLength(1);
        expect(result.current.filteredAliases[0].name).toBe('lg');
    });

    it('is case-insensitive', () => {
        const { result } = renderHook(() => useSearch(mockAliases));

        act(() => {
            result.current.setSearchQuery('CHECKOUT');
        });

        act(() => {
            vi.advanceTimersByTime(350);
        });

        expect(result.current.filteredAliases).toHaveLength(1);
        expect(result.current.filteredAliases[0].name).toBe('co');
    });

    it('returns empty array when no matches', () => {
        const { result } = renderHook(() => useSearch(mockAliases));

        act(() => {
            result.current.setSearchQuery('nonexistent');
        });

        act(() => {
            vi.advanceTimersByTime(350);
        });

        expect(result.current.filteredAliases).toHaveLength(0);
    });

    it('updates debouncedQuery after delay', () => {
        const { result } = renderHook(() => useSearch(mockAliases));

        act(() => {
            result.current.setSearchQuery('test');
        });

        expect(result.current.debouncedQuery).toBe('');

        act(() => {
            vi.advanceTimersByTime(350);
        });

        expect(result.current.debouncedQuery).toBe('test');
    });

    it('handles empty aliases array', () => {
        const { result } = renderHook(() => useSearch([]));

        act(() => {
            result.current.setSearchQuery('anything');
        });

        act(() => {
            vi.advanceTimersByTime(350);
        });

        expect(result.current.filteredAliases).toHaveLength(0);
    });
});
