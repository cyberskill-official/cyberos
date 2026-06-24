import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { Toolbar } from '#/components/Toolbar';

describe('toolbar', () => {
    const mockHandlers = {
        onScopeChange: vi.fn(),
        onAdd: vi.fn(),
        onSelectFolder: vi.fn(),
        onClearFolder: vi.fn(),
    };

    const defaultProps = {
        scope: 'global' as const,
        ...mockHandlers,
    };

    it('renders scope tabs (All, Global, Local)', () => {
        render(<Toolbar {...defaultProps} />);
        expect(screen.getByText('All')).toBeInTheDocument();
        expect(screen.getByText('🌐 Global')).toBeInTheDocument();
        expect(screen.getByText('📁 Local')).toBeInTheDocument();
    });

    it('renders New Alias button', () => {
        render(<Toolbar {...defaultProps} />);
        expect(screen.getByText('New Alias')).toBeInTheDocument();
    });

    it('calls onScopeChange when switching tabs', async () => {
        render(<Toolbar {...defaultProps} />);
        const user = userEvent.setup();
        await user.click(screen.getByText('All'));
        expect(mockHandlers.onScopeChange).toHaveBeenCalledWith('all');
    });

    it('calls onAdd when New Alias is clicked', async () => {
        render(<Toolbar {...defaultProps} />);
        const user = userEvent.setup();
        await user.click(screen.getByText('New Alias'));
        expect(mockHandlers.onAdd).toHaveBeenCalled();
    });

    it('shows folder selector UI when scope is local', () => {
        render(<Toolbar {...defaultProps} scope="local" localPath="/my/repo" />);
        expect(screen.getByText('repo')).toBeInTheDocument();
    });

    it('does not show folder selector when scope is global', () => {
        render(<Toolbar {...defaultProps} scope="global" />);
        expect(screen.queryByTitle('Select folder')).not.toBeInTheDocument();
    });

    it('calls onClearFolder when clear button is clicked in local scope', async () => {
        render(<Toolbar {...defaultProps} scope="local" localPath="/my/repo" />);
        const user = userEvent.setup();
        const clearBtn = screen.getByTitle('Clear folder filter');
        await user.click(clearBtn);
        expect(mockHandlers.onClearFolder).toHaveBeenCalled();
    });

    it('calls onSelectFolder when folder icon is clicked', async () => {
        render(<Toolbar {...defaultProps} scope="local" />);
        const user = userEvent.setup();
        const selectBtn = screen.getByTitle('Select folder');
        await user.click(selectBtn);
        expect(mockHandlers.onSelectFolder).toHaveBeenCalled();
    });
});
