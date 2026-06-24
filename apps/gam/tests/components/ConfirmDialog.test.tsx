import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { ConfirmDialog } from '#/components/ConfirmDialog';

describe('confirmDialog', () => {
    const mockOnConfirm = vi.fn();
    const mockOnCancel = vi.fn();

    const defaultProps = {
        title: 'Delete Alias',
        message: 'Are you sure you want to delete this alias?',
        onConfirm: mockOnConfirm,
        onCancel: mockOnCancel,
    };

    it('renders title and message', () => {
        render(<ConfirmDialog {...defaultProps} />);
        expect(screen.getByText('Delete Alias')).toBeInTheDocument();
        expect(screen.getByText('Are you sure you want to delete this alias?')).toBeInTheDocument();
    });

    it('renders detail text when provided', () => {
        render(<ConfirmDialog {...defaultProps} detail="alias.co = checkout" />);
        expect(screen.getByText('alias.co = checkout')).toBeInTheDocument();
    });

    it('uses default confirm label "Confirm"', () => {
        render(<ConfirmDialog {...defaultProps} />);
        expect(screen.getByText('Confirm')).toBeInTheDocument();
    });

    it('uses custom confirm label', () => {
        render(<ConfirmDialog {...defaultProps} confirmLabel="Yes, Delete" />);
        expect(screen.getByText('Yes, Delete')).toBeInTheDocument();
    });

    it('calls onConfirm when confirm button is clicked', async () => {
        render(<ConfirmDialog {...defaultProps} />);
        const user = userEvent.setup();
        await user.click(screen.getByText('Confirm'));
        expect(mockOnConfirm).toHaveBeenCalled();
    });

    it('calls onCancel when Cancel button is clicked', async () => {
        render(<ConfirmDialog {...defaultProps} />);
        const user = userEvent.setup();
        await user.click(screen.getByText('Cancel'));
        expect(mockOnCancel).toHaveBeenCalled();
    });

    it('calls onCancel when close (✕) button is clicked', async () => {
        render(<ConfirmDialog {...defaultProps} />);
        const user = userEvent.setup();
        await user.click(screen.getByText('✕'));
        expect(mockOnCancel).toHaveBeenCalled();
    });

    it('calls onCancel when Escape key is pressed', () => {
        render(<ConfirmDialog {...defaultProps} />);
        fireEvent.keyDown(window, { key: 'Escape' });
        expect(mockOnCancel).toHaveBeenCalled();
    });

    it('calls onCancel when overlay backdrop is clicked', async () => {
        render(<ConfirmDialog {...defaultProps} />);
        const user = userEvent.setup();
        // The backdrop is the outermost div (the overlay)
        const backdrop = screen.getByText('Delete Alias').closest('.fixed');
        if (backdrop) {
            await user.click(backdrop);
            expect(mockOnCancel).toHaveBeenCalled();
        }
    });

    it('shows danger warning icon when confirmDanger is true', () => {
        render(<ConfirmDialog {...defaultProps} confirmDanger={true} />);
        expect(screen.getByText('⚠')).toBeInTheDocument();
    });
});
