import '@testing-library/jest-dom';

// Some jsdom/vitest worker setups do not expose a writable Web Storage, so hook
// tests that persist (e.g. useTheme) crash on `localStorage.clear()`. Provide a
// deterministic in-memory shim — only when absent — so tests match the browser.
if (typeof globalThis.localStorage === 'undefined') {
    const store = new Map<string, string>();
    const shim: Storage = {
        getItem: k => (store.has(k) ? (store.get(k) as string) : null),
        setItem: (k, v) => {
            store.set(k, String(v));
        },
        removeItem: (k) => {
            store.delete(k);
        },
        clear: () => {
            store.clear();
        },
        key: i => Array.from(store.keys())[i] ?? null,
        get length() {
            return store.size;
        },
    } as Storage;
    Object.defineProperty(globalThis, 'localStorage', { value: shim, configurable: true, writable: true });
}
