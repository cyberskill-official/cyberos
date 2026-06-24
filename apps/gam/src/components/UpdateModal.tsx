import { useEffect, useState } from 'react';

interface I_UpdateModalProps {
    version: string;
    changelog: string;
    downloading: boolean;
    downloadProgress: number;
    error: string | null;
    onUpdate: () => void;
    onDismiss: () => void;
}

export function UpdateModal({
    version,
    changelog,
    downloading,
    downloadProgress,
    error,
    onUpdate,
    onDismiss,
}: I_UpdateModalProps) {
    const [entered, setEntered] = useState(false);

    useEffect(() => {
        const handler = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && !downloading)
                onDismiss();
        };
        window.addEventListener('keydown', handler);
        const timer = setTimeout(setEntered, 50, true);

        return () => {
            window.removeEventListener('keydown', handler);
            clearTimeout(timer);
        };
    }, [onDismiss, downloading]);

    return (
        <div
            className="fixed inset-0 backdrop-blur-[3px] flex items-center justify-center z-[200] animate-fade-in"
            style={{ backgroundColor: 'rgba(0,0,0,0.35)' }}
            onClick={downloading ? undefined : onDismiss}
        >
            <div
                className={`w-[90%] max-w-[440px] flex flex-col overflow-hidden rounded-xl border theme-card transition-all duration-300 ${entered ? 'opacity-100 scale-100' : 'opacity-0 scale-95'}`}
                style={{
                    backgroundColor: 'var(--color-surface-raised)',
                    borderColor: 'var(--color-border)',
                }}
                onClick={e => e.stopPropagation()}
            >
                {/* Header */}
                <div className="flex justify-between items-center px-5 py-3 border-b" style={{ borderColor: 'var(--color-border)' }}>
                    <h2 className="m-0 text-base font-bold flex items-center gap-2" style={{ color: 'var(--color-text)' }}>
                        <span className="text-lg">🚀</span>
                        Update Available
                    </h2>
                    {!downloading && (
                        <button
                            className="bg-transparent border-none text-lg w-7 h-7 flex items-center justify-center cursor-pointer hover:scale-125 hover:rotate-90 transition-all duration-200"
                            style={{ color: 'var(--color-text-muted)' }}
                            onClick={onDismiss}
                        >
                            ✕
                        </button>
                    )}
                </div>

                {/* Body */}
                <div className="flex flex-col gap-3 py-4 px-5">
                    {/* Version badge */}
                    <div className="flex items-center gap-2">
                        <span className="text-sm" style={{ color: 'var(--color-text)' }}>New version:</span>
                        <span
                            className="text-xs font-bold px-2 py-0.5 rounded-full"
                            style={{
                                backgroundColor: 'var(--color-primary)',
                                color: 'white',
                            }}
                        >
                            v
                            {version}
                        </span>
                    </div>

                    {/* Changelog */}
                    {changelog && (
                        <div
                            className="text-xs leading-relaxed max-h-[200px] overflow-y-auto px-3 py-2 rounded border border-dashed whitespace-pre-wrap"
                            style={{
                                backgroundColor: 'var(--color-surface)',
                                borderColor: 'var(--color-border)',
                                color: 'var(--color-text-secondary)',
                            }}
                        >
                            {changelog}
                        </div>
                    )}

                    {/* Download progress */}
                    {downloading && (
                        <div className="flex flex-col gap-1.5">
                            <div className="flex justify-between text-xs" style={{ color: 'var(--color-text-muted)' }}>
                                <span>Downloading update…</span>
                                <span>
                                    {downloadProgress}
                                    %
                                </span>
                            </div>
                            <div
                                className="h-2 rounded-full overflow-hidden"
                                style={{ backgroundColor: 'var(--color-surface)' }}
                            >
                                <div
                                    className="h-full rounded-full transition-all duration-300 ease-out"
                                    style={{
                                        width: `${downloadProgress}%`,
                                        background: 'var(--theme-gradient, var(--color-primary))',
                                    }}
                                />
                            </div>
                        </div>
                    )}

                    {/* Error */}
                    {error && (
                        <div
                            className="text-xs px-3 py-2 rounded border"
                            style={{
                                backgroundColor: 'var(--color-danger-muted)',
                                borderColor: 'var(--color-danger)',
                                color: 'var(--color-danger)',
                            }}
                        >
                            Update failed:
                            {' '}
                            {error}
                        </div>
                    )}
                </div>

                {/* Footer */}
                <div className="flex justify-end gap-2.5 px-5 py-3 border-t" style={{ borderColor: 'var(--color-border)' }}>
                    {!downloading && (
                        <button
                            className="px-4 py-1.5 text-sm font-bold bg-transparent border rounded transition-all cursor-pointer btn-press"
                            style={{ color: 'var(--color-text-muted)', borderColor: 'var(--color-border)' }}
                            onClick={onDismiss}
                        >
                            Later
                        </button>
                    )}
                    <button
                        className="px-4 py-1.5 text-sm font-bold text-white border rounded transition-all cursor-pointer btn-press"
                        style={{
                            backgroundColor: 'var(--color-primary)',
                            borderColor: 'var(--color-primary)',
                            opacity: downloading ? 0.6 : 1,
                        }}
                        disabled={downloading}
                        onClick={onUpdate}
                    >
                        {downloading ? 'Installing…' : 'Update Now'}
                    </button>
                </div>
            </div>
        </div>
    );
}
