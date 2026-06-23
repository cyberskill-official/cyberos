import type { I_Toast } from '#/hooks/useToast';

const ICONS: Record<string, string> = { success: '✓', error: '✗', warning: '⚠', info: 'ℹ' };

interface I_ToastContainerProps {
    toasts: I_Toast[];
    onDismiss: (id: string) => void;
}

const TOAST_STYLES: Record<string, { bg: string; border: string; text: string; progress: string }> = {
    success: {
        bg: 'var(--color-success-muted)',
        border: 'var(--color-success)',
        text: 'var(--color-success)',
        progress: 'var(--color-success)',
    },
    error: {
        bg: 'var(--color-danger-muted)',
        border: 'var(--color-danger)',
        text: 'var(--color-danger)',
        progress: 'var(--color-danger)',
    },
    warning: {
        bg: 'var(--color-warning-muted)',
        border: 'var(--color-warning)',
        text: 'var(--color-text)',
        progress: 'var(--color-warning)',
    },
    info: {
        bg: 'var(--color-info-muted)',
        border: 'var(--color-info)',
        text: 'var(--color-info)',
        progress: 'var(--color-info)',
    },
};

const ACCENT_CLASS: Record<string, string> = {
    success: 'toast-accent--success',
    error: 'toast-accent--error',
    warning: 'toast-accent--warning',
    info: 'toast-accent--info',
};

export function ToastContainer({ toasts, onDismiss }: I_ToastContainerProps) {
    if (toasts.length === 0)
        return null;

    return (
        <div className="fixed bottom-5 right-5 flex flex-col items-end gap-2.5 z-[1000] pointer-events-none">
            {toasts.map((toast) => {
                const s = TOAST_STYLES[toast.type];
                return (
                    <div
                        key={toast.id}
                        className={`toast-accent ${ACCENT_CLASS[toast.type]} flex flex-col pointer-events-auto transition-all duration-300 min-w-[200px] max-w-[360px] animate-bounce-in rounded-lg border ${toast.exiting ? 'opacity-0 scale-95 translate-y-3 pointer-events-none' : ''}`}
                        style={{ backgroundColor: s.bg, borderColor: s.border, color: s.text }}
                    >
                        <div className="flex items-center gap-2.5 px-4 py-2.5 pl-5">
                            <span className="text-lg shrink-0 font-bold">{ICONS[toast.type]}</span>
                            <span className="flex-1 text-sm font-bold">{toast.message}</span>
                            <button className="bg-transparent border-none opacity-50 text-sm cursor-pointer px-0.5 hover:opacity-100 hover:scale-110 transition-all" onClick={() => onDismiss(toast.id)}>✕</button>
                        </div>
                        <div className="h-[2px] w-full px-1 pb-1">
                            <div
                                className="h-full rounded-full opacity-40 animate-progress-shrink"
                                style={{ backgroundColor: s.progress }}
                            />
                        </div>
                    </div>
                );
            })}
        </div>
    );
}
