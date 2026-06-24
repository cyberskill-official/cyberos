import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { AliasForm } from '#/components/AliasForm';

describe('aliasForm', () => {
    const mockOnSave = vi.fn();
    const mockOnClose = vi.fn();

    it('renders empty form for new alias', () => {
        render(<AliasForm onSave={mockOnSave} onClose={mockOnClose} currentScope="global" />);
        expect(screen.getByText('New Alias')).toBeInTheDocument();
    });

    it('shows validation errors for empty fields', async () => {
        render(<AliasForm onSave={mockOnSave} onClose={mockOnClose} currentScope="global" />);

        fireEvent.click(screen.getByRole('button', { name: /Create Alias/i }));

        expect(screen.getByText(/Alias name is required/i)).toBeInTheDocument();
        expect(screen.getByText(/Command is required/i)).toBeInTheDocument();
        expect(mockOnSave).not.toHaveBeenCalled();
    });

    it('calls onSave with correct data when valid', async () => {
        render(<AliasForm onSave={mockOnSave} onClose={mockOnClose} currentScope="global" />);

        const user = userEvent.setup();

        const commandInput = screen.getByPlaceholderText(/e.g. checkout, status/i);
        const nameInput = screen.getByPlaceholderText(/e.g. co/i);

        await user.type(commandInput, 'checkout');
        await user.type(nameInput, 'co');

        const submitBtn = screen.getByRole('button', { name: /Create Alias/i });
        await user.click(submitBtn);

        await waitFor(() => {
            expect(mockOnSave).toHaveBeenCalledWith('co', 'checkout', 'global', undefined);
        });
    });

    it('shows dangerous command warning', async () => {
        render(<AliasForm onSave={mockOnSave} onClose={mockOnClose} currentScope="global" />);
        const user = userEvent.setup();
        const commandInput = screen.getByPlaceholderText(/e.g. checkout, status/i);

        await user.type(commandInput, 'rm -rf');
        expect(screen.getByText(/Contains recursive delete/i)).toBeInTheDocument();
    });
    describe('browse Library button', () => {
        it('should render Browse Library button', () => {
            render(<AliasForm onSave={mockOnSave} onClose={mockOnClose} currentScope="global" />);
            const btn = screen.getByText(/browse alias library/i);
            expect(btn).toBeInTheDocument();
        });

        it('should open library picker when clicked', async () => {
            render(<AliasForm onSave={mockOnSave} onClose={mockOnClose} currentScope="global" />);
            const user = userEvent.setup();
            await user.click(screen.getByText(/browse alias library/i));
            expect(screen.getByText('Alias Library')).toBeInTheDocument();
            expect(screen.getByPlaceholderText(/search aliases/i)).toBeInTheDocument();
        });

        it('should close library picker via close button', async () => {
            render(<AliasForm onSave={mockOnSave} onClose={mockOnClose} currentScope="global" />);
            const user = userEvent.setup();
            await user.click(screen.getByText(/browse alias library/i));
            const closeButtons = screen.getAllByLabelText(/close/i);
            await user.click(closeButtons[closeButtons.length - 1]);
            expect(screen.queryByPlaceholderText(/search aliases/i)).not.toBeInTheDocument();
        });
    });

    describe('library alias selection', () => {
        it('should populate form fields when alias is selected', async () => {
            render(<AliasForm onSave={mockOnSave} onClose={mockOnClose} currentScope="global" />);
            const user = userEvent.setup();
            await user.click(screen.getByText(/browse alias library/i));

            const searchInput = screen.getByPlaceholderText(/search aliases/i);
            await user.type(searchInput, 'checkout');

            const coButton = screen.getByText('co').closest('button');
            expect(coButton).toBeTruthy();
            await user.click(coButton!);

            const nameInput = screen.getByPlaceholderText(/e.g. co/i) as HTMLInputElement;
            const commandInput = screen.getByPlaceholderText(/e.g. checkout, status/i) as HTMLTextAreaElement;
            expect(nameInput.value).toBe('co');
            expect(commandInput.value).toBe('checkout');
        });

        it('should allow user to customize alias name after selection', async () => {
            render(<AliasForm onSave={mockOnSave} onClose={mockOnClose} currentScope="global" />);
            const user = userEvent.setup();
            await user.click(screen.getByText(/browse alias library/i));

            const searchInput = screen.getByPlaceholderText(/search aliases/i);
            await user.type(searchInput, 'checkout');
            const coButton = screen.getByText('co').closest('button');
            await user.click(coButton!);

            const nameInput = screen.getByPlaceholderText(/e.g. co/i) as HTMLInputElement;
            await user.clear(nameInput);
            await user.type(nameInput, 'myco');
            expect(nameInput.value).toBe('myco');
        });
    });

    describe('command textarea', () => {
        it('should render command as textarea', () => {
            render(<AliasForm onSave={mockOnSave} onClose={mockOnClose} currentScope="global" />);
            const textarea = screen.getByPlaceholderText(/e.g. checkout, status/i);
            expect(textarea.tagName.toLowerCase()).toBe('textarea');
        });
    });

    describe('edit mode with library', () => {
        it('should show Browse Library button in edit mode', () => {
            const editAlias = { name: 'oldname', command: 'oldcmd', scope: 'global' as const };
            render(<AliasForm onSave={mockOnSave} onClose={mockOnClose} currentScope="global" alias={editAlias} />);
            expect(screen.getByText(/browse alias library/i)).toBeInTheDocument();
        });

        it('should allow overwriting fields from library in edit mode', async () => {
            const editAlias = { name: 'oldname', command: 'oldcmd', scope: 'global' as const };
            render(<AliasForm onSave={mockOnSave} onClose={mockOnClose} currentScope="global" alias={editAlias} />);
            const user = userEvent.setup();

            await user.click(screen.getByText(/browse alias library/i));
            const searchInput = screen.getByPlaceholderText(/search aliases/i);
            await user.type(searchInput, 'checkout');
            const coButton = screen.getByText('co').closest('button');
            await user.click(coButton!);

            const commandInput = screen.getByPlaceholderText(/e.g. checkout, status/i) as HTMLTextAreaElement;
            expect(commandInput.value).toBe('checkout');
        });

        it('should allow editing command after library selection', async () => {
            render(<AliasForm onSave={mockOnSave} onClose={mockOnClose} currentScope="global" />);
            const user = userEvent.setup();

            await user.click(screen.getByText(/browse alias library/i));
            const searchInput = screen.getByPlaceholderText(/search aliases/i);
            await user.type(searchInput, 'checkout');
            const coButton = screen.getByText('co').closest('button');
            await user.click(coButton!);

            const commandInput = screen.getByPlaceholderText(/e.g. checkout, status/i) as HTMLTextAreaElement;
            await user.clear(commandInput);
            await user.type(commandInput, 'checkout --no-guess');
            expect(commandInput.value).toBe('checkout --no-guess');
        });
    });
});
