export interface I_GitAlias {
    name: string;
    command: string;
    scope: 'global' | 'local';
    localPath?: string;
    score?: number;
}

export type T_SortMode = 'name' | 'rank';

export interface I_AliasFormData {
    name: string;
    command: string;
    scope: 'global' | 'local';
}

export interface I_IpcResult<T = unknown> {
    success: boolean;
    data?: T;
    error?: string;
}

export interface I_ValidationResult {
    valid: boolean;
    warnings: string[];
    errors: string[];
}

export interface I_ExportData {
    version: string;
    exportedAt: string;
    aliases: I_GitAlias[];
    groups?: I_AliasGroup[];
    assignments?: Record<string, string[]>;
}

export interface I_AliasGroup {
    id: string;
    name: string;
    color: string;
}

export interface I_AppAPI {
    getAliases: (
        scope: 'global' | 'local' | 'all',
    ) => Promise<I_IpcResult<I_GitAlias[]>>;
    addAlias: (
        name: string,
        command: string,
        scope: 'global' | 'local',
        localPath?: string,
    ) => Promise<I_IpcResult>;
    updateAlias: (
        oldName: string,
        name: string,
        command: string,
        scope: 'global' | 'local',
        localPath?: string,
    ) => Promise<I_IpcResult>;
    deleteAlias: (name: string, scope: 'global' | 'local', localPath?: string) => Promise<I_IpcResult>;
    validateCommand: (command: string) => Promise<I_IpcResult<I_ValidationResult>>;
    exportAliases: (aliases: I_GitAlias[]) => Promise<I_IpcResult<string>>;
    importAliases: () => Promise<I_IpcResult<I_GitAlias[]>>;
    selectFolder: () => Promise<I_IpcResult<string>>;
    getLocalPath: () => Promise<I_IpcResult<string>>;
    setLocalPath: (path: string) => Promise<I_IpcResult<string>>;
    openLocalFolder: (path: string) => Promise<I_IpcResult>;
    openExternal: (url: string) => Promise<I_IpcResult>;
    getTheme: () => Promise<I_IpcResult<string>>;
    setTheme: (themeId: string) => Promise<I_IpcResult>;
    getHistoryRankingEnabled: () => Promise<I_IpcResult<boolean>>;
    setHistoryRankingEnabled: (enabled: boolean) => Promise<I_IpcResult<boolean>>;
    // Groups
    getGroups: () => Promise<I_IpcResult<I_AliasGroup[]>>;
    createGroup: (name: string, color: string) => Promise<I_IpcResult<I_AliasGroup>>;
    renameGroup: (groupId: string, newName: string) => Promise<I_IpcResult>;
    setGroupColor: (groupId: string, color: string) => Promise<I_IpcResult>;
    deleteGroup: (groupId: string) => Promise<I_IpcResult>;
    setAliasGroups: (aliasName: string, groupIds: string[]) => Promise<I_IpcResult>;
    getAllGroupAssignments: () => Promise<I_IpcResult<Record<string, string[]>>>;
}

// ── Theme System ────────────────────────────────────────────

export type T_ThemeStyle = 'glassmorphism' | 'sketch' | 'cybercore' | 'baroque' | 'shabby' | 'gothic' | 'victorian' | 'cottagecore' | 'pixel' | 'filigree';
export type T_ThemeMode = 'light' | 'dark';
export type T_ThemeId
    = | 'glassmorphism-dark'
        | 'glassmorphism-light'
        | 'sketch-dark'
        | 'sketch-light'
        | 'cybercore-dark'
        | 'cybercore-light'
        | 'baroque-dark'
        | 'baroque-light'
        | 'shabby-dark'
        | 'shabby-light'
        | 'gothic-dark'
        | 'gothic-light'
        | 'victorian-dark'
        | 'victorian-light'
        | 'cottagecore-dark'
        | 'cottagecore-light'
        | 'pixel-dark'
        | 'pixel-light'
        | 'filigree-dark'
        | 'filigree-light';

export interface I_ThemeConfig {
    id: T_ThemeId;
    style: T_ThemeStyle;
    mode: T_ThemeMode;
    label: string;
    description: string;
}

declare global {
    interface Window {
        __TAURI_INTERNALS__?: Record<string, unknown>;
    }
}
