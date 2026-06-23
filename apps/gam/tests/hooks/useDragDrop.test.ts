import { renderHook } from '@testing-library/react';

import { useDragDrop } from '#/hooks/useDragDrop';

vi.mock('#/lib/tauri', () => ({
    isTauri: false,
    tauriAPI: {
        setLocalPath: vi.fn().mockResolvedValue({ success: true }),
    },
}));

describe('useDragDrop', () => {
    let addEventListenerSpy: ReturnType<typeof vi.spyOn>;
    let removeEventListenerSpy: ReturnType<typeof vi.spyOn>;

    beforeEach(() => {
        addEventListenerSpy = vi.spyOn(window, 'addEventListener');
        removeEventListenerSpy = vi.spyOn(window, 'removeEventListener');
    });

    afterEach(() => {
        addEventListenerSpy.mockRestore();
        removeEventListenerSpy.mockRestore();
    });

    it('registers drop and dragover listeners on mount', () => {
        const setLocalPath = vi.fn();
        const fetchAliases = vi.fn().mockResolvedValue(undefined);
        const addToast = vi.fn();

        renderHook(() => useDragDrop(setLocalPath, fetchAliases, addToast));

        expect(addEventListenerSpy).toHaveBeenCalledWith('drop', expect.any(Function));
        expect(addEventListenerSpy).toHaveBeenCalledWith('dragover', expect.any(Function));
    });

    it('removes listeners on unmount', () => {
        const setLocalPath = vi.fn();
        const fetchAliases = vi.fn().mockResolvedValue(undefined);
        const addToast = vi.fn();

        const { unmount } = renderHook(() => useDragDrop(setLocalPath, fetchAliases, addToast));
        unmount();

        expect(removeEventListenerSpy).toHaveBeenCalledWith('drop', expect.any(Function));
        expect(removeEventListenerSpy).toHaveBeenCalledWith('dragover', expect.any(Function));
    });
});
