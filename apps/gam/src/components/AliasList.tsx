import { useEffect, useMemo, useState } from 'react';

import type { I_AliasGroup, I_GitAlias } from '#/types';

import { AliasListItem } from './AliasListItem';

interface I_AliasListProps {
    aliases: I_GitAlias[];
    loading: boolean;
    searchQuery: string;
    localPath?: string;
    onEdit: (alias: I_GitAlias) => void;
    onDelete: (alias: I_GitAlias) => void;
    onOpenLocalFolder: (path: string) => void;
    groups?: I_AliasGroup[];
    assignments?: Record<string, string[]>;
    onSetAliasGroups?: (aliasName: string, groupIds: string[]) => void;
}

type T_SortKey = 'name' | 'command' | 'scope' | 'rank';
type T_SortDir = 'asc' | 'desc';

export function AliasList({
    aliases,
    loading,
    searchQuery,
    localPath,
    onEdit,
    onDelete,
    onOpenLocalFolder,
    groups = [],
    assignments = {},
    onSetAliasGroups,
}: I_AliasListProps) {
    const [sortKey, setSortKey] = useState<T_SortKey>('name');
    const [sortDir, setSortDir] = useState<T_SortDir>('asc');
    const [currentPage, setCurrentPage] = useState(1);
    const itemsPerPage = 10;

    const filteredAliases = useMemo(() => {
        let result = [...aliases];

        if (searchQuery) {
            const q = searchQuery.toLowerCase();
            result = result.filter(
                a =>
                    a.name.toLowerCase().includes(q)
                    || a.command.toLowerCase().includes(q),
            );
        }

        result.sort((a, b) => {
            if (sortKey === 'rank') {
                const aScore = a.score || 0;
                const bScore = b.score || 0;
                const cmp = bScore - aScore;

                if (cmp === 0) {
                    return sortDir === 'asc' ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name);
                }

                return sortDir === 'asc' ? cmp : -cmp;
            }

            const aVal = a[sortKey];
            const bVal = b[sortKey];
            const cmp = String(aVal).localeCompare(String(bVal));

            return sortDir === 'asc' ? cmp : -cmp;
        });

        return result;
    }, [aliases, searchQuery, sortKey, sortDir]);

    useEffect(() => {
        setCurrentPage(1);
    }, [searchQuery, aliases, sortKey, sortDir]);

    const handleSort = (key: T_SortKey) => {
        if (sortKey === key) {
            setSortDir(d => (d === 'asc' ? 'desc' : 'asc'));
        }
        else {
            setSortKey(key);
            setSortDir('asc');
        }
    };

    const totalPages = Math.max(1, Math.ceil(filteredAliases.length / itemsPerPage));
    const paginatedAliases = filteredAliases.slice(
        (currentPage - 1) * itemsPerPage,
        currentPage * itemsPerPage,
    );

    const sortIcon = (key: T_SortKey) => {
        if (sortKey !== key)
            return null;
        return <span className="ml-1" style={{ color: 'var(--color-focus)' }}>{sortDir === 'asc' ? '↑' : '↓'}</span>;
    };

    const headerStyle = (key: T_SortKey) => ({
        color: sortKey === key ? 'var(--color-text)' : 'var(--color-text-muted)',
        borderBottom: sortKey === key ? '2px solid var(--color-focus)' : '2px solid transparent',
    });

    return (
        <div
            className="flex-1 border overflow-hidden flex flex-col page-fold theme-card theme-table"
            style={{ backgroundColor: 'var(--color-surface)', borderColor: 'var(--color-border)' }}
        >
            <table className="w-full text-left table-fixed">
                <thead>
                    <tr className="border-b-2" style={{ borderColor: 'var(--color-border)' }}>
                        <th
                            className="px-5 py-3 font-bold text-[11px] uppercase tracking-wider cursor-pointer select-none whitespace-nowrap transition-all duration-200 w-1/4"
                            style={headerStyle('name')}
                            onClick={() => handleSort('name')}
                            onKeyDown={e => (e.key === 'Enter' || e.key === ' ') && handleSort('name')}
                            role="button"
                            tabIndex={0}
                            aria-sort={sortKey === 'name' ? (sortDir === 'asc' ? 'ascending' : 'descending') : 'none'}
                        >
                            Alias
                            {sortIcon('name')}
                        </th>
                        <th
                            className="px-5 py-3 font-bold text-[11px] uppercase tracking-wider cursor-pointer select-none whitespace-nowrap transition-all duration-200 text-center w-[90px]"
                            style={headerStyle('rank')}
                            onClick={() => handleSort('rank')}
                            title="Sort by telemetry usage rate"
                            role="button"
                            tabIndex={0}
                            aria-sort={sortKey === 'rank' ? (sortDir === 'asc' ? 'ascending' : 'descending') : 'none'}
                        >
                            Usage
                            {sortIcon('rank')}
                        </th>
                        <th
                            className="px-5 py-3 font-bold text-[11px] uppercase tracking-wider cursor-pointer select-none whitespace-nowrap transition-all duration-200"
                            style={headerStyle('command')}
                            onClick={() => handleSort('command')}
                            onKeyDown={e => (e.key === 'Enter' || e.key === ' ') && handleSort('command')}
                            role="button"
                            tabIndex={0}
                            aria-sort={sortKey === 'command' ? (sortDir === 'asc' ? 'ascending' : 'descending') : 'none'}
                        >
                            Command
                            {sortIcon('command')}
                        </th>
                        <th
                            className="px-5 py-3 font-bold text-[11px] uppercase tracking-wider cursor-pointer select-none whitespace-nowrap transition-all duration-200 w-[15%]"
                            style={headerStyle('scope')}
                            onClick={() => handleSort('scope')}
                            onKeyDown={e => (e.key === 'Enter' || e.key === ' ') && handleSort('scope')}
                            role="button"
                            tabIndex={0}
                            aria-sort={sortKey === 'scope' ? (sortDir === 'asc' ? 'ascending' : 'descending') : 'none'}
                        >
                            Scope
                            {sortIcon('scope')}
                        </th>
                        <th className="px-5 py-3 font-bold text-[11px] uppercase tracking-wider w-28" style={{ color: 'var(--color-text-muted)' }}>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    {loading
                        ? (
                                <tr>
                                    <td colSpan={5}>
                                        <div className="flex flex-col items-center justify-center p-16" style={{ color: 'var(--color-text-muted)' }}>
                                            <div className="text-4xl animate-float mb-4">✏️</div>
                                            <span className="text-sm font-bold mb-2">Loading aliases...</span>
                                            <div className="w-32 h-1 rounded-full overflow-hidden" style={{ backgroundColor: 'var(--color-surface-hover)' }}>
                                                <div className="h-full shimmer-bg rounded-full" />
                                            </div>
                                        </div>
                                    </td>
                                </tr>
                            )
                        : aliases.length === 0
                            ? (
                                    <tr>
                                        <td colSpan={5}>
                                            <div className="flex flex-col items-center justify-center p-14 text-center animate-fade-in">
                                                <div className="text-5xl mb-4 animate-float">📝</div>
                                                <div className="text-lg font-bold mb-1" style={{ color: 'var(--color-text)' }}>No aliases yet!</div>
                                                <div className="text-sm max-w-[280px] mb-4" style={{ color: 'var(--color-text-muted)' }}>
                                                    Start by creating your first alias or import an existing collection.
                                                </div>
                                                <div className="w-16 h-px" style={{ backgroundColor: 'var(--color-border)' }} />
                                            </div>
                                        </td>
                                    </tr>
                                )
                            : filteredAliases.length === 0
                                ? (
                                        <tr>
                                            <td colSpan={5}>
                                                <div className="flex flex-col items-center justify-center p-20 text-center animate-fade-in">
                                                    <div className="text-4xl mb-3 animate-float">🔍</div>
                                                    <div className="text-lg font-bold mb-1" style={{ color: 'var(--color-text)' }}>Nothing found</div>
                                                    <div className="text-sm max-w-[280px]" style={{ color: 'var(--color-text-muted)' }}>
                                                        Try a different search term.
                                                    </div>
                                                </div>
                                            </td>
                                        </tr>
                                    )
                                : (
                                        paginatedAliases.map((alias, idx) => {
                                            return (
                                                <AliasListItem
                                                    key={`${alias.scope}-${alias.name}`}
                                                    alias={alias}
                                                    localPath={localPath}
                                                    index={idx}
                                                    onEdit={onEdit}
                                                    onDelete={onDelete}
                                                    onOpenLocalFolder={onOpenLocalFolder}
                                                    groups={groups}
                                                    aliasGroupIds={assignments[alias.name]}
                                                    onSetAliasGroups={onSetAliasGroups}
                                                />
                                            );
                                        })
                                    )}
                </tbody>
            </table>
            {totalPages > 1 && (
                <div className="px-6 py-3.5 border-t flex justify-between items-center" style={{ borderColor: 'var(--color-border)' }}>
                    <button
                        className="px-3 py-1 text-sm font-bold bg-transparent border rounded cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed transition-all btn-press"
                        style={{ color: 'var(--color-text-muted)', borderColor: 'var(--color-border)' }}
                        disabled={currentPage === 1}
                        onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
                    >
                        ← Prev
                    </button>
                    <span className="text-[11px] font-bold uppercase tracking-wider" style={{ color: 'var(--color-text-muted)' }}>
                        {currentPage}
                        {' '}
                        /
                        {totalPages}
                    </span>
                    <button
                        className="px-3 py-1 text-sm font-bold bg-transparent border rounded cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed transition-all btn-press"
                        style={{ color: 'var(--color-text-muted)', borderColor: 'var(--color-border)' }}
                        disabled={currentPage === totalPages}
                        onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
                    >
                        Next →
                    </button>
                </div>
            )}
        </div>
    );
}
