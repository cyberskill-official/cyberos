import { useCallback, useRef, useState } from 'react';

export type T_ToastType = 'success' | 'error' | 'warning' | 'info';

export interface I_Toast {
    id: string;
    type: T_ToastType;
    message: string;
    exiting?: boolean;
}

export function useToast() {
    const [toasts, setToasts] = useState<I_Toast[]>([]);
    const counterRef = useRef(0);

    const addToast = useCallback((type: T_ToastType, message: string) => {
        const id = `toast-${++counterRef.current}`;
        setToasts(prev => [...prev, { id, type, message }]);

        // Auto dismiss after 4 seconds
        setTimeout(() => {
            setToasts(prev =>
                prev.map(t => (t.id === id ? { ...t, exiting: true } : t)),
            );
            setTimeout(() => {
                setToasts(prev => prev.filter(t => t.id !== id));
            }, 200);
        }, 4000);
    }, []);

    const removeToast = useCallback((id: string) => {
        setToasts(prev =>
            prev.map(t => (t.id === id ? { ...t, exiting: true } : t)),
        );
        setTimeout(() => {
            setToasts(prev => prev.filter(t => t.id !== id));
        }, 200);
    }, []);

    return { toasts, addToast, removeToast };
}
