import type { I_ThemeConfig, T_ThemeId } from '#/types';

import { APP_VERSION, THEME_STYLE_ICONS } from '#/lib/constants';

import { SettingsDropdown } from './SettingsDropdown';

interface I_TitleBarProps {
    themeId: T_ThemeId;
    themeConfig: I_ThemeConfig;
    setThemeId: (id: T_ThemeId) => void;
    previewTheme: (id: T_ThemeId) => void;
    cancelPreview: () => void;
    onOpenThemeSettings: () => void;
    onImport: () => void;
    onExport: () => void;
    aliasCount: number;
    onOpenExternal: (url: string) => void;
}

export function TitleBar({
    themeId,
    themeConfig,
    setThemeId,
    previewTheme,
    cancelPreview,
    onOpenThemeSettings,
    onImport,
    onExport,
    aliasCount,
    onOpenExternal,
}: I_TitleBarProps) {
    return (
        <div className="relative z-[100]">
            <div
                className="titlebar h-11 flex items-center justify-center border-b glass relative"
                style={{
                    borderColor: 'var(--color-border)',
                    backgroundColor: 'var(--color-surface)',
                    backdropFilter: 'var(--theme-card-backdrop)',
                }}
            >
                {/* Red margin line (notebook only) */}
                <div className="nb-margin-line absolute left-10 top-0 bottom-0 w-px" />

                <span className="text-sm font-bold tracking-[2px] uppercase flex items-center gap-1.5" style={{ color: 'var(--color-text)' }}>
                    <span className="px-1" style={{ color: 'var(--color-text)' }}>
                        {THEME_STYLE_ICONS[themeConfig.style] || '◈'}
                        {' '}
                        GAM
                    </span>
                    <span
                        className="text-[10px] font-normal tracking-normal px-1.5 py-px rounded"
                        style={{ color: 'var(--color-text-muted)', backgroundColor: 'var(--color-surface-hover)' }}
                    >
                        v
                        {APP_VERSION}
                    </span>
                </span>

                <SettingsDropdown
                    themeId={themeId}
                    onSelectTheme={setThemeId}
                    onPreviewTheme={previewTheme}
                    onCancelPreview={cancelPreview}
                    onOpenThemeSettings={onOpenThemeSettings}
                    onImport={onImport}
                    onExport={onExport}
                    aliasCount={aliasCount}
                    onOpenExternal={onOpenExternal}
                />
            </div>
            {/* Animated gradient accent strip */}
            <div className="gradient-accent theme-gradient-accent h-[2px] w-full" />
        </div>
    );
}
