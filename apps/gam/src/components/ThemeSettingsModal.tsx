import { useEffect } from 'react';

import type { I_ThemeConfig, T_ThemeId } from '#/types';

import { THEME_REGISTRY } from '#/lib/constants';

interface I_ThemeSettingsModalProps {
    currentThemeId: T_ThemeId;
    onSelect: (id: T_ThemeId) => void;
    onPreview: (id: T_ThemeId) => void;
    onCancelPreview: () => void;
    onClose: () => void;
}

const STYLE_ICONS: Record<string, string> = {
    glassmorphism: '◈',
    sketch: '✎',
    cybercore: '⌬',
    baroque: '♛',
    shabby: '❀',
    gothic: '⛧',
    victorian: '⚜',
    cottagecore: '🌿',
    pixel: '▦',
    filigree: '❋',
};

const SWATCHES: Record<string, { bg: string; surface: string; primary: string; text: string }> = {
    'glassmorphism-dark': { bg: '#06091a', surface: '#111830', primary: '#22d3ee', text: '#e2e8f0' },
    'glassmorphism-light': { bg: '#dde4f0', surface: '#eef2fc', primary: '#0891b2', text: '#0f172a' },
    'sketch-dark': { bg: '#181610', surface: '#222018', primary: '#55cc88', text: '#efe0c8' },
    'sketch-light': { bg: '#f5f0e2', surface: '#efe8d6', primary: '#1a6638', text: '#1a1008' },
    'cybercore-dark': { bg: '#050505', surface: '#0c0c0c', primary: '#00ff88', text: '#00ff88' },
    'cybercore-light': { bg: '#eaf5ee', surface: '#ddeee2', primary: '#007744', text: '#061a0e' },
    'baroque-dark': { bg: '#080400', surface: '#150e04', primary: '#dab040', text: '#f0d8a0' },
    'baroque-light': { bg: '#f8f2e4', surface: '#f0e6cc', primary: '#906820', text: '#221004' },
    'shabby-dark': { bg: '#161012', surface: '#20181c', primary: '#e09090', text: '#f2dce2' },
    'shabby-light': { bg: '#fdf8f4', surface: '#fff9f5', primary: '#c87878', text: '#4a2e2e' },
    'gothic-dark': { bg: '#060608', surface: '#100e12', primary: '#cc2828', text: '#ccc8c0' },
    'gothic-light': { bg: '#eee8e4', surface: '#e4dcd4', primary: '#7a0e0e', text: '#141418' },
    'victorian-dark': { bg: '#0c0806', surface: '#181210', primary: '#8b1818', text: '#e8d8c0' },
    'victorian-light': { bg: '#f6eee0', surface: '#eee2cc', primary: '#6a1010', text: '#181004' },
    'cottagecore-dark': { bg: '#14100c', surface: '#1e1814', primary: '#6aa050', text: '#e8d8c0' },
    'cottagecore-light': { bg: '#f6f0e4', surface: '#efe6d4', primary: '#4a7a30', text: '#302418' },
    'pixel-dark': { bg: '#18182a', surface: '#202040', primary: '#00cc66', text: '#e0e0ff' },
    'pixel-light': { bg: '#e0e0ee', surface: '#eaeaf6', primary: '#008844', text: '#181830' },
    'filigree-dark': { bg: '#08080e', surface: '#121218', primary: '#b8a070', text: '#c8c4b8' },
    'filigree-light': { bg: '#f8f6f0', surface: '#f0ece2', primary: '#7a6838', text: '#222018' },
};

