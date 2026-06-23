import { useCallback, useState } from 'react';

import type { I_AliasGroup } from '#/types';

import { ColorDot, DEFAULT_COLORS } from './sidebar/ColorDot';

interface I_GroupSidebarProps {
    groups: I_AliasGroup[];
    activeGroupId: string | null;
    onSelectGroup: (id: string | null) => void;
    onCreateGroup: (name: string, color: string) => Promise<I_AliasGroup | null>;
    onRenameGroup: (id: string, name: string) => Promise<void>;
    onSetGroupColor: (id: string, color: string) => Promise<void>;
    onDeleteGroup: (id: string) => Promise<void>;
    aliasCount?: Record<string, number>;
}

export function GroupSidebar({
    groups,
    activeGroupId,
    onSelectGroup,
    onCreateGroup,
    onRenameGroup,
    onSetGroupColor,
    onDeleteGroup,
    aliasCount = {},
}: I_GroupSidebarProps) {
    const [isCreating, setIsCreating] = useState(false);
    const [newName, setNewName] = useState('');
    const [newColor, setNewColor] = useState(DEFAULT_COLORS[0]);
    const [editingId, setEditingId] = useState<string | null>(null);
    const [editName, setEditName] = useState('');

    const handleCreate = useCallback(async () => {
        const trimmed = newName.trim();
        if (!trimmed) {
            return;
        }
        await onCreateGroup(trimmed, newColor);
        setNewName('');
        setNewColor(DEFAULT_COLORS[Math.floor(Math.random() * DEFAULT_COLORS.length)]);
        setIsCreating(false);
    }, [newName, newColor, onCreateGroup]);

    const handleRename = useCallback(async (id: string) => {
        const trimmed = editName.trim();
        if (!trimmed) {
            return;
        }
        await onRenameGroup(id, trimmed);
        setEditingId(null);
    }, [editName, onRenameGroup]);

    return (
        <div
            className="flex flex-col gap-1 py-2"
            style={{ minWidth: 180 }}
        >
            {/* All (no filter) */}
            <button
                type="button"
                className={`flex items-center justify-between gap-2 px-3 py-1.5 rounded-md text-sm text-left transition-colors cursor-pointer border-none ${activeGroupId === null ? 'font-bold' : 'font-normal'}`}
                style={{
                    backgroundColor: activeGroupId === null ? 'var(--color-surface-hover)' : 'transparent',
                    color: 'var(--color-text)',
                }}
                onClick={() => onSelectGroup(null)}
            >
                <span>All</span>
            </button>

            {/* Group list */}
            {groups.map(group => (
                <div key={group.id} className="flex items-center gap-1.5 group relative">
                    {editingId === group.id
                        ? (
                                <input
                                    className="flex-1 text-sm px-3 py-1 bg-transparent border rounded-md outline-none"
                                    style={{ borderColor: 'var(--color-border)', color: 'var(--color-text)' }}
                                    value={editName}
                                    onChange={e => setEditName(e.target.value)}
                                    onKeyDown={(e) => {
                                        if (e.key === 'Enter')
                                            handleRename(group.id);
                                        if (e.key === 'Escape')
                                            setEditingId(null);
                                    }}
                                    onBlur={() => handleRename(group.id)}
                                    autoFocus
                                />
                            )
                        : (
                                <div className="flex-1 flex items-center gap-0">
                                    <ColorDot
                                        color={group.color}
                                        onChange={c => onSetGroupColor(group.id, c)}
                                    />
                                    {/* Group name button */}
                                    <button
                                        type="button"
                                        className={`flex-1 flex items-center justify-between gap-2 py-1.5 pr-2 rounded-md text-sm text-left transition-colors cursor-pointer border-none ${activeGroupId === group.id ? 'font-bold' : 'font-normal'}`}
                                        style={{
                                            backgroundColor: 'transparent',
                                            color: 'var(--color-text)',
                                        }}
                                        onClick={() => onSelectGroup(group.id)}
                                        onDoubleClick={() => {
                                            setEditingId(group.id);
                                            setEditName(group.name);
                                        }}
                                    >
                                        <span className="truncate max-w-[110px]">{group.name}</span>
                                        {(aliasCount[group.id] ?? 0) > 0 && (
                                            <span
                                                className="text-xs tabular-nums"
                                                style={{ color: 'var(--color-text-muted)' }}
                                            >
                                                {aliasCount[group.id]}
                                            </span>
                                        )}
                                    </button>
                                </div>
                            )}

                    {/* Context actions */}
                    {editingId !== group.id && (
                        <button
                            type="button"
                            className="opacity-0 group-hover:opacity-50 hover:!opacity-100 transition-opacity px-1 text-xs cursor-pointer bg-transparent border-none"
                            style={{ color: 'var(--color-danger)' }}
                            onClick={() => onDeleteGroup(group.id)}
                            title="Delete group"
                        >
                            ✕
                        </button>
                    )}
                </div>
            ))}

            {/* Create new group */}
            {isCreating
                ? (
                        <div className="flex items-center gap-1.5 px-2 py-1">
                            <input
                                type="color"
                                value={newColor}
                                onChange={e => setNewColor(e.target.value)}
                                className="w-6 h-6 p-0 border-none bg-transparent cursor-pointer"
                            />
                            <input
                                className="flex-1 text-sm px-2 py-1 bg-transparent border rounded-md outline-none"
                                style={{ borderColor: 'var(--color-border)', color: 'var(--color-text)' }}
                                placeholder="Group name"
                                value={newName}
                                onChange={e => setNewName(e.target.value)}
                                onKeyDown={(e) => {
                                    if (e.key === 'Enter') {
                                        handleCreate();
                                    }
                                    if (e.key === 'Escape') {
                                        setIsCreating(false);
                                    }
                                }}
                                autoFocus
                            />
                        </div>
                    )
                : (
                        <button
                            type="button"
                            className="flex items-center gap-2 px-3 py-1.5 text-sm cursor-pointer bg-transparent border-none rounded-md transition-colors"
                            style={{ color: 'var(--color-text-muted)' }}
                            onClick={() => setIsCreating(true)}
                        >
                            <span>+</span>
                            <span>New group</span>
                        </button>
                    )}
        </div>
    );
}
