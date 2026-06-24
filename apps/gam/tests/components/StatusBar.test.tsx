import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { StatusBar } from '#/components/StatusBar';

// Mock the tauri-bridge so it doesn't try to import real Tauri APIs
vi.mock('#/lib/tauri', () => ({
    isTauri: false,
    tauriAPI: {
        openExternal: vi.fn(),
    },
}));

describe('statusBar', () => {
    const defaultProps = {
        totalAliases: 10,
        filteredCount: 5,
        scope: 'global',
        isSearching: false,
        onOpenExternal: vi.fn(),
    };

    it('renders scope label for global', () => {
        render(<StatusBar {...defaultProps} scope="global" />);
        expect(screen.getByText('Global')).toBeInTheDocument();
    });

    it('renders scope label for local', () => {
        render(<StatusBar {...defaultProps} scope="local" />);
        expect(screen.getByText('Local')).toBeInTheDocument();
    });

    it('renders scope label for all', () => {
        render(<StatusBar {...defaultProps} scope="all" />);
        expect(screen.getByText('All scopes')).toBeInTheDocument();
    });

    it('shows total alias count when not searching', () => {
        render(<StatusBar {...defaultProps} totalAliases={42} isSearching={false} />);
        expect(screen.getByText('42 aliases')).toBeInTheDocument();
    });

    it('shows singular "alias" for count of 1', () => {
        render(<StatusBar {...defaultProps} totalAliases={1} isSearching={false} />);
        expect(screen.getByText('1 alias')).toBeInTheDocument();
    });

    it('shows filtered count when searching', () => {
        render(<StatusBar {...defaultProps} totalAliases={20} filteredCount={3} isSearching={true} />);
        expect(screen.getByText('3 of 20')).toBeInTheDocument();
    });

    it('renders Buy Me A Banana button', () => {
        render(<StatusBar {...defaultProps} />);
        expect(screen.getByText(/Buy Me A Banana/)).toBeInTheDocument();
    });

    it('banana button has support title', () => {
        render(<StatusBar {...defaultProps} />);
        const btn = screen.getByTitle('Support GAM development');
        expect(btn).toBeInTheDocument();
    });
});
