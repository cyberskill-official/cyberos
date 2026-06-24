import { renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { useKeyboardShortcuts } from '#/hooks/useKeyboardShortcuts';

vi.mock('#/lib/tauri', () => ({
    isTauri: false,
    tauriAPI: {},
}));

describe('useKeyboardShortcuts', () => {
    const createHandlers = () => ({
        onNewAlias: vi.fn(),
        onFocusSearch: vi.fn(),
        onCloseAll: vi.fn(),
    });

    const fireKey = (key: string, options: Partial<KeyboardEventInit> = {}) => {
        window.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true, ...options }));
    };

    it('calls onNewAlias on Cmd+N', () => {
        const handlers = createHandlers();
        renderHook(() => useKeyboardShortcuts(handlers));
        fireKey('n', { metaKey: true });
        expect(handlers.onNewAlias).toHaveBeenCalledTimes(1);
    });

    it('calls onFocusSearch on Cmd+F', () => {
        const handlers = createHandlers();
        renderHook(() => useKeyboardShortcuts(handlers));
        fireKey('f', { metaKey: true });
        expect(handlers.onFocusSearch).toHaveBeenCalledTimes(1);
    });

    it('calls onCloseAll on Escape', () => {
        const handlers = createHandlers();
        renderHook(() => useKeyboardShortcuts(handlers));
        fireKey('Escape');
        expect(handlers.onCloseAll).toHaveBeenCalledTimes(1);
    });

    it('does not call onNewAlias without meta key', () => {
        const handlers = createHandlers();
        renderHook(() => useKeyboardShortcuts(handlers));
        fireKey('n');
        expect(handlers.onNewAlias).not.toHaveBeenCalled();
    });

    it('cleans up listener on unmount', () => {
        const handlers = createHandlers();
        const { unmount } = renderHook(() => useKeyboardShortcuts(handlers));
        unmount();
        fireKey('n', { metaKey: true });
        expect(handlers.onNewAlias).not.toHaveBeenCalled();
    });
});
