/**
 * GitAlias Library Service — Query API
 *
 * Provides query functions over the predefined aliases
 * from https://github.com/GitAlias/gitalias.
 * Static data lives in `gitalias-data.ts`.
 */

import { LIBRARY } from './gitalias-data';

export type { I_LibraryAlias } from './gitalias-data';

/** Get all library aliases. */
export function getAllAliases() {
    return LIBRARY;
}

/** Get unique category names, sorted alphabetically. */
export function getCategories(): string[] {
    const cats = new Set(LIBRARY.map(a => a.category));
    return [...cats].sort();
}

/** Get aliases filtered by category. */
export function getByCategory(category: string) {
    return LIBRARY.filter(a => a.category === category);
}

/** Search aliases by query (matches name, command, category, or description). */
export function searchLibrary(query: string) {
    const q = query.trim().toLowerCase();
    if (!q)
        return LIBRARY;

    return LIBRARY.filter(a =>
        a.name.toLowerCase().includes(q)
        || a.command.toLowerCase().includes(q)
        || a.category.toLowerCase().includes(q)
        || a.description.toLowerCase().includes(q),
    );
}
