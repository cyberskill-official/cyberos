/** App-level constants — single source of truth for magic strings and values. */

import type { I_ThemeConfig, T_ThemeId, T_ThemeStyle } from '#/types';

declare const __APP_VERSION__: string;

export const APP_VERSION = __APP_VERSION__;
export const THEME_STORAGE_KEY = 'gam-theme';
export const TOAST_DURATION_MS = 4000;
export const SEARCH_DEBOUNCE_MS = 300;
export const SUPPORT_URL = 'https://buymeacoffee.com/zintaen';

export const DEFAULT_THEME_ID: T_ThemeId = 'glassmorphism-dark';

export const THEME_REGISTRY: Record<T_ThemeId, I_ThemeConfig> = {
    'glassmorphism-dark': {
        id: 'glassmorphism-dark',
        style: 'glassmorphism',
        mode: 'dark',
        label: 'Glassmorphism Dark',
        description: 'Frosted glass on a deep gradient backdrop',
    },
    'glassmorphism-light': {
        id: 'glassmorphism-light',
        style: 'glassmorphism',
        mode: 'light',
        label: 'Glassmorphism Light',
        description: 'Frosted glass on a soft gradient backdrop',
    },
    'sketch-dark': {
        id: 'sketch-dark',
        style: 'sketch',
        mode: 'dark',
        label: 'Sketch Dark',
        description: 'Raw pencil sketches on dark craft paper',
    },
    'sketch-light': {
        id: 'sketch-light',
        style: 'sketch',
        mode: 'light',
        label: 'Sketch Light',
        description: 'Ink drawings on graph paper',
    },
    'cybercore-dark': {
        id: 'cybercore-dark',
        style: 'cybercore',
        mode: 'dark',
        label: 'Cybercore Dark',
        description: 'Neon-lit hacker terminal',
    },
    'cybercore-light': {
        id: 'cybercore-light',
        style: 'cybercore',
        mode: 'light',
        label: 'Cybercore Light',
        description: 'Clean tech interface with neon accents',
    },
    'baroque-dark': {
        id: 'baroque-dark',
        style: 'baroque',
        mode: 'dark',
        label: 'Baroque Dark',
        description: 'Opulent gold on deep velvet',
    },
    'baroque-light': {
        id: 'baroque-light',
        style: 'baroque',
        mode: 'light',
        label: 'Baroque Light',
        description: 'Gilded elegance on ivory marble',
    },
    'shabby-dark': {
        id: 'shabby-dark',
        style: 'shabby',
        mode: 'dark',
        label: 'Shabby Chic Dark',
        description: 'Vintage rose on midnight linen',
    },
    'shabby-light': {
        id: 'shabby-light',
        style: 'shabby',
        mode: 'light',
        label: 'Shabby Chic Light',
        description: 'Faded florals on whitewashed wood',
    },
    'gothic-dark': {
        id: 'gothic-dark',
        style: 'gothic',
        mode: 'dark',
        label: 'Gothic Dark',
        description: 'Medieval arches in deep shadow',
    },
    'gothic-light': {
        id: 'gothic-light',
        style: 'gothic',
        mode: 'light',
        label: 'Gothic Light',
        description: 'Stone cathedral with stained glass',
    },
    'victorian-dark': {
        id: 'victorian-dark',
        style: 'victorian',
        mode: 'dark',
        label: 'Victorian Dark',
        description: 'Mahogany parlor with gilt details',
    },
    'victorian-light': {
        id: 'victorian-light',
        style: 'victorian',
        mode: 'light',
        label: 'Victorian Light',
        description: 'Damask wallpaper and brass accents',
    },
    'cottagecore-dark': {
        id: 'cottagecore-dark',
        style: 'cottagecore',
        mode: 'dark',
        label: 'Cottagecore Dark',
        description: 'Fireside cabin at dusk',
    },
    'cottagecore-light': {
        id: 'cottagecore-light',
        style: 'cottagecore',
        mode: 'light',
        label: 'Cottagecore Light',
        description: 'Sun-dappled garden cottage',
    },
    'pixel-dark': {
        id: 'pixel-dark',
        style: 'pixel',
        mode: 'dark',
        label: 'Pixel Art Dark',
        description: 'Retro CRT monitor glow',
    },
    'pixel-light': {
        id: 'pixel-light',
        style: 'pixel',
        mode: 'light',
        label: 'Pixel Art Light',
        description: 'Game Boy-inspired pastel screen',
    },
    'filigree-dark': {
        id: 'filigree-dark',
        style: 'filigree',
        mode: 'dark',
        label: 'Filigree Dark',
        description: 'Silver thread on black velvet',
    },
    'filigree-light': {
        id: 'filigree-light',
        style: 'filigree',
        mode: 'light',
        label: 'Filigree Light',
        description: 'Gold lacework on cream parchment',
    },
};

