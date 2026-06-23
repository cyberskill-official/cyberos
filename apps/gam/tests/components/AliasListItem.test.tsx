import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { I_AliasGroup, I_GitAlias } from '#/types';

import { AliasListItem } from '#/components/AliasListItem';

describe('aliasListItem', () => {
    const globalAlias: I_GitAlias = { name: 'co', command: 'checkout', scope: 'global' };
    const localAlias: I_GitAlias = { name: 'st', command: 'status -sb', scope: 'local', localPath: '/repo' };
    const groups: I_AliasGroup[] = [
        { id: 'g1', name: 'Git Flow', color: '#3b82f6' },
    ];

    const defaultProps = {
        alias: globalAlias,
        index: 0,
        onEdit: vi.fn(),
        onDelete: vi.fn(),
        onOpenLocalFolder: vi.fn(),
        groups: [],
        aliasGroupIds: [] as string[],
        onSetAliasGroups: vi.fn(),
    };

    it('renders alias name and command', () => {
        const { container } = render(
            <table><tbody><AliasListItem {...defaultProps} /></tbody></table>,
        );
        expect(screen.getByText('co')).toBeInTheDocument();
        expect(screen.getByText('checkout')).toBeInTheDocument();
        expect(container.querySelector('tr')).toBeTruthy();
    });

    it('shows global scope badge', () => {
        render(<table><tbody><AliasListItem {...defaultProps} /></tbody></table>);
        expect(screen.getByText('global')).toBeInTheDocument();
    });

    it('shows local scope with path', () => {
        render(
            <table><tbody><AliasListItem {...defaultProps} alias={localAlias} localPath="/repo" /></tbody></table>,
        );
        expect(screen.getByText('repo')).toBeInTheDocument();
    });

    it('calls onEdit when edit button is clicked', async () => {
        const user = userEvent.setup();
        render(<table><tbody><AliasListItem {...defaultProps} /></tbody></table>);
        const editBtn = screen.getByTitle('Edit alias');
        await user.click(editBtn);
        expect(defaultProps.onEdit).toHaveBeenCalledWith(globalAlias);
    });

    it('calls onDelete when delete button is clicked', async () => {
        const user = userEvent.setup();
        render(<table><tbody><AliasListItem {...defaultProps} /></tbody></table>);
        const deleteBtn = screen.getByTitle('Delete alias');
        await user.click(deleteBtn);
        expect(defaultProps.onDelete).toHaveBeenCalledWith(globalAlias);
    });

    it('renders group dots when assigned to groups', () => {
        render(
            <table>
                <tbody>
                    <AliasListItem {...defaultProps} groups={groups} aliasGroupIds={['g1']} />
                </tbody>
            </table>,
        );
        expect(screen.getByTitle('Git Flow')).toBeInTheDocument();
    });

    it('shows usage dash when no score', () => {
        render(<table><tbody><AliasListItem {...defaultProps} /></tbody></table>);
        expect(screen.getByText('—')).toBeInTheDocument();
    });
});
