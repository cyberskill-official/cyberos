import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import App from '#/App';

vi.mock('#/hooks/useAliases', () => ({
    useAliases: () => ({
        aliases: [{ name: 'co', command: 'checkout', scope: 'global' }],
        loading: false,
        error: null,
        scope: 'global',
        setScope: vi.fn(),
        fetchAliases: vi.fn(),
        addAlias: vi.fn(),
        updateAlias: vi.fn(),
        deleteAlias: vi.fn(),
        openLocalFolder: vi.fn(),
    }),
}));

vi.mock('#/hooks/useToast', () => ({
    useToast: () => ({
        toasts: [],
        addToast: vi.fn(),
        removeToast: vi.fn(),
    }),
}));

vi.mock('#/hooks/useTheme', () => ({
    useTheme: () => ({
        themeId: 'glassmorphism-dark',
        themeConfig: { id: 'glassmorphism-dark', style: 'glassmorphism', mode: 'dark', label: 'Glassmorphism Dark', description: '' },
        setThemeId: vi.fn(),
        previewTheme: vi.fn(),
        cancelPreview: vi.fn(),
        isPreview: false,
    }),
}));

describe('app', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders title and basic layout', () => {
        render(<App />);
        expect(screen.getByText(/GAM/)).toBeInTheDocument();
    });

    it('renders the mocked aliases', () => {
        render(<App />);
        expect(screen.getByText('co')).toBeInTheDocument();
        expect(screen.getByText('checkout')).toBeInTheDocument();
    });
});
