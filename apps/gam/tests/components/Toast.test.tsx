import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { ToastContainer } from '#/components/Toast';

describe('toastContainer', () => {
    const mockOnDismiss = vi.fn();

    it('renders nothing when toasts array is empty', () => {
        const { container } = render(<ToastContainer toasts={[]} onDismiss={mockOnDismiss} />);
        expect(container.firstChild).toBeNull();
    });

    it('renders a success toast', () => {
        const toasts = [{ id: '1', type: 'success' as const, message: 'Alias created!' }];
        render(<ToastContainer toasts={toasts} onDismiss={mockOnDismiss} />);
        expect(screen.getByText('Alias created!')).toBeInTheDocument();
        expect(screen.getByText('✓')).toBeInTheDocument();
    });

    it('renders an error toast', () => {
        const toasts = [{ id: '1', type: 'error' as const, message: 'Failed to save' }];
        render(<ToastContainer toasts={toasts} onDismiss={mockOnDismiss} />);
        expect(screen.getByText('Failed to save')).toBeInTheDocument();
        expect(screen.getByText('✗')).toBeInTheDocument();
    });

    it('renders a warning toast', () => {
        const toasts = [{ id: '1', type: 'warning' as const, message: 'Dangerous command' }];
        render(<ToastContainer toasts={toasts} onDismiss={mockOnDismiss} />);
        expect(screen.getByText('Dangerous command')).toBeInTheDocument();
        expect(screen.getByText('⚠')).toBeInTheDocument();
    });

    it('renders an info toast', () => {
        const toasts = [{ id: '1', type: 'info' as const, message: 'Tip of the day' }];
        render(<ToastContainer toasts={toasts} onDismiss={mockOnDismiss} />);
        expect(screen.getByText('Tip of the day')).toBeInTheDocument();
        expect(screen.getByText('ℹ')).toBeInTheDocument();
    });

    it('renders multiple toasts', () => {
        const toasts = [
            { id: '1', type: 'success' as const, message: 'First' },
            { id: '2', type: 'error' as const, message: 'Second' },
        ];
        render(<ToastContainer toasts={toasts} onDismiss={mockOnDismiss} />);
        expect(screen.getByText('First')).toBeInTheDocument();
        expect(screen.getByText('Second')).toBeInTheDocument();
    });

    it('calls onDismiss when dismiss button is clicked', async () => {
        const toasts = [{ id: 'toast-1', type: 'success' as const, message: 'Done!' }];
        render(<ToastContainer toasts={toasts} onDismiss={mockOnDismiss} />);
        const user = userEvent.setup();
        await user.click(screen.getByText('✕'));
        expect(mockOnDismiss).toHaveBeenCalledWith('toast-1');
    });

    it('applies exiting class when toast is exiting', () => {
        const toasts = [{ id: '1', type: 'success' as const, message: 'Bye', exiting: true }];
        render(<ToastContainer toasts={toasts} onDismiss={mockOnDismiss} />);
        const toast = screen.getByText('Bye').closest('[class*="opacity-0"]');
        expect(toast).toBeTruthy();
    });
});
