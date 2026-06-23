import { act, renderHook } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import type { I_GitAlias } from '#/types';

import { useModals } from '#/hooks/useModals';

vi.mock('#/lib/tauri', () => ({
    isTauri: false,
    tauriAPI: {},
}));

describe('useModals', () => {
    const mockAlias: I_GitAlias = { name: 'co', command: 'checkout', scope: 'global' };

    it('starts with all modals closed', () => {
        const { result } = renderHook(() => useModals());
        expect(result.current.showForm).toBe(false);
        expect(result.current.editingAlias).toBeNull();
        expect(result.current.deletingAlias).toBeNull();
        expect(result.current.showThemeSettings).toBe(false);
    });

    it('opens add form', () => {
        const { result } = renderHook(() => useModals());
        act(() => { result.current.openAddForm(); });
        expect(result.current.showForm).toBe(true);
        expect(result.current.editingAlias).toBeNull();
    });

    it('opens edit form with alias', () => {
        const { result } = renderHook(() => useModals());
        act(() => { result.current.openEditForm(mockAlias); });
        expect(result.current.showForm).toBe(true);
        expect(result.current.editingAlias).toEqual(mockAlias);
    });

    it('closes form', () => {
        const { result } = renderHook(() => useModals());
        act(() => { result.current.openAddForm(); });
        expect(result.current.showForm).toBe(true);
        act(() => { result.current.closeForm(); });
        expect(result.current.showForm).toBe(false);
        expect(result.current.editingAlias).toBeNull();
    });

    it('opens delete confirmation', () => {
        const { result } = renderHook(() => useModals());
        act(() => { result.current.openDeleteConfirm(mockAlias); });
        expect(result.current.deletingAlias).toEqual(mockAlias);
    });

    it('closes delete confirmation', () => {
        const { result } = renderHook(() => useModals());
        act(() => { result.current.openDeleteConfirm(mockAlias); });
        act(() => { result.current.closeDeleteConfirm(); });
        expect(result.current.deletingAlias).toBeNull();
    });

    it('opens and closes theme settings', () => {
        const { result } = renderHook(() => useModals());
        act(() => { result.current.openThemeSettings(); });
        expect(result.current.showThemeSettings).toBe(true);
        act(() => { result.current.closeThemeSettings(); });
        expect(result.current.showThemeSettings).toBe(false);
    });

    it('closeAll closes everything', () => {
        const { result } = renderHook(() => useModals());
        act(() => {
            result.current.openAddForm();
            result.current.openDeleteConfirm(mockAlias);
            result.current.openThemeSettings();
        });
        act(() => { result.current.closeAll(); });
        expect(result.current.showForm).toBe(false);
        expect(result.current.editingAlias).toBeNull();
        expect(result.current.deletingAlias).toBeNull();
        expect(result.current.showThemeSettings).toBe(false);
    });
});
