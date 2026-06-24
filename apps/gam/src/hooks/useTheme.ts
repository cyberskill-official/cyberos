import { useCallback, useEffect, useRef, useState } from 'react';

import type { I_ThemeConfig, T_ThemeId } from '#/types';

import { DEFAULT_THEME_ID, THEME_REGISTRY, THEME_STORAGE_KEY } from '#/lib/constants';
import { isTauri, tauriAPI } from '#/lib/tauri';

/** Parse a ThemeId string into style + mode, with fallback to default. */
function parseThemeId(raw: string | null | undefined): T_ThemeId {
    if (raw && raw in THEME_REGISTRY) {
        return raw as T_ThemeId;
    }

    return DEFAULT_THEME_ID;
}

/** Apply data-style and data-mode attributes to :root */
function applyTheme(themeId: T_ThemeId) {
    const config = THEME_REGISTRY[themeId];

    if (!config) {
        return;
    }
    document.documentElement.setAttribute('data-style', config.style);
    document.documentElement.setAttribute('data-mode', config.mode);
    document.documentElement.style.fontFamily = `var(--theme-font-sans)`;
}

export function useTheme() {
    const [themeIdState, setThemeIdState] = useState<T_ThemeId>(() => {
        const stored = localStorage.getItem(THEME_STORAGE_KEY);

        return parseThemeId(stored);
    });

    const [isPreview, setIsPreview] = useState(false);
    const committedRef = useRef<T_ThemeId>(themeIdState);

    // Load theme from Tauri backend on mount
    useEffect(() => {
        if (!isTauri) {
            return;
        }

        tauriAPI.getTheme().then((result) => {
            if (result.success && result.data) {
                const parsed = parseThemeId(result.data);
                committedRef.current = parsed;
                setThemeIdState(parsed);
            }
        }).catch(() => {
            // Fallback to localStorage value (already set in useState initializer)
        });
    }, []);

    // Apply theme whenever themeId changes
    useEffect(() => {
        applyTheme(themeIdState);

        // Only persist when it's not a preview
        if (!isPreview) {
            localStorage.setItem(THEME_STORAGE_KEY, themeIdState);
        }
    }, [themeIdState, isPreview]);

    /** Set theme permanently (persists to backend + localStorage) */
    const setThemeId = useCallback((id: T_ThemeId) => {
        committedRef.current = id;
        setThemeIdState(id);
        setIsPreview(false);
        localStorage.setItem(THEME_STORAGE_KEY, id);

        if (isTauri) {
            tauriAPI.setTheme(id).catch(() => {
                // Silent fail — localStorage is the fallback
            });
        }
    }, []);

    /** Temporarily preview a theme without persisting */
    const previewTheme = useCallback((id: T_ThemeId) => {
        setThemeIdState(id);
        setIsPreview(true);
    }, []);

    /** Cancel preview and revert to committed theme */
    const cancelPreview = useCallback(() => {
        setThemeIdState(committedRef.current);
        setIsPreview(false);
    }, []);

    const themeConfig: I_ThemeConfig = THEME_REGISTRY[themeIdState] ?? THEME_REGISTRY[DEFAULT_THEME_ID];

    return {
        themeId: themeIdState,
        themeConfig,
        setThemeId,
        previewTheme,
        cancelPreview,
        isPreview,
    };
}
