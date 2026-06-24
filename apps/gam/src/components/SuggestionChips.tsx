import type { I_AliasSuggestion } from '#/services/suggestion-service';

interface I_SuggestionChipsProps {
    suggestions: I_AliasSuggestion[];
    onSelect: (alias: string) => void;
}

export function SuggestionChips({ suggestions, onSelect }: I_SuggestionChipsProps) {
    if (suggestions.length === 0)
        return null;

    return (
        <div
            className="flex flex-col gap-1.5 p-2.5 border border-dashed rounded animate-fade-in"
            style={{ backgroundColor: 'var(--color-success-muted)', borderColor: 'var(--color-success)' }}
        >
            <div className="text-xs font-bold flex items-center gap-1" style={{ color: 'var(--color-success)' }}>
                <span className="inline-block animate-float text-sm">💡</span>
                Suggested names:
            </div>
            <div className="flex flex-wrap gap-1.5">
                {suggestions.map((s, i) => (
                    <button
                        key={`${s.alias}-${i}`}
                        type="button"
                        className="border px-2 py-0.5 rounded cursor-pointer transition-all duration-200 hover:-translate-y-0.5 hover:scale-105 font-bold btn-press"
                        style={{
                            backgroundColor: 'var(--color-badge-global-bg)',
                            borderColor: 'var(--color-badge-global-border)',
                            color: 'var(--color-badge-global-text)',
                            animation: `chipPopIn 0.3s cubic-bezier(0.34, 1.56, 0.64, 1) ${i * 0.06}s both`,
                        }}
                        onClick={() => onSelect(s.alias)}
                        title={`${s.reason} (${s.scheme})`}
                    >
                        <span className="font-mono text-sm">{s.alias}</span>
                    </button>
                ))}
            </div>
        </div>
    );
}