export function ThemeSettingsModal({
    currentThemeId,
    onSelect,
    onPreview,
    onCancelPreview,
    onClose,
}: I_ThemeSettingsModalProps) {
    useEffect(() => {
        function handleKeyDown(e: KeyboardEvent) {
            if (e.key === 'Escape')
                onClose();
        }
        document.addEventListener('keydown', handleKeyDown);
        return () => document.removeEventListener('keydown', handleKeyDown);
    }, [onClose]);

    const themes = Object.values(THEME_REGISTRY);

    return (
        <div
            className="fixed inset-0 z-[300] flex items-center justify-center animate-fade-in"
            style={{ backgroundColor: 'rgba(0,0,0,0.4)', backdropFilter: 'blur(3px)' }}
            onClick={(e) => {
                if (e.target === e.currentTarget)
                    onClose();
            }}
            role="dialog"
            aria-modal="true"
            aria-labelledby="theme-settings-title"
        >
            <div
                className="w-[92%] max-w-[720px] max-h-[85vh] flex flex-col overflow-hidden animate-bounce-in rounded-2xl border"
                style={{
                    backgroundColor: 'var(--color-surface-raised)',
                    borderColor: 'var(--color-border)',
                }}
                onMouseLeave={onCancelPreview}
            >
                {/* Header */}
                <div className="flex justify-between items-center px-6 py-4 border-b" style={{ borderColor: 'var(--color-border)' }}>
                    <h2 className="m-0 text-base font-bold flex items-center gap-2" id="theme-settings-title" style={{ color: 'var(--color-text)' }}>
                        ⚙ Theme Settings
                    </h2>
                    <button
                        className="bg-transparent border-none text-lg w-7 h-7 flex items-center justify-center cursor-pointer transition-all duration-200 hover:scale-125 hover:rotate-90"
                        style={{ color: 'var(--color-text-muted)' }}
                        onClick={onClose}
                        aria-label="Close"
                    >
                        ✕
                    </button>
                </div>

                {/* Grid */}
                <div className="p-6 overflow-y-auto">
                    <div className="grid grid-cols-2 gap-3 sm:grid-cols-4">
                        {themes.map((t: I_ThemeConfig) => {
                            const swatch = SWATCHES[t.id];
                            const isActive = t.id === currentThemeId;

                            return (
                                <button
                                    key={t.id}
                                    className="text-left border-2 rounded-xl p-0 overflow-hidden cursor-pointer transition-all duration-200 hover:-translate-y-1 btn-press group"
                                    style={{
                                        backgroundColor: 'var(--color-surface)',
                                        borderColor: isActive ? 'var(--color-primary)' : 'var(--color-border)',
                                    }}
                                    onMouseEnter={() => onPreview(t.id)}
                                    onClick={() => onSelect(t.id)}
                                >
                                    {/* Swatch preview */}
                                    <div
                                        className="h-[56px] relative overflow-hidden"
                                        style={{ backgroundColor: swatch.bg }}
                                    >
                                        {/* Mini card mockup */}
                                        <div
                                            className="absolute top-2 left-2 right-2 bottom-2 rounded-sm flex items-center gap-1.5 px-1.5"
                                            style={{ backgroundColor: swatch.surface, border: `1px solid rgba(128,128,128,0.15)` }}
                                        >
                                            <div className="w-1.5 h-1.5 rounded-full" style={{ backgroundColor: swatch.primary }} />
                                            <div className="flex-1 flex flex-col gap-0.5">
                                                <div className="h-1 rounded-full w-3/4" style={{ backgroundColor: swatch.text, opacity: 0.6 }} />
                                                <div className="h-0.5 rounded-full w-1/2" style={{ backgroundColor: swatch.text, opacity: 0.3 }} />
                                            </div>
                                        </div>

                                        {/* Active checkmark */}
                                        {isActive && (
                                            <div
                                                className="absolute top-1 right-1 w-4 h-4 rounded-full flex items-center justify-center text-[9px] font-bold text-white"
                                                style={{ backgroundColor: swatch.primary }}
                                            >
                                                ✓
                                            </div>
                                        )}
                                    </div>

                                    {/* Label */}
                                    <div className="px-2 py-1.5">
                                        <div className="text-[11px] font-bold flex items-center gap-1" style={{ color: 'var(--color-text)' }}>
                                            <span className="text-[10px]">{STYLE_ICONS[t.style]}</span>
                                            {t.label}
                                        </div>
                                    </div>
                                </button>
                            );
                        })}
                    </div>
                </div>
            </div>
        </div>
    );
}
