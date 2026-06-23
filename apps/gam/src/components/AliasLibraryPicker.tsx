import { useCallback, useMemo, useState } from 'react';

import type { I_LibraryAlias } from '#/services/gitalias-library';

import { getAllAliases, getByCategory, getCategories, searchLibrary } from '#/services/gitalias-library';

interface I_AliasLibraryPickerProps {
    onSelect: (alias: I_LibraryAlias) => void;
    onClose: () => void;
}

export function AliasLibraryPicker({ onSelect, onClose }: I_AliasLibraryPickerProps) {
    const [search, setSearch] = useState('');
    const [activeCategory, setActiveCategory] = useState<string | null>(null);

    const categories = useMemo(() => getCategories(), []);

    const filteredAliases = useMemo(() => {
        if (search.trim())
            return searchLibrary(search);
        if (activeCategory)
            return getByCategory(activeCategory);
        return getAllAliases();
    }, [search, activeCategory]);

    const handleCategoryClick = useCallback((cat: string) => {
        setActiveCategory(prev => prev === cat ? null : cat);
        setSearch('');
    }, []);

    const handleSelect = useCallback((alias: I_LibraryAlias) => {
        onSelect(alias);
        onClose();
    }, [onSelect, onClose]);

    return (
        <div
            className="fixed inset-0 backdrop-blur-[3px] flex items-center justify-center z-[300] animate-fade-in"
            style={{ backgroundColor: 'rgba(0,0,0,0.35)' }}
            onClick={onClose}
        >
            <div
                className="w-[92%] max-w-[680px] max-h-[85vh] flex flex-col overflow-hidden animate-bounce-in rounded-xl border theme-card"
                style={{ backgroundColor: 'var(--color-surface-raised)', borderColor: 'var(--color-border)' }}
                onClick={e => e.stopPropagation()}
            >
                {/* Header */}
                <div className="flex justify-between items-center px-6 py-4 border-b" style={{ borderColor: 'var(--color-border)' }}>
                    <h2 className="m-0 text-base font-bold flex items-center gap-2" id="library-picker-title" style={{ color: 'var(--color-text)' }}>
                        <span className="inline-block animate-wiggle">📚</span>
                        Alias Library
                        <span className="text-xs font-normal ml-1" style={{ color: 'var(--color-text-muted)' }}>from GitAlias</span>
                    </h2>
                    <button
                        className="bg-transparent border-none text-lg w-7 h-7 flex items-center justify-center cursor-pointer transition-all duration-200 hover:scale-125 hover:rotate-90"
                        style={{ color: 'var(--color-text-muted)' }}
                        onClick={onClose}
                        aria-label="Close library"
                    >
                        ✕
                    </button>
                </div>

                {/* Search */}
                <div className="px-6 pt-4 pb-2">
                    <div className="relative ink-underline">
                        <span className="absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" style={{ color: 'var(--color-text-muted)' }}>🔍</span>
                        <input
                            type="text"
                            className="w-full border-2 rounded py-2 pl-9 pr-3 text-sm transition-all duration-200 focus-glow focus:outline-none"
                            style={{ backgroundColor: 'var(--color-surface)', borderColor: 'var(--color-border)', color: 'var(--color-text)' }}
                            placeholder="Search aliases by name, command, or description…"
                            value={search}
                            onChange={(e) => {
                                setSearch(e.target.value);
                                setActiveCategory(null);
                            }}
                            autoFocus
                            aria-label="Search aliases"
                        />
                    </div>
                </div>

                {/* Category chips */}
                <div className="px-6 py-2 flex flex-wrap gap-1.5 border-b" style={{ borderColor: 'var(--color-border)' }}>
                    {categories.map(cat => (
                        <button
                            key={cat}
                            type="button"
                            className="px-2 py-0.5 text-xs font-bold rounded cursor-pointer transition-all duration-200 border btn-press"
                            style={{
                                backgroundColor: activeCategory === cat ? 'var(--color-primary-muted)' : 'var(--color-surface)',
                                borderColor: activeCategory === cat ? 'var(--color-primary)' : 'var(--color-border)',
                                color: activeCategory === cat ? 'var(--color-primary)' : 'var(--color-text-muted)',
                            }}
                            onClick={() => handleCategoryClick(cat)}
                        >
                            {cat}
                        </button>
                    ))}
                </div>

                {/* Alias list */}
                <div className="flex-1 overflow-y-auto px-6 py-3 flex flex-col gap-2 min-h-0">
                    {filteredAliases.length === 0
                        ? (
                                <div className="text-center py-8 text-sm" style={{ color: 'var(--color-text-muted)' }}>
                                    <span className="block text-2xl mb-2">🤷</span>
                                    No aliases found matching your search.
                                </div>
                            )
                        : (
                                <>
                                    <div className="text-xs mb-1" style={{ color: 'var(--color-text-muted)' }}>
                                        {filteredAliases.length}
                                        {' '}
                                        alias
                                        {filteredAliases.length !== 1 ? 'es' : ''}
                                        {' '}
                                        found
                                        {activeCategory && (
                                            <span className="ml-1">
                                                in
                                                <strong>{activeCategory}</strong>
                                            </span>
                                        )}
                                    </div>
                                    {filteredAliases.map((alias, i) => (
                                        <button
                                            key={`${alias.name}-${i}`}
                                            type="button"
                                            className="w-full text-left border rounded p-3 cursor-pointer transition-all duration-200 hover:-translate-y-0.5 group btn-press"
                                            style={{
                                                backgroundColor: 'var(--color-surface)',
                                                borderColor: 'var(--color-border)',
                                                animation: `rowEnter 0.35s cubic-bezier(0.34, 1.56, 0.64, 1) ${Math.min(i * 0.03, 0.5)}s both`,
                                            }}
                                            onClick={() => handleSelect(alias)}
                                            title="Click to use this alias"
                                        >
                                            <div className="flex items-start gap-3">
                                                <span
                                                    className="shrink-0 font-mono text-sm font-bold border px-2 py-0.5 rounded min-w-[60px] text-center"
                                                    style={{ color: 'var(--color-badge-global-text)', backgroundColor: 'var(--color-badge-global-bg)', borderColor: 'var(--color-badge-global-border)' }}
                                                >
                                                    {alias.name}
                                                </span>
                                                <div className="flex-1 min-w-0">
                                                    <div className="text-sm font-bold leading-tight mb-1 capitalize" style={{ color: 'var(--color-text)' }}>
                                                        {alias.description}
                                                    </div>
                                                    <div className="font-mono text-xs leading-snug break-all line-clamp-2" style={{ color: 'var(--color-text-secondary)' }}>
                                                        {alias.command}
                                                    </div>
                                                </div>
                                                <span
                                                    className="shrink-0 text-[10px] font-bold px-1.5 py-0.5 rounded uppercase tracking-wider"
                                                    style={{ color: 'var(--color-text-muted)', backgroundColor: 'var(--color-surface-hover)' }}
                                                >
                                                    {alias.category}
                                                </span>
                                            </div>
                                        </button>
                                    ))}
                                </>
                            )}
                </div>

                {/* Footer */}
                <div className="flex justify-between items-center px-6 py-3 border-t text-xs" style={{ borderColor: 'var(--color-border)', color: 'var(--color-text-muted)' }}>
                    <span>
                        Source:
                        {' '}
                        <a href="https://github.com/GitAlias/gitalias" target="_blank" rel="noopener noreferrer" style={{ color: 'var(--color-primary)' }} className="hover:underline">GitAlias/gitalias</a>
                    </span>
                    <button
                        type="button"
                        className="px-4 py-1.5 text-sm font-bold bg-transparent border rounded transition-all cursor-pointer btn-press"
                        style={{ color: 'var(--color-text-muted)', borderColor: 'var(--color-border)' }}
                        onClick={onClose}
                    >
                        Cancel
                    </button>
                </div>
            </div>
        </div>
    );
}
