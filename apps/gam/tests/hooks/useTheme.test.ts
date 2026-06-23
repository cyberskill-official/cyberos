import { act, renderHook } from '@testing-library/react';

import { useTheme } from '#/hooks/useTheme';

const mockGetTheme = vi.fn();
const mockSetTheme = vi.fn();

vi.mock('#/lib/tauri', () => ({
    isTauri: false,
    tauriAPI: {
        getTheme: () => mockGetTheme(),
        setTheme: (id: string) => mockSetTheme(id),
    },
}));

describe('useTheme', () => {
    beforeEach(() => {
        localStorage.clear();
        document.documentElement.removeAttribute('data-style');
        document.documentElement.removeAttribute('data-mode');
        mockGetTheme.mockReset();
        mockSetTheme.mockReset();
    });

    it('defaults to glassmorphism-dark', () => {
        const { result } = renderHook(() => useTheme());
        expect(result.current.themeId).toBe('glassmorphism-dark');
    });

    it('reads stored theme from localStorage', () => {
        localStorage.setItem('gam-theme', 'cybercore-dark');
        const { result } = renderHook(() => useTheme());
        expect(result.current.themeId).toBe('cybercore-dark');
    });

    it('falls back to default for invalid stored theme', () => {
        localStorage.setItem('gam-theme', 'invalid-theme');
        const { result } = renderHook(() => useTheme());
        expect(result.current.themeId).toBe('glassmorphism-dark');
    });

    it('setThemeId changes theme and persists to localStorage', () => {
        const { result } = renderHook(() => useTheme());

        act(() => {
            result.current.setThemeId('sketch-dark');
        });

        expect(result.current.themeId).toBe('sketch-dark');
        expect(localStorage.getItem('gam-theme')).toBe('sketch-dark');
    });

    it('applies data-style and data-mode to document', () => {
        const { result } = renderHook(() => useTheme());

        act(() => {
            result.current.setThemeId('baroque-light');
        });

        expect(document.documentElement.getAttribute('data-style')).toBe('baroque');
        expect(document.documentElement.getAttribute('data-mode')).toBe('light');
    });

    it('previewTheme changes display without persisting', () => {
        const { result } = renderHook(() => useTheme());

        act(() => {
            result.current.setThemeId('sketch-dark');
        });

        act(() => {
            result.current.previewTheme('gothic-dark');
        });

        expect(result.current.themeId).toBe('gothic-dark');
        expect(result.current.isPreview).toBe(true);
        // localStorage should still have the committed theme
        expect(localStorage.getItem('gam-theme')).toBe('sketch-dark');
    });

    it('cancelPreview reverts to committed theme', () => {
        const { result } = renderHook(() => useTheme());

        act(() => {
            result.current.setThemeId('pixel-dark');
        });

        act(() => {
            result.current.previewTheme('gothic-dark');
        });

        act(() => {
            result.current.cancelPreview();
        });

        expect(result.current.themeId).toBe('pixel-dark');
        expect(result.current.isPreview).toBe(false);
    });

    it('themeConfig matches the current themeId', () => {
        const { result } = renderHook(() => useTheme());
        expect(result.current.themeConfig.id).toBe('glassmorphism-dark');
        expect(result.current.themeConfig.style).toBe('glassmorphism');
        expect(result.current.themeConfig.mode).toBe('dark');
    });
});
