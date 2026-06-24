import { useEffect } from 'react';

import { isTauri, tauriAPI } from '#/lib/tauri';

import type { T_ToastType } from './useToast';

export function useDragDrop(
    setLocalPath: (path: string) => void,
    fetchAliases: () => Promise<void>,
    addToast: (type: T_ToastType, message: string) => void,
) {
    useEffect(() => {
        const handleDrop = async (e: DragEvent) => {
            e.preventDefault();
            e.stopPropagation();
            if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
                const path = (e.dataTransfer.files[0] as any).path;

                if (path && isTauri) {
                    const res = await tauriAPI.setLocalPath(path);

                    if (res.success) {
                        setLocalPath(path);
                        fetchAliases();
                        addToast('success', `Selected folder: ${path.split('/').pop()}`);
                    }
                }
            }
        };
        const handleDragOver = (e: DragEvent) => {
            e.preventDefault();
            e.stopPropagation();
        };

        window.addEventListener('drop', handleDrop);
        window.addEventListener('dragover', handleDragOver);

        return () => {
            window.removeEventListener('drop', handleDrop);
            window.removeEventListener('dragover', handleDragOver);
        };
    }, [setLocalPath, fetchAliases, addToast]);
}
