import { useEffect, useRef, useState } from 'react';

const PALETTE = [
    '#ef4444',
    '#f97316',
    '#f59e0b',
    '#eab308',
    '#84cc16',
    '#22c55e',
    '#10b981',
    '#06b6d4',
    '#0ea5e9',
    '#3b82f6',
    '#6366f1',
    '#8b5cf6',
    '#a855f7',
    '#d946ef',
    '#ec4899',
    '#f43f5e',
];

export const DEFAULT_COLORS = PALETTE.slice(0, 8);

export function ColorDot({ color, onChange }: { color: string; onChange: (c: string) => void }) {
    const [open, setOpen] = useState(false);
    const [hex, setHex] = useState(color);
    const containerRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!open) {
            return;
        }
        setHex(color);
        const handleClick = (e: MouseEvent) => {
            if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
                setOpen(false);
            }
        };
        document.addEventListener('mousedown', handleClick);
        return () => document.removeEventListener('mousedown', handleClick);
    }, [open, color]);

    const applyHex = () => {
        const cleaned = hex.trim();

        if (/^#[0-9a-f]{6}$/i.test(cleaned)) {
            onChange(cleaned);
            setOpen(false);
        }
    };

    return (
        <div ref={containerRef} className="relative inline-flex items-center justify-center w-8 h-8 flex-shrink-0">
            <button
                type="button"
                className="w-3.5 h-3.5 rounded-full border border-white/20 cursor-pointer hover:scale-125 transition-transform p-0 bg-transparent"
                style={{ backgroundColor: color }}
                onClick={(e) => {
                    e.stopPropagation();
                    setOpen(v => !v);
                }}
                title="Click to change color"
            />
            {open && (
                <div
                    className="absolute left-0 top-full mt-1 z-50 rounded-lg shadow-lg border"
                    style={{
                        backgroundColor: 'var(--color-surface)',
                        borderColor: 'var(--color-border)',
                        width: 132,
                    }}
                >
                    <div className="grid grid-cols-4 gap-2 p-2.5">
                        {PALETTE.map(c => (
                            <button
                                key={c}
                                type="button"
                                className="rounded-full border-2 cursor-pointer p-0 hover:scale-125 transition-transform"
                                style={{
                                    backgroundColor: c,
                                    borderColor: c === color ? 'white' : 'transparent',
                                    width: 20,
                                    height: 20,
                                }}
                                onClick={(e) => {
                                    e.stopPropagation();
                                    onChange(c);
                                    setOpen(false);
                                }}
                            />
                        ))}
                    </div>
                    <div
                        className="flex items-center gap-1.5 px-2.5 py-2 border-t"
                        style={{ borderColor: 'var(--color-border)' }}
                    >
                        <span
                            className="inline-block w-4 h-4 rounded-full border border-white/20 flex-shrink-0"
                            style={{ backgroundColor: /^#[0-9a-f]{6}$/i.test(hex) ? hex : color }}
                        />
                        <input
                            type="text"
                            value={hex}
                            onChange={e => setHex(e.target.value)}
                            onKeyDown={e => e.key === 'Enter' && applyHex()}
                            onBlur={applyHex}
                            placeholder="#ff9900"
                            maxLength={7}
                            className="flex-1 text-xs font-mono px-1.5 py-0.5 rounded border bg-transparent outline-none"
                            style={{
                                color: 'var(--color-text)',
                                borderColor: 'var(--color-border)',
                                width: 70,
                            }}
                            onClick={e => e.stopPropagation()}
                        />
                    </div>
                </div>
            )}
        </div>
    );
}
