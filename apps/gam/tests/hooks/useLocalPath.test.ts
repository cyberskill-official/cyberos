import { act, renderHook } from '@testing-library/react';

import { useLocalPath } from '#/hooks/useLocalPath';

const mockGetLocalPath = vi.fn();
const mockSelectFolder = vi.fn();
const mockSetLocalPath = vi.fn();

vi.mock('#/lib/tauri', () => ({
    isTauri: false,
    tauriAPI: {
        getLocalPath: () => mockGetLocalPath(),
        selectFolder: () => mockSelectFolder(),
        setLocalPath: (path: string) => mockSetLocalPath(path),
    },
}));

describe('useLocalPath', () => {
    const addToast = vi.fn();
    const fetchAliases = vi.fn().mockResolvedValue(undefined);

    beforeEach(() => {
        addToast.mockReset();
        fetchAliases.mockReset().mockResolvedValue(undefined);
        mockGetLocalPath.mockReset();
        mockSelectFolder.mockReset();
        mockSetLocalPath.mockReset();
    });

    it('starts with empty localPath', () => {
        const { result } = renderHook(() => useLocalPath(addToast, fetchAliases));
        expect(result.current.localPath).toBe('');
    });

    it('exposes handleSelectFolder and handleClearFolder', () => {
        const { result } = renderHook(() => useLocalPath(addToast, fetchAliases));
        expect(typeof result.current.handleSelectFolder).toBe('function');
        expect(typeof result.current.handleClearFolder).toBe('function');
    });

    it('setLocalPath updates state directly', () => {
        const { result } = renderHook(() => useLocalPath(addToast, fetchAliases));

        act(() => {
            result.current.setLocalPath('/some/path');
        });

        expect(result.current.localPath).toBe('/some/path');
    });

    it('handleSelectFolder is no-op when not in Tauri', async () => {
        const { result } = renderHook(() => useLocalPath(addToast, fetchAliases));

        await act(async () => {
            await result.current.handleSelectFolder();
        });

        expect(mockSelectFolder).not.toHaveBeenCalled();
    });

    it('handleClearFolder is no-op when not in Tauri', async () => {
        const { result } = renderHook(() => useLocalPath(addToast, fetchAliases));

        await act(async () => {
            await result.current.handleClearFolder();
        });

        expect(mockSetLocalPath).not.toHaveBeenCalled();
    });
});
