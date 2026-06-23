interface I_ToolbarProps {
    scope: 'global' | 'local' | 'all';
    onScopeChange: (scope: 'global' | 'local' | 'all') => void;
    onAdd: () => void;
    localPath?: string;
    onSelectFolder?: () => void;
    onClearFolder?: () => void;
}

export function Toolbar({
    scope,
    onScopeChange,
    onAdd,
    localPath,
    onSelectFolder,
    onClearFolder,
}: I_ToolbarProps) {
    const scopes = ['all', 'global', 'local'] as const;

    return (
        <div className="flex justify-between items-center flex-wrap gap-4 relative z-10">
            <div className="flex items-center gap-4 flex-wrap">
                {/* Scope tabs */}
                <div
                    className="flex rounded-lg p-1 border"
                    style={{ backgroundColor: 'var(--color-surface)', borderColor: 'var(--color-border)' }}
                >
                    {scopes.map((s) => {
                        const isActive = scope === s;
                        const label = s === 'all' ? 'All' : s === 'global' ? '🌐 Global' : '📁 Local';

                        return (
                            <button
                                key={s}
                                className={`px-4 py-1.5 text-sm font-bold whitespace-nowrap transition-all duration-200 rounded-md cursor-pointer border theme-scope-tab ${isActive
                                    ? 'border-[var(--color-border)]'
                                    : 'bg-transparent border-transparent hover:bg-[var(--color-surface-hover)]'
                                }`}
                                style={{
                                    backgroundColor: isActive ? 'var(--color-scope-active-bg)' : undefined,
                                    color: isActive
                                        ? s === 'global' ? 'var(--color-badge-global-text)' : s === 'local' ? 'var(--color-badge-local-text)' : 'var(--color-text)'
                                        : 'var(--color-text-muted)',
                                }}
                                onClick={() => onScopeChange(s)}
                            >
                                {label}
                            </button>
                        );
                    })}
                </div>

                {scope === 'local' && (
                    <div
                        className="flex items-center gap-2 px-3 py-1.5 rounded-md border border-dashed max-w-[320px] animate-fade-in"
                        style={{ backgroundColor: 'var(--color-surface)', borderColor: 'var(--color-border)' }}
                        title={localPath || 'No active folder constraint'}
                    >
                        <span className="text-xs overflow-hidden text-ellipsis whitespace-nowrap font-bold" style={{ color: 'var(--color-text-secondary)' }}>
                            {localPath ? localPath.split(/[/\\]/).pop() : 'All Repos'}
                        </span>
                        <div className="flex gap-1">
                            {localPath && (
                                <button onClick={onClearFolder} className="bg-transparent border-none cursor-pointer text-xs px-1 hover:text-[var(--color-danger)] focus:outline-none transition-colors" style={{ color: 'var(--color-text-muted)' }} title="Clear folder filter">✕</button>
                            )}
                            <button onClick={onSelectFolder} className="bg-transparent border-none cursor-pointer focus:outline-none hover:scale-110 transition-transform text-sm" style={{ color: 'var(--color-badge-global-text)' }} title="Select folder">📂</button>
                        </div>
                    </div>
                )}

                <button
                    className="group px-5 py-2 text-sm font-bold text-white border-none cursor-pointer flex items-center gap-2 btn-press theme-btn-primary"
                    onClick={onAdd}
                >
                    <span className="text-lg inline-block transition-transform duration-300 group-hover:rotate-[-15deg] group-hover:scale-110">＋</span>
                    New Alias
                </button>
            </div>
        </div>
    );
}
