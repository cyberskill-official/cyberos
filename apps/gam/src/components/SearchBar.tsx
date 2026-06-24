import * as React from 'react';

interface I_SearchBarProps {
    value: string;
    onChange: (value: string) => void;
    resultCount: number;
    totalCount: number;
}

export const SearchBar = React.memo(({
    value,
    onChange,
    totalCount,
}: I_SearchBarProps) => {
    const inputRef = React.useRef<HTMLInputElement>(null);

    return (
        <div className="relative" role="search">
            <span className="absolute left-4 top-1/2 -translate-y-1/2 text-base pointer-events-none select-none transition-transform duration-200" style={{ color: 'var(--color-text-muted)' }} aria-hidden="true">
                🔍
            </span>
            <input
                ref={inputRef}
                type="text"
                placeholder="Search aliases... (⌘F)"
                value={value}
                onChange={e => onChange(e.target.value)}
                className="w-full border py-3.5 pl-11 pr-14 text-[15px] transition-all duration-300 focus-glow focus:outline-none theme-card theme-input"
                style={{
                    backgroundColor: 'var(--color-surface)',
                    borderColor: 'var(--color-border)',
                    color: 'var(--color-text)',
                }}
                id="search-input"
                aria-label="Search aliases"
            />
            {value
                ? (
                        <button
                            className="absolute right-3 top-1/2 -translate-y-1/2 bg-transparent border-none text-base cursor-pointer p-0.5 hover:scale-125 hover:rotate-90 transition-all duration-200 focus:outline-none"
                            style={{ color: 'var(--color-text-muted)' }}
                            onClick={() => onChange('')}
                            aria-label="Clear search"
                        >
                            ✕
                        </button>
                    )
                : (
                        <span
                            className="absolute right-3 top-1/2 -translate-y-1/2 px-1.5 py-px rounded text-[11px] font-mono font-bold pointer-events-none select-none animate-fade-in"
                            style={{ color: 'var(--color-text-muted)', backgroundColor: 'var(--color-surface-hover)' }}
                        >
                            {totalCount}
                        </span>
                    )}
        </div>
    );
});
