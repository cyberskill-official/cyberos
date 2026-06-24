import { APP_VERSION } from '#/lib/constants';

interface I_AboutPanelProps {
    onOpenExternal: (url: string) => void;
}

export function AboutPanel({ onOpenExternal }: I_AboutPanelProps) {
    return (
        <div
            className="absolute right-0 top-full mt-2 w-[220px] border rounded-xl z-50 overflow-hidden animate-bounce-in shadow-xl no-theme-transition"
            style={{ backgroundColor: 'var(--color-surface-raised)', borderColor: 'var(--color-border)' }}
            role="menu"
        >
            <div className="px-4 py-3 flex flex-col gap-2 text-[12px]" style={{ color: 'var(--color-text-secondary)' }}>
                <div className="flex items-center gap-2">
                    <span className="font-bold text-[14px]" style={{ color: 'var(--color-text)' }}>GAM</span>
                    <span className="px-1.5 py-px rounded text-[10px] font-mono" style={{ backgroundColor: 'var(--color-surface-hover)', color: 'var(--color-text-muted)' }}>
                        v
                        {APP_VERSION}
                    </span>
                </div>
                <div style={{ color: 'var(--color-text-muted)' }}>Git Alias Manager</div>
                <div className="h-px" style={{ backgroundColor: 'var(--color-border)' }} />
                <div className="flex flex-col gap-1.5">
                    <div className="flex items-center gap-2">
                        <span>👤</span>
                        <span>Stephen Cheng</span>
                    </div>
                    <div className="flex items-center gap-2">
                        <span>✉️</span>
                        <button
                            className="bg-transparent border-none p-0 cursor-pointer hover:underline text-[12px]"
                            style={{ color: 'var(--color-primary)' }}
                            onClick={() => onOpenExternal('mailto:zintaen@gmail.com')}
                        >
                            zintaen@gmail.com
                        </button>
                    </div>
                    <div className="flex items-center gap-2">
                        <span>🔗</span>
                        <button
                            className="bg-transparent border-none p-0 cursor-pointer hover:underline text-[12px]"
                            style={{ color: 'var(--color-primary)' }}
                            onClick={() => onOpenExternal('https://github.com/zintaen/gam')}
                        >
                            github.com/zintaen/gam
                        </button>
                    </div>
                    <div className="flex items-center gap-2">
                        <span>📄</span>
                        <span>MIT License</span>
                    </div>
                </div>
            </div>
        </div>
    );
}
