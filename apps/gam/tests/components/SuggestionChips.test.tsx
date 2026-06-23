import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { I_AliasSuggestion } from '#/services/suggestion-service';

import { SuggestionChips } from '#/components/SuggestionChips';

function makeSuggestion(alias: string, scheme = 'abbreviation'): I_AliasSuggestion {
    return {
        alias,
        reason: 'Test reason',
        scheme,
    };
}

describe('suggestionChips', () => {
    const mockOnSelect = vi.fn();

    it('renders nothing when suggestions array is empty', () => {
        const { container } = render(<SuggestionChips suggestions={[]} onSelect={mockOnSelect} />);
        expect(container.innerHTML).toBe('');
    });

    it('renders a chip for each suggestion', () => {
        const suggestions = [makeSuggestion('gco'), makeSuggestion('gst'), makeSuggestion('gpl')];
        render(<SuggestionChips suggestions={suggestions} onSelect={mockOnSelect} />);
        expect(screen.getByText('gco')).toBeInTheDocument();
        expect(screen.getByText('gst')).toBeInTheDocument();
        expect(screen.getByText('gpl')).toBeInTheDocument();
    });

    it('shows the "Suggested names:" label', () => {
        const suggestions = [makeSuggestion('gco')];
        render(<SuggestionChips suggestions={suggestions} onSelect={mockOnSelect} />);
        expect(screen.getByText(/Suggested names/i)).toBeInTheDocument();
    });

    it('calls onSelect with the alias when a chip is clicked', async () => {
        const user = userEvent.setup();
        const suggestions = [makeSuggestion('gco'), makeSuggestion('gst')];
        render(<SuggestionChips suggestions={suggestions} onSelect={mockOnSelect} />);

        await user.click(screen.getByText('gco'));
        expect(mockOnSelect).toHaveBeenCalledWith('gco');
    });

    it('shows scheme and reason in tooltip', () => {
        const suggestions = [makeSuggestion('gco', 'abbreviation')];
        render(<SuggestionChips suggestions={suggestions} onSelect={mockOnSelect} />);
        const chip = screen.getByText('gco').closest('button');
        expect(chip).toHaveAttribute('title', 'Test reason (abbreviation)');
    });
});
