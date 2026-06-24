/**
 * Platform detection and keyboard shortcut utilities.
 * Provides platform-appropriate modifier key display (⌘ vs Ctrl).
 */
const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad|iPod/.test(navigator.platform ?? '');
const isWindows = typeof navigator !== 'undefined' && /Win/.test(navigator.platform ?? '');

export const platform = {
    isMac,
    isWindows,
    isLinux: !isMac && !isWindows,

    /** Primary modifier key label for shortcuts */
    mod: isMac ? '⌘' : 'Ctrl',

    /** Alt/Option label */
    alt: isMac ? '⌥' : 'Alt',

    /** Shift label */
    shift: '⇧',

    /** Format a shortcut for display, e.g. platform.shortcut('S') → "⌘S" or "Ctrl+S" */
    shortcut(key: string): string {
        return isMac ? `${this.mod}${key}` : `${this.mod}+${key}`;
    },
};
