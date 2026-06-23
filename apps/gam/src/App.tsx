import { useCallback, useEffect, useMemo } from 'react';

import { AliasForm } from './components/AliasForm';
import { ConfirmDialog } from './components/ConfirmDialog';
import { ContentArea } from './components/ContentArea';
import { GroupSidebar } from './components/GroupSidebar';
import { ThemeSettingsModal } from './components/ThemeSettingsModal';
import { TitleBar } from './components/TitleBar';
import { ToastContainer } from './components/Toast';
import { UpdateModal } from './components/UpdateModal';
import { useAliasActions } from './hooks/useAliasActions';
import { useAliases } from './hooks/useAliases';
import { useDragDrop } from './hooks/useDragDrop';
import { useGroups } from './hooks/useGroups';
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts';
import { useLocalPath } from './hooks/useLocalPath';
import { useModals } from './hooks/useModals';
import { useSearch } from './hooks/useSearch';
import { useTheme } from './hooks/useTheme';
import { useToast } from './hooks/useToast';
import { useUpdater } from './hooks/useUpdater';
import { isTauri, tauriAPI } from './lib/tauri';

export default function App() {
    // Global error tracking
    useEffect(() => {
        const handleError = (event: ErrorEvent) => {
            console.error('[GAM] Uncaught error:', event.message, event.filename, event.lineno);
        };
        const handleRejection = (event: PromiseRejectionEvent) => {
            console.error('[GAM] Unhandled rejection:', event.reason);
        };

        window.addEventListener('error', handleError);
        window.addEventListener('unhandledrejection', handleRejection);

        // Startup timing
        if (typeof performance !== 'undefined') {
            const renderTime = performance.now();
            console.info(`[GAM] Frontend render in ${renderTime.toFixed(0)}ms`);
        }

        return () => {
            window.removeEventListener('error', handleError);
            window.removeEventListener('unhandledrejection', handleRejection);
        };
    }, []);

    const {
        aliases,
        loading,
        error,
        scope,
        setScope,
        fetchAliases,
        addAlias,
        updateAlias,
        deleteAlias,
    } = useAliases();

    const { toasts, addToast, removeToast } = useToast();
    const { themeId, themeConfig, setThemeId, previewTheme, cancelPreview } = useTheme();
    const { localPath, setLocalPath, handleSelectFolder, handleClearFolder } = useLocalPath(addToast, fetchAliases);
    const { searchQuery, setSearchQuery, debouncedQuery, filteredAliases } = useSearch(aliases);

    const {
        showForm,
        editingAlias,
        deletingAlias,
        showThemeSettings,
        openAddForm,
        openEditForm,
        closeForm,
        openDeleteConfirm,
        closeDeleteConfirm,
        openThemeSettings,
        closeThemeSettings,
        closeAll,
    } = useModals();

    useDragDrop(setLocalPath, fetchAliases, addToast);

    const {
        updateAvailable,
        version: updateVersion,
        changelog,
        downloading,
        downloadProgress,
        error: updateError,
        startDownload,
        dismiss: dismissUpdate,
    } = useUpdater();

    const existingNames = useMemo(() => aliases.map(a => a.name), [aliases]);

    const {
        groups,
        assignments,
        activeGroupId,
        setActiveGroupId,
        createGroup,
        renameGroup,
        setGroupColor,
        deleteGroup,
        setAliasGroups,
    } = useGroups();

    // Count aliases per group
    const groupAliasCounts = useMemo(() => {
        const counts: Record<string, number> = {};
        for (const ids of Object.values(assignments)) {
            for (const id of ids) {
                counts[id] = (counts[id] ?? 0) + 1;
            }
        }
        return counts;
    }, [assignments]);

    // Filter by active group
    const groupFilteredAliases = useMemo(() => {
        if (!activeGroupId)
            return filteredAliases;
        return filteredAliases.filter(a =>
            (assignments[a.name] ?? []).includes(activeGroupId),
        );
    }, [filteredAliases, activeGroupId, assignments]);

    const {
        handleSave,
        handleConfirmDelete,
        handleImport,
        handleExport,
        handleOpenLocalFolder,
    } = useAliasActions({
        editingAlias,
        deletingAlias,
        aliases,
        addAlias,
        updateAlias,
        deleteAlias,
        addToast,
        clearDeletingAlias: closeDeleteConfirm,
    });

    const handleOpenExternal = useCallback(async (url: string) => {
        if (isTauri) {
            await tauriAPI.openExternal(url);
        }
        else {
            window.open(url, '_blank');
        }
    }, []);

    const handleFocusSearch = useCallback(() => {
        const el = document.getElementById('search-input');
        if (el)
            el.focus();
    }, []);

    // Keyboard shortcuts: ⌘N, ⌘F, Escape
    useKeyboardShortcuts({
        onNewAlias: openAddForm,
        onFocusSearch: handleFocusSearch,
        onCloseAll: closeAll,
    });

    const activeGroupName = activeGroupId ? groups.find(g => g.id === activeGroupId)?.name : undefined;

    return (
        <>
            <TitleBar
                themeId={themeId}
                themeConfig={themeConfig}
                setThemeId={setThemeId}
                previewTheme={previewTheme}
                cancelPreview={cancelPreview}
                onOpenThemeSettings={openThemeSettings}
                onImport={handleImport}
                onExport={handleExport}
                aliasCount={aliases.length}
                onOpenExternal={handleOpenExternal}
            />

            {/* Main Content */}
            <div className="flex-1 flex overflow-hidden relative">
                {/* Group sidebar */}
                <div className="sidebar-container shrink-0 border-r px-2 py-3 overflow-y-auto">
                    <GroupSidebar
                        groups={groups}
                        activeGroupId={activeGroupId}
                        onSelectGroup={setActiveGroupId}
                        onCreateGroup={createGroup}
                        onRenameGroup={renameGroup}
                        onSetGroupColor={setGroupColor}
                        onDeleteGroup={deleteGroup}
                        aliasCount={groupAliasCounts}
                    />
                </div>

                <ContentArea
                    scope={scope}
                    onScopeChange={setScope}
                    onAdd={openAddForm}
                    localPath={localPath}
                    onSelectFolder={handleSelectFolder}
                    onClearFolder={handleClearFolder}
                    searchQuery={searchQuery}
                    onSearchChange={setSearchQuery}
                    filteredAliases={filteredAliases}
                    totalAliases={aliases}
                    debouncedQuery={debouncedQuery}
                    groupFilteredAliases={groupFilteredAliases}
                    loading={loading}
                    error={error}
                    fetchAliases={fetchAliases}
                    onEdit={openEditForm}
                    onDelete={openDeleteConfirm}
                    onOpenLocalFolder={handleOpenLocalFolder}
                    groups={groups}
                    assignments={assignments}
                    onSetAliasGroups={setAliasGroups}
                    activeGroupName={activeGroupName}
                    onOpenExternal={handleOpenExternal}
                />
            </div>

            {showForm && (
                <AliasForm
                    alias={editingAlias}
                    existingNames={existingNames}
                    currentScope={scope}
                    localPath={localPath}
                    onSelectFolder={handleSelectFolder}
                    onSave={handleSave}
                    onClose={closeForm}
                />
            )}

            {deletingAlias && (
                <ConfirmDialog
                    title="Delete Alias"
                    message={`Are you sure you want to delete "${deletingAlias.name}"?`}
                    detail={`git ${deletingAlias.name} → ${deletingAlias.command}`}
                    confirmLabel="Delete"
                    confirmDanger
                    onConfirm={handleConfirmDelete}
                    onCancel={closeDeleteConfirm}
                />
            )}

            {showThemeSettings && (
                <ThemeSettingsModal
                    currentThemeId={themeId}
                    onSelect={(id) => {
                        setThemeId(id);
                        closeThemeSettings();
                    }}
                    onPreview={previewTheme}
                    onCancelPreview={cancelPreview}
                    onClose={() => {
                        cancelPreview();
                        closeThemeSettings();
                    }}
                />
            )}

            {updateAvailable && (
                <UpdateModal
                    version={updateVersion}
                    changelog={changelog}
                    downloading={downloading}
                    downloadProgress={downloadProgress}
                    error={updateError}
                    onUpdate={startDownload}
                    onDismiss={dismissUpdate}
                />
            )}

            <ToastContainer toasts={toasts} onDismiss={removeToast} />
        </>
    );
}