export const THEME_STYLE_ICONS: Record<T_ThemeStyle, string> = {
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

export const THEME_STYLE_GROUPS: { style: T_ThemeStyle; label: string }[] = [
    { style: 'glassmorphism', label: 'Glassmorphism' },
    { style: 'sketch', label: 'Conceptual Sketch' },
    { style: 'cybercore', label: 'Cybercore' },
    { style: 'baroque', label: 'Baroque' },
    { style: 'shabby', label: 'Shabby Chic' },
    { style: 'gothic', label: 'Gothic' },
    { style: 'victorian', label: 'Victorian' },
    { style: 'cottagecore', label: 'Cottagecore' },
    { style: 'pixel', label: 'Pixel Art' },
    { style: 'filigree', label: 'Filigree' },
];

export const THEME_SWATCHES: Record<T_ThemeId, { bg: string; primary: string; surface: string }> = {
    'glassmorphism-dark': { bg: '#06091a', primary: '#22d3ee', surface: '#111830' },
    'glassmorphism-light': { bg: '#dde4f0', primary: '#0891b2', surface: '#eef2fc' },
    'sketch-dark': { bg: '#181610', primary: '#55cc88', surface: '#222018' },
    'sketch-light': { bg: '#f5f0e2', primary: '#1a6638', surface: '#efe8d6' },
    'cybercore-dark': { bg: '#050505', primary: '#00ff88', surface: '#0c0c0c' },
    'cybercore-light': { bg: '#eaf5ee', primary: '#007744', surface: '#ddeee2' },
    'baroque-dark': { bg: '#080400', primary: '#dab040', surface: '#150e04' },
    'baroque-light': { bg: '#f8f2e4', primary: '#906820', surface: '#f0e6cc' },
    'shabby-dark': { bg: '#161012', primary: '#e09090', surface: '#20181c' },
    'shabby-light': { bg: '#fdf8f4', primary: '#c87878', surface: '#fff9f5' },
    'gothic-dark': { bg: '#060608', primary: '#cc2828', surface: '#100e12' },
    'gothic-light': { bg: '#eee8e4', primary: '#7a0e0e', surface: '#e4dcd4' },
    'victorian-dark': { bg: '#0c0806', primary: '#8b1818', surface: '#181210' },
    'victorian-light': { bg: '#f6eee0', primary: '#6a1010', surface: '#eee2cc' },
    'cottagecore-dark': { bg: '#14100c', primary: '#6aa050', surface: '#1e1814' },
    'cottagecore-light': { bg: '#f6f0e4', primary: '#4a7a30', surface: '#efe6d4' },
    'pixel-dark': { bg: '#18182a', primary: '#00cc66', surface: '#202040' },
    'pixel-light': { bg: '#e0e0ee', primary: '#008844', surface: '#eaeaf6' },
    'filigree-dark': { bg: '#08080e', primary: '#b8a070', surface: '#121218' },
    'filigree-light': { bg: '#f8f6f0', primary: '#7a6838', surface: '#f0ece2' },
};
