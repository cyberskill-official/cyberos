import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { I_AliasGroup } from '#/types';

import { GroupSidebar } from '#/components/GroupSidebar';

const mockGroups: I_AliasGroup[] = [
    { id: 'g1', name: 'Frontend', color: '#3b82f6' },
    { id: 'g2', name: 'Backend', color: '#22c55e' },
];

describe('groupSidebar', () => {
    const defaultProps = {
        groups: mockGroups,
        activeGroupId: null as string | null,
        onSelectGroup: vi.fn(),
        onCreateGroup: vi.fn().mockResolvedValue(null),
        onRenameGroup: vi.fn().mockResolvedValue(undefined),
        onSetGroupColor: vi.fn().mockResolvedValue(undefined),
        onDeleteGroup: vi.fn().mockResolvedValue(undefined),
        aliasCount: { g1: 5, g2: 3 },
    };

    it('renders "All" button and group names', () => {
        render(<GroupSidebar {...defaultProps} />);
        expect(screen.getByText('All')).toBeInTheDocument();
        expect(screen.getByText('Frontend')).toBeInTheDocument();
        expect(screen.getByText('Backend')).toBeInTheDocument();
    });

    it('shows alias counts', () => {
        render(<GroupSidebar {...defaultProps} />);
        expect(screen.getByText('5')).toBeInTheDocument();
        expect(screen.getByText('3')).toBeInTheDocument();
    });

    it('calls onSelectGroup when clicking All', async () => {
        const user = userEvent.setup();
        render(<GroupSidebar {...defaultProps} />);
        await user.click(screen.getByText('All'));
        expect(defaultProps.onSelectGroup).toHaveBeenCalledWith(null);
    });

    it('calls onSelectGroup when clicking a group', async () => {
        const user = userEvent.setup();
        render(<GroupSidebar {...defaultProps} />);
        await user.click(screen.getByText('Frontend'));
        expect(defaultProps.onSelectGroup).toHaveBeenCalledWith('g1');
    });

    it('shows "New group" button', () => {
        render(<GroupSidebar {...defaultProps} />);
        expect(screen.getByText('New group')).toBeInTheDocument();
    });

    it('shows create form when New group is clicked', async () => {
        const user = userEvent.setup();
        render(<GroupSidebar {...defaultProps} />);
        await user.click(screen.getByText('New group'));
        expect(screen.getByPlaceholderText('Group name')).toBeInTheDocument();
    });

    it('renders with empty groups', () => {
        render(<GroupSidebar {...defaultProps} groups={[]} />);
        expect(screen.getByText('All')).toBeInTheDocument();
        expect(screen.getByText('New group')).toBeInTheDocument();
    });
});
