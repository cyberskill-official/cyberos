import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { T_ThemeId } from '#/types';

import { SettingsDropdown } from '#/components/SettingsDropdown';

vi.mock('#/lib/constants', () => ({
    APP_VERSION: '1.0.0',
    THEME_REGISTRY: {
        'neo-dark': { id: 'neo-dark', label: 'Neo Dark', style: 'neo', mode: 'dark' },
    },
    THEME_STYLE_GROUPS: [{ style: 'neo', label: 'Neo' }],
    THEME_STYLE_ICONS: { neo: '◈' },
    THEME_SWATCHES: {
        'neo-dark': { bg: '#111', surface: '#222', primary: '#3b82f6' },
    },
}));

describe('settingsDropdown', () => {
    const defaultProps = {
        themeId: 'neo-dark' as T_ThemeId,
        onSelectTheme: vi.fn(),
        onPreviewTheme: vi.fn(),
        onCancelPreview: vi.fn(),
        onOpenThemeSettings: vi.fn(),
        onImport: vi.fn(),
        onExport: vi.fn(),
        aliasCount: 5,
        onOpenExternal: vi.fn(),
    };

    it('renders theme, data, and about buttons', () => {
        render(<SettingsDropdown {...defaultProps} />);
        expect(screen.getByText('🎨')).toBeInTheDocument();
        expect(screen.getByText('📋')).toBeInTheDocument();
        expect(screen.getByText('ℹ️')).toBeInTheDocument();
    });

    it('opens data panel when data button is clicked', async () => {
        const user = userEvent.setup();
        render(<SettingsDropdown {...defaultProps} />);
        await user.click(screen.getByText('📋'));
        expect(screen.getByText('Import JSON')).toBeInTheDocument();
        expect(screen.getByText('Export JSON')).toBeInTheDocument();
    });

    it('opens about panel when about button is clicked', async () => {
        const user = userEvent.setup();
        render(<SettingsDropdown {...defaultProps} />);
        await user.click(screen.getByText('ℹ️'));
        expect(screen.getByText('GAM')).toBeInTheDocument();
        expect(screen.getByText('Git Alias Manager')).toBeInTheDocument();
    });

    it('disables export when aliasCount is 0', async () => {
        const user = userEvent.setup();
        render(<SettingsDropdown {...defaultProps} aliasCount={0} />);
        await user.click(screen.getByText('📋'));
        const exportBtn = screen.getByText('Export JSON').closest('button');
        expect(exportBtn).toBeDisabled();
    });
});
