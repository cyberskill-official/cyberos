import { useCallback, useEffect, useState } from 'react';

import { isTauri, tauriAPI } from '#/lib/tauri';

const STORAGE_KEY = 'gam-history-ranking-enabled';

/**
 * Consent state for usage-ranking.
 *
 * Ranking reads the user's shell history (zsh, bash, fish, PowerShell) locally to
 * score aliases by how often they are used. This hook surfaces that to the user and
 * lets them turn it off, persisting the choice to the backend with a localStorage
 * fallback for browser/dev mode. Default is enabled.
 */
export function useHistoryRanking() {
    const [enabled, setEnabledState] = useState<boolean>(() => {
        const stored = localStorage.getItem(STORAGE_KEY);

        return stored === null ? true : stored !== 'false';
    });

    // Load the authoritative value from the backend on mount.
    useEffect(() => {
        if (!isTauri) {
            return;
        }

        tauriAPI.getHistoryRankingEnabled().then((result) => {
            if (result.success && typeof result.data === 'boolean') {
                setEnabledState(result.data);
                localStorage.setItem(STORAGE_KEY, String(result.data));
            }
        }).catch(() => {
            // Keep the localStorage fallback already applied in the initializer.
        });
    }, []);

    const setEnabled = useCallback((next: boolean) => {
        setEnabledState(next);
        localStorage.setItem(STORAGE_KEY, String(next));

        if (isTauri) {
            tauriAPI.setHistoryRankingEnabled(next).catch(() => {
                // Silent fail — localStorage is the fallback.
            });
        }
    }, []);

    return { enabled, setEnabled };
}
