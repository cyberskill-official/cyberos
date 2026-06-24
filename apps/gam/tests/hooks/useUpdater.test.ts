import { act, renderHook, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { useUpdater } from '#/hooks/useUpdater';

const mockCheck = vi.fn();
const mockRelaunch = vi.fn();

vi.mock('@tauri-apps/plugin-updater', () => ({
    check: (...args: unknown[]) => mockCheck(...args),
}));

vi.mock('@tauri-apps/plugin-process', () => ({
    relaunch: (...args: unknown[]) => mockRelaunch(...args),
}));

/* mock isTauri so the hook actually runs */
vi.mock('#/lib/tauri', () => ({
    isTauri: true,
    tauriAPI: {},
}));

describe('useUpdater', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('returns no update when check() resolves null', async () => {
        mockCheck.mockResolvedValue(null);

        const { result } = renderHook(() => useUpdater());

        expect(result.current.checking).toBe(true);

        await waitFor(() => {
            expect(result.current.checking).toBe(false);
        });

        expect(result.current.updateAvailable).toBe(false);
        expect(result.current.version).toBe('');
    });

    it('returns update metadata when available', async () => {
        mockCheck.mockResolvedValue({
            version: '2.0.0',
            body: 'New features and improvements',
            downloadAndInstall: vi.fn(),
        });

        const { result } = renderHook(() => useUpdater());

        await waitFor(() => {
            expect(result.current.updateAvailable).toBe(true);
        });

        expect(result.current.version).toBe('2.0.0');
        expect(result.current.changelog).toBe('New features and improvements');
        expect(result.current.checking).toBe(false);
    });

    it('handles check errors gracefully', async () => {
        mockCheck.mockRejectedValue(new Error('Network error'));

        const { result } = renderHook(() => useUpdater());

        await waitFor(() => {
            expect(result.current.checking).toBe(false);
        });

        expect(result.current.error).toBe('Network error');
        expect(result.current.updateAvailable).toBe(false);
    });

    it('dismiss clears update state', async () => {
        mockCheck.mockResolvedValue({
            version: '2.0.0',
            body: 'Changes',
            downloadAndInstall: vi.fn(),
        });

        const { result } = renderHook(() => useUpdater());

        await waitFor(() => {
            expect(result.current.updateAvailable).toBe(true);
        });

        act(() => {
            result.current.dismiss();
        });

        expect(result.current.updateAvailable).toBe(false);
        expect(result.current.version).toBe('');
    });

    it('startDownload calls downloadAndInstall and relaunches', async () => {
        const mockDownloadAndInstall = vi.fn().mockImplementation(async (cb) => {
            cb({ event: 'Started', data: { contentLength: 1000 } });
            cb({ event: 'Progress', data: { chunkLength: 500 } });
            cb({ event: 'Progress', data: { chunkLength: 500 } });
            cb({ event: 'Finished', data: {} });
        });
        mockRelaunch.mockResolvedValue(undefined);

        mockCheck.mockResolvedValue({
            version: '2.0.0',
            body: '',
            downloadAndInstall: mockDownloadAndInstall,
        });

        const { result } = renderHook(() => useUpdater());

        await waitFor(() => {
            expect(result.current.updateAvailable).toBe(true);
        });

        await act(async () => {
            await result.current.startDownload();
        });

        expect(mockDownloadAndInstall).toHaveBeenCalled();
        expect(mockRelaunch).toHaveBeenCalled();
    });

    it('handles download errors gracefully', async () => {
        const mockDownloadAndInstall = vi.fn().mockRejectedValue(new Error('Download failed'));

        mockCheck.mockResolvedValue({
            version: '2.0.0',
            body: '',
            downloadAndInstall: mockDownloadAndInstall,
        });

        const { result } = renderHook(() => useUpdater());

        await waitFor(() => {
            expect(result.current.updateAvailable).toBe(true);
        });

        await act(async () => {
            await result.current.startDownload();
        });

        expect(result.current.error).toBe('Download failed');
        expect(result.current.downloading).toBe(false);
    });
});
