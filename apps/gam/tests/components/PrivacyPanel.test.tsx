import { fireEvent, render, screen } from '@testing-library/react';

import { PrivacyPanel } from '#/components/settings/PrivacyPanel';

vi.mock('#/lib/tauri', () => ({
    isTauri: false,
    tauriAPI: {
        getHistoryRankingEnabled: () => Promise.resolve({ success: true, data: true }),
        setHistoryRankingEnabled: () => Promise.resolve({ success: true, data: true }),
    },
}));

describe('privacyPanel', () => {
    beforeEach(() => {
        localStorage.clear();
    });

    it('discloses what shell-history ranking reads', () => {
        render(<PrivacyPanel />);
        expect(screen.getByText(/shell history/i)).toBeInTheDocument();
        expect(screen.getByText(/Nothing is sent anywhere/i)).toBeInTheDocument();
    });

    it('shows the toggle enabled by default and can turn it off', () => {
        render(<PrivacyPanel />);
        const toggle = screen.getByRole('switch') as HTMLInputElement;
        expect(toggle.checked).toBe(true);

        fireEvent.click(toggle);

        expect(toggle.checked).toBe(false);
        expect(localStorage.getItem('gam-history-ranking-enabled')).toBe('false');
    });
});
