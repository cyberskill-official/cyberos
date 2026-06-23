import { act, renderHook } from '@testing-library/react';

import type { I_GitAlias } from '#/types';

import { useAliasActions } from '#/hooks/useAliasActions';

vi.mock('#/lib/tauri', () => ({
    isTauri: false,
    tauriAPI: {
        importAliases: vi.fn(),
        exportAliases: vi.fn(),
        openLocalFolder: vi.fn(),
    },
}));

function createOptions(overrides: Partial<Parameters<typeof useAliasActions>[0]> = {}) {
    return {
        editingAlias: null as I_GitAlias | null,
        deletingAlias: null as I_GitAlias | null,
        aliases: [] as I_GitAlias[],
        addAlias: vi.fn().mockResolvedValue(undefined),
        updateAlias: vi.fn().mockResolvedValue(undefined),
        deleteAlias: vi.fn().mockResolvedValue(undefined),
        addToast: vi.fn(),
        clearDeletingAlias: vi.fn(),
        ...overrides,
    };
}

describe('useAliasActions', () => {
    it('handleSave calls addAlias for new alias', async () => {
        const opts = createOptions();
        const { result } = renderHook(() => useAliasActions(opts));

        await act(async () => {
            await result.current.handleSave('co', 'checkout', 'global');
        });

        expect(opts.addAlias).toHaveBeenCalledWith('co', 'checkout', 'global', undefined);
        expect(opts.addToast).toHaveBeenCalledWith('success', 'Alias "co" created');
    });

    it('handleSave calls updateAlias when editing', async () => {
        const editingAlias: I_GitAlias = { name: 'co', command: 'checkout', scope: 'global' };
        const opts = createOptions({ editingAlias });
        const { result } = renderHook(() => useAliasActions(opts));

        await act(async () => {
            await result.current.handleSave('co', 'checkout -b', 'global');
        });

        expect(opts.updateAlias).toHaveBeenCalledWith('co', 'co', 'checkout -b', 'global', undefined);
    });

    it('handleSave creates new local alias when scope changes global → local', async () => {
        const editingAlias: I_GitAlias = { name: 'co', command: 'checkout', scope: 'global' };
        const opts = createOptions({ editingAlias });
        const { result } = renderHook(() => useAliasActions(opts));

        await act(async () => {
            await result.current.handleSave('co', 'checkout', 'local', '/repo');
        });

        expect(opts.addAlias).toHaveBeenCalledWith('co', 'checkout', 'local', '/repo');
        expect(opts.addToast).toHaveBeenCalledWith('success', expect.stringContaining('local'));
    });

    it('handleConfirmDelete deletes and clears deletingAlias', async () => {
        const deletingAlias: I_GitAlias = { name: 'st', command: 'status', scope: 'global' };
        const opts = createOptions({ deletingAlias });
        const { result } = renderHook(() => useAliasActions(opts));

        await act(async () => {
            await result.current.handleConfirmDelete();
        });

        expect(opts.deleteAlias).toHaveBeenCalledWith('st', 'global', undefined);
        expect(opts.clearDeletingAlias).toHaveBeenCalled();
        expect(opts.addToast).toHaveBeenCalledWith('success', 'Alias "st" deleted');
    });

    it('handleConfirmDelete shows error toast on failure', async () => {
        const deletingAlias: I_GitAlias = { name: 'st', command: 'status', scope: 'global' };
        const deleteAlias = vi.fn().mockRejectedValue(new Error('Delete failed'));
        const opts = createOptions({ deletingAlias, deleteAlias });
        const { result } = renderHook(() => useAliasActions(opts));

        await act(async () => {
            await result.current.handleConfirmDelete();
        });

        expect(opts.addToast).toHaveBeenCalledWith('error', 'Delete failed');
    });

    it('handleConfirmDelete is no-op when deletingAlias is null', async () => {
        const opts = createOptions();
        const { result } = renderHook(() => useAliasActions(opts));

        await act(async () => {
            await result.current.handleConfirmDelete();
        });

        expect(opts.deleteAlias).not.toHaveBeenCalled();
    });

    it('handleImport shows info toast when not in Tauri', async () => {
        const opts = createOptions();
        const { result } = renderHook(() => useAliasActions(opts));

        await act(async () => {
            await result.current.handleImport();
        });

        expect(opts.addToast).toHaveBeenCalledWith('info', expect.stringContaining('desktop'));
    });

    it('handleExport creates download in browser mode', async () => {
        const aliases: I_GitAlias[] = [
            { name: 'co', command: 'checkout', scope: 'global' },
        ];
        const opts = createOptions({ aliases });
        const { result } = renderHook(() => useAliasActions(opts));

        // Mock URL and DOM APIs
        const mockCreateObjectURL = vi.fn().mockReturnValue('blob:test');
        const mockRevokeObjectURL = vi.fn();
        globalThis.URL.createObjectURL = mockCreateObjectURL;
        globalThis.URL.revokeObjectURL = mockRevokeObjectURL;

        const clickSpy = vi.fn();
        vi.spyOn(document, 'createElement').mockReturnValue({
            href: '',
            download: '',
            click: clickSpy,
        } as unknown as HTMLElement);

        await act(async () => {
            await result.current.handleExport();
        });

        expect(opts.addToast).toHaveBeenCalledWith('success', expect.stringContaining('1'));

        vi.restoreAllMocks();
    });

    it('exposes all handler functions', () => {
        const opts = createOptions();
        const { result } = renderHook(() => useAliasActions(opts));

        expect(typeof result.current.handleSave).toBe('function');
        expect(typeof result.current.handleConfirmDelete).toBe('function');
        expect(typeof result.current.handleImport).toBe('function');
        expect(typeof result.current.handleExport).toBe('function');
        expect(typeof result.current.handleOpenLocalFolder).toBe('function');
    });
});
