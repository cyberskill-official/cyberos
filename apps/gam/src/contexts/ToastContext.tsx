import { createContext, use } from 'react';

import type { I_Toast, T_ToastType } from '#/hooks/useToast';

import { useToast } from '#/hooks/useToast';

interface I_ToastContextValue {
    toasts: I_Toast[];
    addToast: (type: T_ToastType, message: string) => void;
    removeToast: (id: string) => void;
}

const ToastContext = createContext<I_ToastContextValue | null>(null);

export function ToastProvider({ children }: { children: React.ReactNode }) {
    const toast = useToast();

    return (
        <ToastContext value={toast}>
            {children}
        </ToastContext>
    );
}

export function useToastContext() {
    const ctx = use(ToastContext);
    if (!ctx) {
        throw new Error('useToastContext must be used within <ToastProvider>');
    }
    return ctx;
}
