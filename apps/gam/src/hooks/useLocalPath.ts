import { useCallback, useEffect, useState } from 'react';

import { isTauri, tauriAPI } from '#/lib/tauri';

import type { T_ToastType } from './useToast';

export function useLocalPath(addToast: (type: T_ToastType, message: string) => void, fetchAliases: () => Promise<void>) {
    const [localPath, setLocalPath] = useState('');

    useEffect(() => {
        if (isTauri) {
            tauriAPI.getLocalPath().then((res) => {
                if (res.success && res.data) {
                    setLocalPath(res.data);
                }
            });
        }
    }, []);

    const handleSelectFolder = useCallback(async () => {
        if (!isTauri) {
            return;
        }

        try {
            const res = await tauriAPI.selectFolder();

            if (res.success && res.data) {
                setLocalPath(res.data);
                fetchAliases();
            }
        }
        catch (e: any) {
            addToast('error', e.message || 'Failed to select folder');
        }
    }, [fetchAliases, addToast]);

    const handleClearFolder = useCallback(async () => {
        if (!isTauri) {
            return;
        }

        try {
            const res = await tauriAPI.setLocalPath('');

            if (res.success) {
                setLocalPath('');
                fetchAliases();
                addToast('info', 'Viewing all known local repositories');
            }
        }
        catch (e: any) {
            addToast('error', e.message || 'Failed to clear folder filter');
        }
    }, [fetchAliases, addToast]);

    return { localPath, setLocalPath, handleSelectFolder, handleClearFolder };
}
