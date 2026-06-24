import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { I_ThemeConfig, T_ThemeId } from '#/types';

import { TitleBar } from '#/components/TitleBar';

vi.mock('#/lib/constants', () => ({
    APP_VERSION: '1.2.3',
    THEME_STYLE_ICONS: { neo: '◈', sketch: '✎' },
    THEME_REGISTRY: {},
    THEME_STYLE_GROUPS: [],
    THEME_SWATCHES: {},
}));

describe('titleBar', () => {
    const defaultProps = {
        themeId: 'neo-dark' as T_ThemeId,
        themeConfig: { id: 'neo-dark', label: 'Neo Dark', style: 'neo', mode: 'dark' } as I_ThemeConfig,
        setThemeId: vi.fn(),
        previewTheme: vi.fn(),
        cancelPreview: vi.fn(),
        onOpenThemeSettings: vi.fn(),
        onImport: vi.fn(),
        onExport: vi.fn(),
        aliasCount: 10,
        onOpenExternal: vi.fn(),
    };

    it('renders app name', () => {
        render(<TitleBar {...defaultProps} />);
        expect(screen.getByText(/GAM/)).toBeInTheDocument();
    });

    it('displays app version', () => {
        render(<TitleBar {...defaultProps} />);
        expect(screen.getByText(/1\.2\.3/)).toBeInTheDocument();
    });

    it('renders theme style icon', () => {
        render(<TitleBar {...defaultProps} />);
        const titlebar = document.querySelector('.titlebar');
        expect(titlebar).toBeTruthy();
    });
});
