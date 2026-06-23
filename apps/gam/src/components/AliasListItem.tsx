import * as React from 'react';
import { useCallback, useState } from 'react';

import type { I_AliasGroup, I_GitAlias } from '#/types';

export interface I_AliasListItemProps {
    alias: I_GitAlias;
    localPath?: string;
    index: number;
    onEdit: (alias: I_GitAlias) => void;
    onDelete: (alias: I_GitAlias) => void;
    onOpenLocalFolder: (path: string) => void;
    groups?: I_AliasGroup[];
    aliasGroupIds?: string[];
    onSetAliasGroups?: (aliasName: string, groupIds: string[]) => void;
}

export const AliasListItem = React.memo(({
    alias,
    localPath,
    index,
    onEdit,
    onDelete,
    onOpenLocalFolder,
    groups = [],
    aliasGroupIds = [],
    onSetAliasGroups,
}: I_AliasListItemProps) => {
    const isGlobal = alias.scope === 'global';
    const [showGroupMenu, setShowGroupMenu] = useState(false);

    const toggleGroup = useCallback((groupId: string) => {
        if (!onSetAliasGroups)
            return;
        const current = [...aliasGroupIds];
        const idx = current.indexOf(groupId);
        if (idx >= 0)
            current.splice(idx, 1);
        else current.push(groupId);
        onSetAliasGroups(alias.name, current);
    }, [alias.name, aliasGroupIds, onSetAliasGroups]);

    const aliasGroups = groups.filter(g => aliasGroupIds.includes(g.id));

    return (
        <tr
            className="alias-row group border-b border-l-2 row-hover-lift"
            style={{
                animation: `rowEnter 0.35s cubic-bezier(0.34, 1.56, 0.64, 1) ${index * 0.04}s both`,
                borderBottomColor: 'var(--color-border)',
                borderLeftColor: isGlobal ? 'var(--color-badge-global-text)' : 'var(--color-badge-local-text)',
            }}
            onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--theme-row-hover-bg)'; }}
            onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.backgroundColor = ''; }}
        >
            <td>
                <div className="flex items-center gap-1.5">
                    <span
                        className="font-mono font-bold text-[15px] px-2 py-0.5 inline-block"
                        style={{ color: isGlobal ? 'var(--color-badge-global-text)' : 'var(--color-badge-local-text)' }}
                    >
                        {alias.name}
                    </span>
                    {aliasGroups.map(g => (
                        <span
                            key={g.id}
                            className="inline-block w-2 h-2 rounded-full flex-shrink-0"
                            style={{ backgroundColor: g.color }}
                            title={g.name}
                        />
                    ))}
                </div>
            </td>
            <td className="text-center">
                <span
                    className="font-mono text-sm"
                    style={{ color: isGlobal ? 'var(--color-badge-global-text)' : 'var(--color-badge-local-text)', opacity: 0.6 }}
                >
                    {alias.score ? Math.round(alias.score) : '—'}
                </span>
            </td>
            <td>
                <span
                    className="font-mono text-[13px] break-all inline-block max-w-[400px] overflow-hidden text-ellipsis"
                    style={{ color: 'var(--color-text-secondary)' }}
                    title={alias.command}
                >
                    {alias.command}
                </span>
            </td>
            <td>
                <span
                    className={`inline-block px-2 py-0.5 text-[11px] font-bold uppercase tracking-wide transition-all rounded-sm border ${!isGlobal && (alias.localPath || localPath) ? 'cursor-pointer underline decoration-dotted' : ''}`}
                    style={{
                        backgroundColor: isGlobal ? 'var(--color-badge-global-bg)' : 'var(--color-badge-local-bg)',
                        color: isGlobal ? 'var(--color-badge-global-text)' : 'var(--color-badge-local-text)',
                        borderColor: isGlobal ? 'var(--color-badge-global-border)' : 'var(--color-badge-local-border)',
                    }}
                    onClick={() => {
                        const targetPath = alias.localPath || localPath;
                        if (alias.scope === 'local' && targetPath) {
                            onOpenLocalFolder(targetPath);
                        }
                    }}
                    title={!isGlobal && (alias.localPath || localPath) ? 'Click to open local repository folder' : ''}
                >
                    {!isGlobal && (alias.localPath || localPath) ? (alias.localPath || localPath)!.split(/[/\\]/).pop() : alias.scope}
                </span>
            </td>
            <td>
                <div className="flex gap-1.5 items-center">
                    <button className="bg-transparent border-none text-base cursor-pointer hover:scale-125 transition-all duration-200 p-0.5" onClick={() => onEdit(alias)} title="Edit alias" aria-label={`Edit alias ${alias.name}`}>
                        ✏️
                    </button>
                    <button
                        className="bg-transparent border-none text-base cursor-pointer hover:scale-125 transition-all duration-200 p-0.5"
                        style={{ color: 'inherit' }}
                        onClick={() => onDelete(alias)}
                        title="Delete alias"
                        aria-label={`Delete alias ${alias.name}`}
                    >
                        🗑
                    </button>
                    <div className="relative">
                        <button
                            className="bg-transparent border-none text-sm cursor-pointer hover:scale-125 transition-all duration-200 p-0.5"
                            onClick={() => setShowGroupMenu(v => !v)}
                            title="Assign to groups"
                            aria-label={`Assign ${alias.name} to groups`}
                        >
                            🏷️
                        </button>
                        {showGroupMenu && (
                            <div
                                className="absolute right-0 top-full mt-1 z-50 rounded-lg shadow-lg border py-1 min-w-[160px]"
                                style={{
                                    backgroundColor: 'var(--color-surface)',
                                    borderColor: 'var(--color-border)',
                                }}
                            >
                                {groups.length === 0
                                    ? (
                                            <div className="px-3 py-2 text-xs" style={{ color: 'var(--color-text-muted)' }}>
                                                No groups yet. Create one in the sidebar.
                                            </div>
                                        )
                                    : (
                                            groups.map(g => (
                                                <label
                                                    key={g.id}
                                                    className="flex items-center gap-2 px-3 py-1.5 text-sm cursor-pointer hover:bg-[var(--color-surface-hover)] transition-colors"
                                                    style={{ color: 'var(--color-text)' }}
                                                >
                                                    <input
                                                        type="checkbox"
                                                        checked={aliasGroupIds.includes(g.id)}
                                                        onChange={() => toggleGroup(g.id)}
                                                        className="accent-current"
                                                    />
                                                    <span
                                                        className="inline-block w-2.5 h-2.5 rounded-full flex-shrink-0"
                                                        style={{ backgroundColor: g.color }}
                                                    />
                                                    <span className="truncate">{g.name}</span>
                                                </label>
                                            ))
                                        )}
                                <button
                                    type="button"
                                    className="w-full text-left px-3 py-1 text-xs cursor-pointer bg-transparent border-none border-t mt-1 pt-1.5"
                                    style={{ color: 'var(--color-text-muted)', borderTopColor: 'var(--color-border)' }}
                                    onClick={() => setShowGroupMenu(false)}
                                >
                                    Done
                                </button>
                            </div>
                        )}
                    </div>
                </div>
            </td>
        </tr>
    );
});
