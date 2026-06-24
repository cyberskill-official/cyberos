import { useEffect, useRef, useState } from 'react';

import type { T_ThemeId } from '#/types';

import { AboutPanel } from './settings/AboutPanel';
import { DataPanel } from './settings/DataPanel';
import { PrivacyPanel } from './settings/PrivacyPanel';
import { ThemePanel } from './settings/ThemePanel';

interface I_SettingsDropdownProps {
    themeId: T_ThemeId;
    onSelectTheme: (id: T_ThemeId) => void;
    onPreviewTheme: (id: T_ThemeId) => void;
    onCancelPreview: () => void;
    onOpenThemeSettings: () => void;
    onImport: () => void;
    onExport: () => void;
    aliasCount: number;
    onOpenExternal: (url: string) => void;
}

type T_OpenPanel = 'theme' | 'data' | 'privacy' | 'about' | null;

export function SettingsDropdown({
    themeId,
    onSelectTheme,
    onPreviewTheme,
    onCancelPreview,
    onOpenThemeSettings,
    onImport,
    onExport,
    aliasCount,
    onOpenExternal,
}: I_SettingsDropdownProps) {
    const [openPanel, setOpenPanel] = useState<T_OpenPanel>(null);
    const containerRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        function handleClickOutside(event: MouseEvent) {
            if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
                setOpenPanel(null);
                onCancelPreview();
            }
        }
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, [onCancelPreview]);

    const toggle = (panel: T_OpenPanel) => {
        setOpenPanel((prev) => {
            if (prev !== null && prev !== panel)
                onCancelPreview();
            return prev === panel ? null : panel;
        });
    };

    const close = () => {
        setOpenPanel(null);
        onCancelPreview();
    };

    const btnClass = (active: boolean) =>
        `group/tip flex items-center justify-center h-6 px-1.5 bg-transparent border border-transparent cursor-pointer text-[11px] transition-all duration-200 rounded-md ${active
            ? 'text-[var(--color-text)] bg-[var(--color-surface-hover)] border-[var(--color-border)]'
            : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:bg-[var(--color-surface-hover)]'
        }`;

    const tooltip = (label: string) => (
        <span className="absolute top-full mt-1.5 left-1/2 -translate-x-1/2 px-2 py-0.5 rounded text-[10px] font-bold whitespace-nowrap opacity-0 group-hover/tip:opacity-100 pointer-events-none transition-opacity duration-150 z-[9999]" style={{ backgroundColor: 'var(--color-text)', color: 'var(--color-bg)' }}>
            {label}
        </span>
    );

    return (
        <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1" ref={containerRef}>
            {/* ── Theme button ─── */}
            <div className="relative">
                <button
                    className={btnClass(openPanel === 'theme')}
                    onClick={() => toggle('theme')}
                    aria-haspopup="true"
                    aria-expanded={openPanel === 'theme'}
                >
                    🎨
                    {tooltip('Theme')}
                </button>

                {openPanel === 'theme' && (
                    <ThemePanel
                        themeId={themeId}
                        onSelectTheme={onSelectTheme}
                        onPreviewTheme={onPreviewTheme}
                        onCancelPreview={onCancelPreview}
                        onOpenThemeSettings={onOpenThemeSettings}
                        onClose={close}
                    />
                )}
            </div>

            {/* ── Data button ─── */}
            <div className="relative">
                <button
                    className={btnClass(openPanel === 'data')}
                    onClick={() => toggle('data')}
                    aria-haspopup="true"
                    aria-expanded={openPanel === 'data'}
                >
                    📋
                    {tooltip('Data')}
                </button>

                {openPanel === 'data' && (
                    <DataPanel
                        onImport={onImport}
                        onExport={onExport}
                        aliasCount={aliasCount}
                        onClose={close}
                    />
                )}
            </div>

            {/* ── Privacy button ─── */}
            <div className="relative">
                <button
                    className={btnClass(openPanel === 'privacy')}
                    onClick={() => toggle('privacy')}
                    aria-haspopup="true"
                    aria-expanded={openPanel === 'privacy'}
                >
                    🔒
                    {tooltip('Privacy')}
                </button>

                {openPanel === 'privacy' && <PrivacyPanel />}
            </div>

            {/* ── About button ─── */}
            <div className="relative">
                <button
                    className={btnClass(openPanel === 'about')}
                    onClick={() => toggle('about')}
                    aria-haspopup="true"
                    aria-expanded={openPanel === 'about'}
                >
                    ℹ️
                    {tooltip('About')}
                </button>

                {openPanel === 'about' && (
                    <AboutPanel onOpenExternal={onOpenExternal} />
                )}
            </div>
        </div>
    );
}
