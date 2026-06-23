interface I_DangerZoneWarningsProps {
    warnings: string[];
}

export function DangerZoneWarnings({ warnings }: I_DangerZoneWarningsProps) {
    if (warnings.length === 0)
        return null;

    return (
        <div className="flex flex-col gap-1">
            {warnings.map((w, i) => (
                <div
                    key={i}
                    className="border border-dashed px-3 py-1.5 rounded text-sm flex items-center gap-2 animate-fade-in"
                    style={{ backgroundColor: 'var(--color-warning-muted)', borderColor: 'var(--color-warning)', color: 'var(--color-text)' }}
                >
                    <span>⚠</span>
                    <span className="font-bold">{w}</span>
                </div>
            ))}
        </div>
    );
}
