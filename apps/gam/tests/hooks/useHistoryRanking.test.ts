import { act, renderHook } from '@testing-library/react';

import { useHistoryRanking } from '#/hooks/useHistoryRanking';

const mockGet = vi.fn();
const mockSet = vi.fn();

vi.mock('#/lib/tauri', () => ({
    isTauri: false,
    tauriAPI: {
        getHistoryRankingEnabled: () => mockGet(),
        setHistoryRankingEnabled: (enabled: boolean) => mockSet(enabled),
    },
}));

describe('useHistoryRanking', () => {
    beforeEach(() => {
        localStorage.clear();
        mockGet.mockReset();
        mockSet.mockReset();
    });

    it('defaults to enabled', () => {
        const { result } = renderHook(() => useHistoryRanking());
        expect(result.current.enabled).toBe(true);
    });

    it('reads a stored opt-out from localStorage', () => {
        localStorage.setItem('gam-history-ranking-enabled', 'false');
        const { result } = renderHook(() => useHistoryRanking());
        expect(result.current.enabled).toBe(false);
    });

    it('setEnabled(false) disables and persists to localStorage', () => {
        const { result } = renderHook(() => useHistoryRanking());

        act(() => {
            result.current.setEnabled(false);
        });

        expect(result.current.enabled).toBe(false);
        expect(localStorage.getItem('gam-history-ranking-enabled')).toBe('false');
    });

    it('setEnabled(true) re-enables and persists', () => {
        localStorage.setItem('gam-history-ranking-enabled', 'false');
        const { result } = renderHook(() => useHistoryRanking());

        act(() => {
            result.current.setEnabled(true);
        });

        expect(result.current.enabled).toBe(true);
        expect(localStorage.getItem('gam-history-ranking-enabled')).toBe('true');
    });
});
