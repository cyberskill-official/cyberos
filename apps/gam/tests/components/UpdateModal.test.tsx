import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { UpdateModal } from '#/components/UpdateModal';

describe('updateModal', () => {
    const mockOnUpdate = vi.fn();
    const mockOnDismiss = vi.fn();

    const defaultProps = {
        version: '2.0.0',
        changelog: 'Bug fixes and improvements',
        downloading: false,
        downloadProgress: 0,
        error: null,
        onUpdate: mockOnUpdate,
        onDismiss: mockOnDismiss,
    };

    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders version and changelog', () => {
        render(<UpdateModal {...defaultProps} />);
        expect(screen.getByText('v2.0.0')).toBeInTheDocument();
        expect(screen.getByText('Bug fixes and improvements')).toBeInTheDocument();
    });

    it('renders "Update Available" title', () => {
        render(<UpdateModal {...defaultProps} />);
        expect(screen.getByText('Update Available')).toBeInTheDocument();
    });

    it('calls onUpdate when "Update Now" is clicked', async () => {
        render(<UpdateModal {...defaultProps} />);
        const user = userEvent.setup();
        await user.click(screen.getByText('Update Now'));
        expect(mockOnUpdate).toHaveBeenCalled();
    });

    it('calls onDismiss when "Later" is clicked', async () => {
        render(<UpdateModal {...defaultProps} />);
        const user = userEvent.setup();
        await user.click(screen.getByText('Later'));
        expect(mockOnDismiss).toHaveBeenCalled();
    });

    it('calls onDismiss when close button is clicked', async () => {
        render(<UpdateModal {...defaultProps} />);
        const user = userEvent.setup();
        await user.click(screen.getByText('✕'));
        expect(mockOnDismiss).toHaveBeenCalled();
    });

    it('calls onDismiss on Escape key', () => {
        render(<UpdateModal {...defaultProps} />);
        fireEvent.keyDown(window, { key: 'Escape' });
        expect(mockOnDismiss).toHaveBeenCalled();
    });

    it('shows progress bar when downloading', () => {
        render(<UpdateModal {...defaultProps} downloading={true} downloadProgress={45} />);
        expect(screen.getByText('Downloading update…')).toBeInTheDocument();
        expect(screen.getByText('45%')).toBeInTheDocument();
    });

    it('disables Update Now button during download', () => {
        render(<UpdateModal {...defaultProps} downloading={true} downloadProgress={50} />);
        expect(screen.getByText('Installing…')).toBeDisabled();
    });

    it('hides Later button during download', () => {
        render(<UpdateModal {...defaultProps} downloading={true} downloadProgress={50} />);
        expect(screen.queryByText('Later')).not.toBeInTheDocument();
    });

    it('does not dismiss on Escape during download', () => {
        render(<UpdateModal {...defaultProps} downloading={true} downloadProgress={50} />);
        fireEvent.keyDown(window, { key: 'Escape' });
        expect(mockOnDismiss).not.toHaveBeenCalled();
    });

    it('shows error message when error is present', () => {
        render(<UpdateModal {...defaultProps} error="Network timeout" />);
        expect(screen.getByText('Update failed: Network timeout')).toBeInTheDocument();
    });

    it('renders without changelog', () => {
        render(<UpdateModal {...defaultProps} changelog="" />);
        expect(screen.getByText('v2.0.0')).toBeInTheDocument();
        expect(screen.getByText('Update Now')).toBeInTheDocument();
    });
});
