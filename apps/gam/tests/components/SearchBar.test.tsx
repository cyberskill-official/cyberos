import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { SearchBar } from '#/components/SearchBar';

describe('searchBar', () => {
    const mockOnChange = vi.fn();

    const defaultProps = {
        value: '',
        onChange: mockOnChange,
        resultCount: 0,
        totalCount: 10,
    };

    it('renders search input with placeholder', () => {
        render(<SearchBar {...defaultProps} />);
        expect(screen.getByPlaceholderText(/Search aliases/i)).toBeInTheDocument();
    });

    it('displays total count badge when not searching', () => {
        render(<SearchBar {...defaultProps} totalCount={42} />);
        expect(screen.getByText('42')).toBeInTheDocument();
    });

    it('calls onChange when typing', async () => {
        render(<SearchBar {...defaultProps} />);
        const user = userEvent.setup();
        const input = screen.getByPlaceholderText(/Search aliases/i);
        await user.type(input, 'checkout');
        expect(mockOnChange).toHaveBeenCalled();
    });

    it('shows clear button when value is non-empty', () => {
        render(<SearchBar {...defaultProps} value="test" />);
        expect(screen.getByLabelText('Clear search')).toBeInTheDocument();
    });

    it('does not show clear button when value is empty', () => {
        render(<SearchBar {...defaultProps} value="" />);
        expect(screen.queryByLabelText('Clear search')).not.toBeInTheDocument();
    });

    it('calls onChange with empty string when clear button is clicked', async () => {
        render(<SearchBar {...defaultProps} value="test" />);
        const user = userEvent.setup();
        await user.click(screen.getByLabelText('Clear search'));
        expect(mockOnChange).toHaveBeenCalledWith('');
    });

    it('has search role for accessibility', () => {
        render(<SearchBar {...defaultProps} />);
        expect(screen.getByRole('search')).toBeInTheDocument();
    });

    it('has proper aria-label', () => {
        render(<SearchBar {...defaultProps} />);
        expect(screen.getByLabelText('Search aliases')).toBeInTheDocument();
    });

    it('has search-input id for global keyboard shortcut targeting', () => {
        render(<SearchBar {...defaultProps} />);
        const input = screen.getByPlaceholderText(/Search aliases/i);
        expect(input).toHaveAttribute('id', 'search-input');
    });
});
