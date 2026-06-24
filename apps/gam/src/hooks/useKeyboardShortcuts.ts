import { useEffect } from 'react';

interface I_KeyboardShortcutHandlers {
    onNewAlias: () => void;
    onFocusSearch: () => void;
    onCloseAll: () => void;
}

export function useKeyboardShortcuts({
    onNewAlias,
    onFocusSearch,
    onCloseAll,
}: I_KeyboardShortcutHandlers) {
    useEffect(() => {
        function handleKeyDown(e: KeyboardEvent) {
            const meta = e.metaKey || e.ctrlKey;

            // ⌘N / Ctrl+N → New alias
            if (meta && e.key === 'n') {
                e.preventDefault();
                onNewAlias();
                return;
            }

            // ⌘F / Ctrl+F → Focus search
            if (meta && e.key === 'f') {
                e.preventDefault();
                onFocusSearch();
                return;
            }

            // Escape → Close modals/forms
            if (e.key === 'Escape') {
                onCloseAll();
            }
        }

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [onNewAlias, onFocusSearch, onCloseAll]);
}
