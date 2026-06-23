import { useCallback, useEffect, useRef, useState } from 'react';

import { isTauri } from '#/lib/tauri';

interface I_UpdateState {
    checking: boolean;
    updateAvailable: boolean;
    version: string;
    changelog: string;
    downloading: boolean;
    downloadProgress: number;
    error: string | null;
}

const initialState: I_UpdateState = {
    checking: false,
    updateAvailable: false,
    version: '',
    changelog: '',
    downloading: false,
    downloadProgress: 0,
    error: null,
};

export function useUpdater() {
    const [state, setState] = useState<I_UpdateState>(initialState);
    const updateRef = useRef<Awaited<ReturnType<typeof import('@tauri-apps/plugin-updater').check>> | null>(null);

    useEffect(() => {
        if (!isTauri)
            return;

        let cancelled = false;

        async function checkForUpdate() {
            setState(prev => ({ ...prev, checking: true, error: null }));

            try {
                const { check } = await import('@tauri-apps/plugin-updater');
                const update = await check();

                if (cancelled)
                    return;

                if (update) {
                    updateRef.current = update;
                    setState(prev => ({
                        ...prev,
                        checking: false,
                        updateAvailable: true,
                        version: update.version,
                        changelog: update.body ?? '',
                    }));
                }
                else {
                    setState(prev => ({ ...prev, checking: false }));
                }
            }
            catch (err) {
                if (cancelled)
                    return;
                setState(prev => ({
                    ...prev,
                    checking: false,
                    error: err instanceof Error ? err.message : String(err),
                }));
            }
        }

        checkForUpdate();

        return () => {
            cancelled = true;
        };
    }, []);

    const startDownload = useCallback(async () => {
        const update = updateRef.current;
        if (!update)
            return;

        setState(prev => ({ ...prev, downloading: true, downloadProgress: 0, error: null }));

        try {
            let downloaded = 0;
            let contentLength = 1;

            await update.downloadAndInstall((event) => {
                switch (event.event) {
                    case 'Started':
                        contentLength = event.data.contentLength ?? 1;
                        break;
                    case 'Progress':
                        downloaded += event.data.chunkLength;
                        setState(prev => ({
                            ...prev,
                            downloadProgress: Math.min(Math.round((downloaded / contentLength) * 100), 100),
                        }));
                        break;
                    case 'Finished':
                        setState(prev => ({ ...prev, downloadProgress: 100 }));
                        break;
                }
            });

            const { relaunch } = await import('@tauri-apps/plugin-process');
            await relaunch();
        }
        catch (err) {
            setState(prev => ({
                ...prev,
                downloading: false,
                error: err instanceof Error ? err.message : String(err),
            }));
        }
    }, []);

    const dismiss = useCallback(() => {
        updateRef.current = null;
        setState(initialState);
    }, []);

    return { ...state, startDownload, dismiss };
}
