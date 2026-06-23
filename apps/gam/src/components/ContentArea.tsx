import type { I_AliasGroup, I_GitAlias } from '#/types';

import { AliasList } from './AliasList';
import { SearchBar } from './SearchBar';
import { StatusBar } from './StatusBar';
import { Toolbar } from './Toolbar';

interface I_ContentAreaProps {
    scope: 'global' | 'local' | 'all';
    onScopeChange: (scope: 'global' | 'local' | 'all') => void;
    onAdd: () => void;
    localPath: string | null | undefined;
    onSelectFolder: () => void;
    onClearFolder: () => void;
    searchQuery: string;
    onSearchChange: (query: string) => void;
    filteredAliases: I_GitAlias[];
    totalAliases: I_GitAlias[];
    debouncedQuery: string;
    groupFilteredAliases: I_GitAlias[];
    loading: boolean;
    error: string | null;
    fetchAliases: () => void;
    onEdit: (alias: I_GitAlias) => void;
    onDelete: (alias: I_GitAlias) => void;
    onOpenLocalFolder: (path: string) => void;
    groups: I_AliasGroup[];
    assignments: Record<string, string[]>;
    onSetAliasGroups: (aliasName: string, groupIds: string[]) => void;
    activeGroupName?: string;
    onOpenExternal: (url: string) => void;
}

export function ContentArea({
    scope,
    onScopeChange,
    onAdd,
    localPath,
    onSelectFolder,
    onClearFolder,
    searchQuery,
    onSearchChange,
    filteredAliases,
    totalAliases,
    debouncedQuery,
    groupFilteredAliases,
    loading,
    error,
    fetchAliases,
    onEdit,
    onDelete,
    onOpenLocalFolder,
    groups,
    assignments,
    onSetAliasGroups,
    activeGroupName,
    onOpenExternal,
}: I_ContentAreaProps) {
    const q = debouncedQuery.toLowerCase();
    const displayCount = q
        ? groupFilteredAliases.filter(a => a.name.toLowerCase().includes(q) || a.command.toLowerCase().includes(q)).length
        : groupFilteredAliases.length;

    return (
        <div className="flex-1 flex flex-col px-6 py-5 gap-5 overflow-hidden relative">
            {/* Red margin line (notebook only) */}
            <div className="nb-margin-line--subtle absolute left-10 top-0 bottom-0 w-px z-0 pointer-events-none" />

            <Toolbar
                scope={scope}
                onScopeChange={onScopeChange}
                onAdd={onAdd}
                localPath={localPath ?? undefined}
                onSelectFolder={onSelectFolder}
                onClearFolder={onClearFolder}
            />

            <SearchBar
                value={searchQuery}
                onChange={onSearchChange}
                resultCount={filteredAliases.length}
                totalCount={totalAliases.length}
            />

            {error && (
                <div
                    className="error-banner rounded-lg border px-5 py-3 flex items-center gap-3"
                >
                    <span className="font-bold text-lg">✗</span>
                    <span className="text-sm font-bold flex-1">{error}</span>
                    <button
                        className="error-banner-btn px-3 py-1 text-xs font-bold rounded border cursor-pointer transition-all duration-200 hover:opacity-80 btn-press"
                        onClick={fetchAliases}
                    >
                        Retry
                    </button>
                </div>
            )}

            <AliasList
                aliases={groupFilteredAliases}
                loading={loading}
                searchQuery={debouncedQuery}
                localPath={localPath ?? undefined}
                onEdit={onEdit}
                onDelete={onDelete}
                onOpenLocalFolder={onOpenLocalFolder}
                groups={groups}
                assignments={assignments}
                onSetAliasGroups={onSetAliasGroups}
            />

            <StatusBar
                totalAliases={totalAliases.length}
                filteredCount={displayCount}
                scope={scope}
                isSearching={!!searchQuery}
                activeGroupName={activeGroupName}
                onOpenExternal={onOpenExternal}
            />
        </div>
    );
}
