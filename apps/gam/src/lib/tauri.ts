/**
 * Tauri environment detection & API re-export.
 *
 * Import `isTauri` and `tauriAPI` from here instead of duplicating
 * the detection logic across multiple files.
 */
export { tauriAPI } from '#/tauri-bridge';

export const isTauri
    = typeof window !== 'undefined' && window.__TAURI_INTERNALS__ !== undefined;
