import type { I_ThemeConfig, T_ThemeId } from '#/types';

import { THEME_REGISTRY, THEME_STYLE_GROUPS, THEME_STYLE_ICONS, THEME_SWATCHES } from '#/lib/constants';

interface I_ThemePanelProps {
    themeId: T_ThemeId;
    onSelectTheme: (id: T_ThemeId) => void;
    onPreviewTheme: (id: T_ThemeId) => void;
    onCancelPreview: () => void;
    onOpenThemeSettings: () => void;
    onClose: () => void;
}

export function ThemePanel({
    themeId,
    onSelectTheme,
    onPreviewTheme,
    onCancelPreview,
    onOpenThemeSettings,
    onClose,
}: I_ThemePanelProps) {
    const themes = Object.values(THEME_REGISTRY);

    return (
        <div
            className="absolute right-0 top-full mt-2 w-[280px] border rounded-xl z-50 overflow-hidden animate-bounce-in shadow-xl no-theme-transition"
            style={{ backgroundColor: 'var(--color-surface-raised)', borderColor: 'var(--color-border)', maxHeight: 'calc(100vh - 70px)' }}
            role="menu"
        >
            <div className="overflow-y-auto" style={{ maxHeight: 'calc(100vh - 70px)' }}>
                <div
                    className="px-1.5 py-2 flex flex-col gap-0.5"
                    onMouseLeave={onCancelPreview}
                >
                    {THEME_STYLE_GROUPS.map((group) => {
                        const groupThemes = themes.filter(t => t.style === group.style);
                        return (
                            <div key={group.style}>
                                <div className="px-2.5 py-1 text-[10px] font-bold uppercase tracking-wider text-[var(--color-text-muted)]">
                                    {THEME_STYLE_ICONS[group.style]}
                                    {' '}
                                    {group.label}
                                </div>
                                <div className="flex gap-1 px-1.5 pb-1">
                                    {groupThemes.map((t: I_ThemeConfig) => {
                                        const swatch = THEME_SWATCHES[t.id];
                                        const isActive = t.id === themeId;
                                        return (
                                            <button
                                                key={t.id}
                                                className={`flex-1 rounded-lg border-2 cursor-pointer transition-all duration-150 overflow-hidden p-0 ${isActive
                                                    ? 'border-[var(--color-primary)] scale-[1.02]'
                                                    : 'border-transparent hover:border-[var(--color-border-strong)]'
                                                }`}
                                                style={{ background: 'none' }}
                                                role="menuitem"
                                                onMouseEnter={() => onPreviewTheme(t.id)}
                                                onClick={() => {
                                                    onSelectTheme(t.id);
                                                    onClose();
                                                }}
                                            >
                                                <div className="h-[28px] relative" style={{ backgroundColor: swatch.bg }}>
                                                    <div
                                                        className="absolute inset-[4px] rounded-[2px]"
                                                        style={{ backgroundColor: swatch.surface, border: `1px solid ${swatch.primary}33` }}
                                                    >
                                                        <div className="absolute top-[3px] left-[4px] right-[4px] h-[2px] rounded-full" style={{ backgroundColor: swatch.primary, opacity: 0.7 }} />
                                                        <div className="absolute bottom-[3px] left-[4px] w-[40%] h-[1px] rounded-full" style={{ backgroundColor: swatch.primary, opacity: 0.3 }} />
                                                    </div>
                                                    {isActive && (
                                                        <div className="absolute top-[2px] right-[2px] w-[10px] h-[10px] rounded-full flex items-center justify-center text-[6px] font-bold text-white" style={{ backgroundColor: swatch.primary }}>✓</div>
                                                    )}
                                                </div>
                                                <div className="text-[10px] py-1 text-center font-bold" style={{ color: 'var(--color-text-secondary)', backgroundColor: 'var(--color-surface)' }}>
                                                    {t.mode === 'dark' ? '🌙' : '☀️'}
                                                </div>
                                            </button>
                                        );
                                    })}
                                </div>
                            </div>
                        );
                    })}
                    <div className="border-t border-[var(--color-border)] mt-1 pt-1">
                        <button
                            className="w-full text-left px-3 py-1.5 bg-transparent border-none text-[12px] text-[var(--color-text-muted)] cursor-pointer rounded-lg hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)] transition-colors"
                            onClick={() => {
                                onOpenThemeSettings();
                                onClose();
                            }}
                        >
                            ⚙ All Themes…
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
