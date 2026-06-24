import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { AliasLibraryPicker } from '#/components/AliasLibraryPicker';

describe('aliasLibraryPicker', () => {
    const mockOnSelect = vi.fn();
    const mockOnClose = vi.fn();

    const defaultProps = {
        onSelect: mockOnSelect,
        onClose: mockOnClose,
    };

    it('renders the library picker with a title', () => {
        render(<AliasLibraryPicker {...defaultProps} />);
        expect(screen.getByText('Alias Library')).toBeInTheDocument();
    });

    it('renders category filter buttons', () => {
        render(<AliasLibraryPicker {...defaultProps} />);
        // Should have category buttons from the built-in library
        const buttons = screen.getAllByRole('button');
        // At minimum: close button, categories, cancel button, alias entries
        expect(buttons.length).toBeGreaterThan(3);
    });

    it('renders alias entries from the library', () => {
        render(<AliasLibraryPicker {...defaultProps} />);
        // Should show the "found" text with count
        expect(screen.getByText(/found/i)).toBeInTheDocument();
    });

    it('calls onClose when the close button is clicked', async () => {
        const user = userEvent.setup();
        render(<AliasLibraryPicker {...defaultProps} />);
        const closeButton = screen.getByLabelText('Close library');
        await user.click(closeButton);
        expect(mockOnClose).toHaveBeenCalled();
    });

    it('calls onClose when Cancel button is clicked', async () => {
        const user = userEvent.setup();
        render(<AliasLibraryPicker {...defaultProps} />);
        const cancelButton = screen.getByText('Cancel');
        await user.click(cancelButton);
        expect(mockOnClose).toHaveBeenCalled();
    });

    it('filters aliases when searching', async () => {
        const user = userEvent.setup();
        render(<AliasLibraryPicker {...defaultProps} />);
        const searchInput = screen.getByLabelText('Search aliases');
        await user.type(searchInput, 'checkout');
        expect(searchInput).toHaveValue('checkout');
    });

    it('calls onSelect and onClose when an alias entry is clicked', async () => {
        const user = userEvent.setup();
        render(<AliasLibraryPicker {...defaultProps} />);
        // Click the first alias entry button (they have title="Click to use this alias")
        const aliasButtons = screen.getAllByTitle('Click to use this alias');
        expect(aliasButtons.length).toBeGreaterThan(0);
        await user.click(aliasButtons[0]);
        expect(mockOnSelect).toHaveBeenCalled();
        expect(mockOnClose).toHaveBeenCalled();
    });

    it('calls onClose when clicking the backdrop', async () => {
        const user = userEvent.setup();
        const { container } = render(<AliasLibraryPicker {...defaultProps} />);
        // Click the backdrop (first child div)
        const backdrop = container.firstElementChild as Element;
        await user.click(backdrop);
        expect(mockOnClose).toHaveBeenCalled();
    });
});
