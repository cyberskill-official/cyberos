import { describe, expect, it } from 'vitest';

import {
    getAllAliases,
    getByCategory,
    getCategories,
    searchLibrary,
} from '#/services/gitalias-library';

describe('gitalias-library', () => {
    describe('getAllAliases', () => {
        it('should return all aliases', () => {
            const all = getAllAliases();
            expect(all.length).toBeGreaterThan(100);
        });

        it('every alias should have name, command, category, and description', () => {
            const all = getAllAliases();
            for (const alias of all) {
                expect(alias.name).toBeTruthy();
                expect(alias.command).toBeTruthy();
                expect(alias.category).toBeTruthy();
                expect(alias.description).toBeTruthy();
            }
        });

        it('should contain well-known short aliases', () => {
            const all = getAllAliases();
            const names = all.map(a => a.name);
            expect(names).toContain('a');
            expect(names).toContain('co');
            expect(names).toContain('ci');
            expect(names).toContain('rb');
            expect(names).toContain('lg');
        });
    });

    describe('getCategories', () => {
        it('should return sorted category list', () => {
            const cats = getCategories();
            expect(cats.length).toBeGreaterThan(5);
            const sorted = [...cats].sort();
            expect(cats).toEqual(sorted);
        });

        it('should contain expected categories', () => {
            const cats = getCategories();
            expect(cats).toContain('add');
            expect(cats).toContain('branch');
            expect(cats).toContain('commit');
            expect(cats).toContain('diff');
            expect(cats).toContain('log');
            expect(cats).toContain('workflow');
        });
    });

    describe('getByCategory', () => {
        it('should filter by category', () => {
            const addAliases = getByCategory('add');
            expect(addAliases.length).toBeGreaterThan(0);
            for (const a of addAliases) {
                expect(a.category).toBe('add');
            }
        });

        it('should return empty for unknown category', () => {
            const result = getByCategory('nonexistent-category');
            expect(result).toHaveLength(0);
        });
    });

    describe('searchLibrary', () => {
        it('should search by alias name', () => {
            const results = searchLibrary('co');
            const names = results.map(a => a.name);
            expect(names).toContain('co');
        });

        it('should search by command content', () => {
            const results = searchLibrary('checkout');
            expect(results.length).toBeGreaterThan(0);
            const hasCheckout = results.some(a => a.command.includes('checkout'));
            expect(hasCheckout).toBe(true);
        });

        it('should search by description', () => {
            const results = searchLibrary('amend');
            expect(results.length).toBeGreaterThan(0);
            const hasAmend = results.some(a => a.description.includes('amend'));
            expect(hasAmend).toBe(true);
        });

        it('should be case-insensitive', () => {
            const upper = searchLibrary('CHECKOUT');
            const lower = searchLibrary('checkout');
            expect(upper.length).toBe(lower.length);
        });

        it('should return all aliases for empty query', () => {
            const all = getAllAliases();
            const result = searchLibrary('');
            expect(result.length).toBe(all.length);
        });

        it('should return all aliases for whitespace query', () => {
            const all = getAllAliases();
            const result = searchLibrary('   ');
            expect(result.length).toBe(all.length);
        });

        it('should search by category', () => {
            const results = searchLibrary('workflow');
            expect(results.length).toBeGreaterThan(0);
            const hasWorkflow = results.some(a => a.category === 'workflow');
            expect(hasWorkflow).toBe(true);
        });
    });
});
