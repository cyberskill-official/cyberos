import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { I_GitAlias } from '#/types';

import { ContentArea } from '#/components/ContentArea';

const mockAliases: I_GitAlias[] = [
    { name: 'co', command: 'checkout', scope: 'global' },
    { name: 'st', command: 'status', scope: 'global' },
];

describe('contentArea', () => {
    const defaultProps = {
        scope: 'all' as const,
        onScopeChange: vi.fn(),
        onAdd: vi.fn(),
        localPath: undefined,
        onSelectFolder: vi.fn(),
        onClearFolder: vi.fn(),
        searchQuery: '',
        onSearchChange: vi.fn(),
        filteredAliases: mockAliases,
        totalAliases: mockAliases,
        debouncedQuery: '',
        groupFilteredAliases: mockAliases,
        loading: false,
        error: null,
        fetchAliases: vi.fn(),
        onEdit: vi.fn(),
        onDelete: vi.fn(),
        onOpenLocalFolder: vi.fn(),
        groups: [],
        assignments: {},
        onSetAliasGroups: vi.fn(),
        onOpenExternal: vi.fn(),
    };

    it('renders toolbar and search bar', () => {
        render(<ContentArea {...defaultProps} />);
        expect(screen.getByText('New Alias')).toBeInTheDocument();
    });

    it('shows error banner when error is set', () => {
        render(<ContentArea {...defaultProps} error="Failed to load aliases" />);
        expect(screen.getByText('Failed to load aliases')).toBeInTheDocument();
        expect(screen.getByText('Retry')).toBeInTheDocument();
    });

    it('calls fetchAliases when retry button is clicked', async () => {
        const { default: userEvent } = await import('@testing-library/user-event');
        const user = userEvent.setup();
        render(<ContentArea {...defaultProps} error="Network error" />);
        await user.click(screen.getByText('Retry'));
        expect(defaultProps.fetchAliases).toHaveBeenCalled();
    });

    it('renders without error banner when no error', () => {
        render(<ContentArea {...defaultProps} />);
        expect(screen.queryByText('Retry')).not.toBeInTheDocument();
    });
});
