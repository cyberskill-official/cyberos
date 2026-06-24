import { useHistoryRanking } from '#/hooks';

/**
 * Consent surface for usage-ranking. Discloses that ranking reads local shell
 * history and lets the user turn it off. Self-contained: reads and persists the
 * choice through the useHistoryRanking hook.
 */
export function PrivacyPanel() {
    const { enabled, setEnabled } = useHistoryRanking();

    return (
        <div
            className="absolute right-0 top-full mt-2 w-[260px] border rounded-xl z-50 overflow-hidden animate-bounce-in shadow-xl no-theme-transition"
            style={{ backgroundColor: 'var(--color-surface-raised)', borderColor: 'var(--color-border)' }}
            role="menu"
        >
            <div className="p-3 flex flex-col gap-2">
                <label className="flex items-center justify-between gap-3 cursor-pointer">
                    <span className="text-[13px] font-semibold" style={{ color: 'var(--color-text)' }}>
                        Usage ranking
                    </span>
                    <input
                        type="checkbox"
                        role="switch"
                        aria-label="Enable usage ranking from shell history"
                        className="cursor-pointer w-4 h-4"
                        style={{ accentColor: 'var(--color-accent)' }}
                        checked={enabled}
                        onChange={e => setEnabled(e.target.checked)}
                    />
                </label>
                <p className="text-[11px] leading-snug" style={{ color: 'var(--color-text-muted)' }}>
                    Ranks aliases by reading your shell history (zsh, bash, fish, PowerShell) on this
                    machine. Nothing is sent anywhere. Turn this off to stop reading history and disable
                    usage-based ranking.
                </p>
            </div>
        </div>
    );
}
