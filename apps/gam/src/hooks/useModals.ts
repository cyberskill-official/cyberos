import { useCallback, useState } from 'react';

import type { I_GitAlias } from '#/types';

interface I_ModalState {
    showForm: boolean;
    editingAlias: I_GitAlias | null;
    deletingAlias: I_GitAlias | null;
    showThemeSettings: boolean;
}

export function useModals() {
    const [state, setState] = useState<I_ModalState>({
        showForm: false,
        editingAlias: null,
        deletingAlias: null,
        showThemeSettings: false,
    });

    const openAddForm = useCallback(() => {
        setState(s => ({ ...s, showForm: true, editingAlias: null }));
    }, []);

    const openEditForm = useCallback((alias: I_GitAlias) => {
        setState(s => ({ ...s, showForm: true, editingAlias: alias }));
    }, []);

    const closeForm = useCallback(() => {
        setState(s => ({ ...s, showForm: false }));
    }, []);

    const openDeleteConfirm = useCallback((alias: I_GitAlias) => {
        setState(s => ({ ...s, deletingAlias: alias }));
    }, []);

    const closeDeleteConfirm = useCallback(() => {
        setState(s => ({ ...s, deletingAlias: null }));
    }, []);

    const openThemeSettings = useCallback(() => {
        setState(s => ({ ...s, showThemeSettings: true }));
    }, []);

    const closeThemeSettings = useCallback(() => {
        setState(s => ({ ...s, showThemeSettings: false }));
    }, []);

    const closeAll = useCallback(() => {
        setState({ showForm: false, editingAlias: null, deletingAlias: null, showThemeSettings: false });
    }, []);

    return {
        ...state,
        openAddForm,
        openEditForm,
        closeForm,
        openDeleteConfirm,
        closeDeleteConfirm,
        openThemeSettings,
        closeThemeSettings,
        closeAll,
    };
}
