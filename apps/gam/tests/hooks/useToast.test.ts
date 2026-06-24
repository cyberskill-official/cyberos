import { act, renderHook } from '@testing-library/react';

import { useToast } from '#/hooks/useToast';

vi.mock('#/lib/tauri', () => ({
    isTauri: false,
    tauriAPI: {},
}));

describe('useToast', () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it('starts with empty toasts', () => {
        const { result } = renderHook(() => useToast());
        expect(result.current.toasts).toEqual([]);
    });

    it('adds a toast with correct fields', () => {
        const { result } = renderHook(() => useToast());

        act(() => {
            result.current.addToast('success', 'Alias created');
        });

        expect(result.current.toasts).toHaveLength(1);
        expect(result.current.toasts[0]).toMatchObject({
            type: 'success',
            message: 'Alias created',
        });
        expect(result.current.toasts[0].id).toMatch(/^toast-/);
    });

    it('adds multiple toasts', () => {
        const { result } = renderHook(() => useToast());

        act(() => {
            result.current.addToast('success', 'First');
            result.current.addToast('error', 'Second');
        });

        expect(result.current.toasts).toHaveLength(2);
        expect(result.current.toasts[0].type).toBe('success');
        expect(result.current.toasts[1].type).toBe('error');
    });

    it('assigns unique IDs', () => {
        const { result } = renderHook(() => useToast());

        act(() => {
            result.current.addToast('info', 'A');
            result.current.addToast('info', 'B');
        });

        expect(result.current.toasts[0].id).not.toBe(result.current.toasts[1].id);
    });

    it('auto-dismisses toast after 4 seconds', () => {
        const { result } = renderHook(() => useToast());

        act(() => {
            result.current.addToast('success', 'Will vanish');
        });

        expect(result.current.toasts).toHaveLength(1);

        // Mark as exiting at 4s
        act(() => {
            vi.advanceTimersByTime(4000);
        });

        expect(result.current.toasts[0]?.exiting).toBe(true);

        // Fully removed after exit animation (200ms)
        act(() => {
            vi.advanceTimersByTime(200);
        });

        expect(result.current.toasts).toHaveLength(0);
    });

    it('removeToast marks as exiting then removes', () => {
        const { result } = renderHook(() => useToast());

        act(() => {
            result.current.addToast('warning', 'Manual close');
        });

        const id = result.current.toasts[0].id;

        act(() => {
            result.current.removeToast(id);
        });

        expect(result.current.toasts[0]?.exiting).toBe(true);

        act(() => {
            vi.advanceTimersByTime(200);
        });

        expect(result.current.toasts).toHaveLength(0);
    });

    it('supports all toast types', () => {
        const { result } = renderHook(() => useToast());
        const types = ['success', 'error', 'warning', 'info'] as const;

        act(() => {
            for (const type of types) {
                result.current.addToast(type, `${type} message`);
            }
        });

        expect(result.current.toasts).toHaveLength(4);
        types.forEach((type, i) => {
            expect(result.current.toasts[i].type).toBe(type);
        });
    });
});
