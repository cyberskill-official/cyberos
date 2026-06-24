import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { AliasList } from '#/components/AliasList';

describe('aliasList', () => {
    const mockAliases = [
        { name: 'co', command: 'checkout', scope: 'global' as const },
        { name: 'st', command: 'status', scope: 'global' as const },
        { name: 'lg', command: 'log --oneline', scope: 'local' as const, localPath: '/mock/path' },
    ];

    const mockHandlers = {
        onEdit: vi.fn(),
        onDelete: vi.fn(),
        onOpenLocalFolder: vi.fn(),
    };

    it('renders empty state when there are no aliases', () => {
        render(
            <AliasList
                aliases={[]}
                loading={false}
                searchQuery=""
                {...mockHandlers}
            />,
        );
        expect(screen.getByText(/No aliases yet/i)).toBeInTheDocument();
    });

    it('renders loading state', () => {
        render(
            <AliasList
                aliases={[]}
                loading={true}
                searchQuery=""
                {...mockHandlers}
            />,
        );
        expect(screen.getByText(/Loading aliases/i)).toBeInTheDocument();
    });

    it('renders list of aliases', () => {
        render(
            <AliasList
                aliases={mockAliases}
                loading={false}
                searchQuery=""
                {...mockHandlers}
            />,
        );
        expect(screen.getByText('co')).toBeInTheDocument();
        expect(screen.getByText('st')).toBeInTheDocument();
        expect(screen.getByText('lg')).toBeInTheDocument();
    });

    it('filters aliases based on search query', () => {
        render(
            <AliasList
                aliases={mockAliases}
                loading={false}
                searchQuery="checkout"
                {...mockHandlers}
            />,
        );
        expect(screen.getByText('co')).toBeInTheDocument();
        expect(screen.queryByText('st')).not.toBeInTheDocument();
    });

    it('calls edit handler when edit button is clicked', () => {
        render(
            <AliasList
                aliases={mockAliases}
                loading={false}
                searchQuery=""
                {...mockHandlers}
            />,
        );
        const editButtons = screen.getAllByTitle('Edit alias');
        fireEvent.click(editButtons[0]);
        expect(mockHandlers.onEdit).toHaveBeenCalledWith(mockAliases[0]);
    });

    it('calls open folder handler when local scope badge is clicked', () => {
        render(
            <AliasList
                aliases={mockAliases}
                loading={false}
                searchQuery=""
                localPath="/mock/path"
                {...mockHandlers}
            />,
        );
        const folderLink = screen.getByText(/path/);
        fireEvent.click(folderLink);
        expect(mockHandlers.onOpenLocalFolder).toHaveBeenCalledWith('/mock/path');
    });
});
