interface I_DataPanelProps {
    onImport: () => void;
    onExport: () => void;
    aliasCount: number;
    onClose: () => void;
}

export function DataPanel({ onImport, onExport, aliasCount, onClose }: I_DataPanelProps) {
    return (
        <div
            className="absolute right-0 top-full mt-2 w-[180px] border rounded-xl z-50 overflow-hidden animate-bounce-in shadow-xl no-theme-transition"
            style={{ backgroundColor: 'var(--color-surface-raised)', borderColor: 'var(--color-border)' }}
            role="menu"
        >
            <div className="p-1.5 flex flex-col gap-0.5">
                <button
                    className="w-full text-left px-3 py-2 bg-transparent border-none text-[13px] cursor-pointer rounded-lg transition-colors hover:bg-[var(--color-success-muted)] flex items-center gap-2"
                    style={{ color: 'var(--color-text-secondary)' }}
                    role="menuitem"
                    onClick={() => {
                        onImport();
                        onClose();
                    }}
                >
                    <span>↓</span>
                    {' '}
                    Import JSON
                </button>
                <button
                    className="w-full text-left px-3 py-2 bg-transparent border-none text-[13px] cursor-pointer rounded-lg transition-colors hover:bg-[var(--color-info-muted)] disabled:opacity-30 disabled:cursor-not-allowed flex items-center gap-2"
                    style={{ color: 'var(--color-text-secondary)' }}
                    role="menuitem"
                    onClick={() => {
                        onExport();
                        onClose();
                    }}
                    disabled={aliasCount === 0}
                >
                    <span>↑</span>
                    {' '}
                    Export JSON
                </button>
            </div>
        </div>
    );
}
