import { useEffect, useMemo, useState } from 'react';

import type { I_GitAlias } from '#/types';

import { SEARCH_DEBOUNCE_MS } from '#/lib/constants';

export function useSearch(aliases: I_GitAlias[]) {
    const [searchQuery, setSearchQuery] = useState('');
    const [debouncedQuery, setDebouncedQuery] = useState('');

    useEffect(() => {
        const handler = setTimeout(() => {
            setDebouncedQuery(searchQuery);
        }, SEARCH_DEBOUNCE_MS);

        return () => clearTimeout(handler);
    }, [searchQuery]);

    const filteredAliases = useMemo(() => {
        if (!debouncedQuery) {
            return aliases;
        }

        const q = debouncedQuery.toLowerCase();

        return aliases.filter(
            a =>
                a.name.toLowerCase().includes(q) || a.command.toLowerCase().includes(q),
        );
    }, [aliases, debouncedQuery]);

    return { searchQuery, setSearchQuery, debouncedQuery, filteredAliases };
}
