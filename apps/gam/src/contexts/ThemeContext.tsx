import { createContext, use } from 'react';

import type { I_ThemeConfig, T_ThemeId } from '#/types';

import { useTheme } from '#/hooks/useTheme';

interface I_ThemeContextValue {
    themeId: T_ThemeId;
    themeConfig: I_ThemeConfig;
    setThemeId: (id: T_ThemeId) => void;
    previewTheme: (id: T_ThemeId) => void;
    cancelPreview: () => void;
    isPreview: boolean;
}

const ThemeContext = createContext<I_ThemeContextValue | null>(null);

export function ThemeProvider({ children }: { children: React.ReactNode }) {
    const theme = useTheme();

    return (
        <ThemeContext value={theme}>
            {children}
        </ThemeContext>
    );
}

export function useThemeContext() {
    const ctx = use(ThemeContext);
    if (!ctx) {
        throw new Error('useThemeContext must be used within <ThemeProvider>');
    }
    return ctx;
}
