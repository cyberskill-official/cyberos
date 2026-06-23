interface I_ScopeSelectorProps {
    scope: 'global' | 'local';
    onScopeChange: (scope: 'global' | 'local') => void;
    isEditing: boolean;
    localPath?: string;
    aliasLocalPath?: string;
    onSelectFolder?: () => Promise<void>;
}

export function ScopeSelector({
    scope,
    onScopeChange,
    isEditing,
    localPath,
    aliasLocalPath,
    onSelectFolder,
}: I_ScopeSelectorProps) {
    return (
        <>
            {/* Scope toggle */}
            <div className="flex items-center justify-between">
                <label className="text-[11px] font-bold uppercase tracking-wider" style={{ color: 'var(--color-text-muted)' }}>Scope</label>
                <div
                    className="flex rounded p-0.5 border relative"
                    style={{ backgroundColor: 'var(--color-surface)', borderColor: 'var(--color-border)' }}
                    role="radiogroup"
                    aria-label="Scope"
                >
                    <div
                        className="absolute top-0.5 bottom-0.5 w-[calc(50%-2px)] border rounded transition-all duration-300 ease-[cubic-bezier(0.34,1.56,0.64,1)] pointer-events-none"
                        style={{
                            left: scope === 'global' ? '2px' : 'calc(50% + 2px)',
                            backgroundColor: 'var(--color-scope-active-bg)',
                            borderColor: 'var(--color-border)',
                        }}
                    />
                    <button
                        type="button"
                        role="radio"
                        className="px-3.5 py-1 text-sm font-bold transition-all duration-200 rounded relative z-[1] bg-transparent border-none cursor-pointer"
                        style={{ color: scope === 'global' ? 'var(--color-badge-global-text)' : 'var(--color-text-muted)' }}
                        onClick={() => onScopeChange('global')}
                        aria-checked={scope === 'global'}
                    >
                        🌐 Global
                    </button>
                    <button
                        type="button"
                        role="radio"
                        className="px-3.5 py-1 text-sm font-bold transition-all duration-200 rounded relative z-[1] bg-transparent border-none cursor-pointer"
                        style={{ color: scope === 'local' ? 'var(--color-badge-local-text)' : 'var(--color-text-muted)' }}
                        onClick={() => onScopeChange('local')}
                        aria-checked={scope === 'local'}
                    >
                        📁 Local
                    </button>
                </div>
            </div>

            {/* Folder Selection */}
            <div
                className={`flex items-center gap-2 border border-dashed py-1.5 px-3 rounded transition-all duration-300 ${scope === 'local' ? 'opacity-100 max-h-20' : 'opacity-25 max-h-20 pointer-events-none'}`}
                style={{ backgroundColor: 'var(--color-surface)', borderColor: 'var(--color-border)' }}
            >
                <span className="text-sm">📁</span>
                <span className="font-mono text-xs flex-1 overflow-hidden text-ellipsis whitespace-nowrap" style={{ color: 'var(--color-text-secondary)' }}>
                    {scope === 'local'
                        ? ((aliasLocalPath || localPath) || 'No local repository selected')
                        : 'Local repo (disabled for global)'}
                </span>
                {onSelectFolder && !isEditing && (
                    <button
                        type="button"
                        className="whitespace-nowrap py-0.5 px-2 text-xs font-bold border rounded disabled:opacity-30 disabled:cursor-not-allowed transition-all cursor-pointer btn-press"
                        style={{ color: 'var(--color-text-muted)', backgroundColor: 'var(--color-scope-active-bg)', borderColor: 'var(--color-border)' }}
                        onClick={async () => { await onSelectFolder(); }}
                        disabled={scope !== 'local'}
                    >
                        Change…
                    </button>
                )}
            </div>
        </>
    );
}
