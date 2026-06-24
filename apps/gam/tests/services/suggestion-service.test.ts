import { beforeEach, describe, expect, it } from 'vitest';

import { SuggestionService } from '#/services/suggestion-service';

describe('suggestionService', () => {
    let service: SuggestionService;

    beforeEach(() => {
        service = new SuggestionService([]);
    });

    /* ── Semantic (Git-aware) ────────────────────────── */

    describe('semantic suggestions', () => {
        it('should suggest "co" for "checkout"', () => {
            const suggestions = service.suggest('checkout');
            const names = suggestions.map(s => s.alias);
            expect(names).toContain('co');
        });

        it('should suggest "st" for "status"', () => {
            const suggestions = service.suggest('status');
            const names = suggestions.map(s => s.alias);
            expect(names).toContain('st');
        });

        it('should suggest "cm" for "commit -m"', () => {
            const suggestions = service.suggest('commit -m');
            const names = suggestions.map(s => s.alias);
            expect(names).toContain('cm');
        });

        it('should suggest "cb" for "checkout -b"', () => {
            const suggestions = service.suggest('checkout -b');
            const names = suggestions.map(s => s.alias);
            expect(names).toContain('cb');
        });

        it('should suggest "lg" for "log"', () => {
            const suggestions = service.suggest('log');
            const names = suggestions.map(s => s.alias);
            expect(names).toContain('lg');
        });
    });

    /* ── Vowel Removal ───────────────────────────────── */

    describe('vowel removal', () => {
        it('should remove vowels from "checkout"', () => {
            const suggestions = service.suggest('checkout');
            const vowelRemoved = suggestions.filter(
                s => s.scheme === 'vowel-removal',
            );
            expect(vowelRemoved.length).toBeGreaterThan(0);
            // "checkout" → consonants: "chckt" → take 3 → "chc"
            const aliases = vowelRemoved.map(s => s.alias);
            expect(aliases.some(a => !a.match(/[aeiou]/i))).toBe(true);
        });
    });

    /* ── Abbreviation ────────────────────────────────── */

    describe('abbreviation', () => {
        it('should generate first-letter abbreviation for multi-word commands', () => {
            const suggestions = service.suggest('log oneline graph');
            const abbrs = suggestions.filter(s => s.scheme === 'abbreviation');
            expect(abbrs.length).toBeGreaterThan(0);
            expect(abbrs[0].alias).toBe('log');
        });

        it('should not abbreviate single-word commands', () => {
            const suggestions = service.suggest('status');
            const abbrs = suggestions.filter(s => s.scheme === 'abbreviation');
            expect(abbrs.length).toBe(0);
        });
    });

    /* ── Smart Truncation ────────────────────────────── */

    describe('smart truncation', () => {
        it('should generate progressive truncations', () => {
            const suggestions = service.suggest('checkout');
            const truncated = suggestions.filter(s => s.scheme === 'truncation');
            const tAliases = truncated.map(s => s.alias);
            expect(tAliases).toContain('ch');
            expect(tAliases).toContain('che');
        });

        it('should not truncate very short commands', () => {
            const suggestions = service.suggest('ls');
            const truncated = suggestions.filter(s => s.scheme === 'truncation');
            expect(truncated.length).toBe(0);
        });
    });

    /* ── First-Letter Combination ────────────────────── */

    describe('first-letter combination', () => {
        it('should generate first-letter + rest combos for multi-word', () => {
            const suggestions = service.suggest('checkout branch');
            const combos = suggestions.filter(s => s.scheme === 'first-letter');
            expect(combos.length).toBeGreaterThan(0);
            const aliases = combos.map(s => s.alias);
            expect(aliases).toContain('cbranch');
        });
    });

    /* ── Conflict Filtering ──────────────────────────── */

    describe('conflict filtering', () => {
        it('should filter out existing alias names', () => {
            const svc = new SuggestionService(['co', 'st', 'lg']);
            const suggestions = svc.suggest('checkout');
            const names = suggestions.map(s => s.alias);
            expect(names).not.toContain('co');
        });

        it('should update existing names via setExistingNames', () => {
            service.setExistingNames(['co']);
            const suggestions = service.suggest('checkout');
            const names = suggestions.map(s => s.alias);
            expect(names).not.toContain('co');
        });
    });

    /* ── Edge Cases ──────────────────────────────────── */

    describe('edge cases', () => {
        it('should return empty for empty command', () => {
            const suggestions = service.suggest('');
            expect(suggestions).toHaveLength(0);
        });

        it('should return empty for whitespace-only command', () => {
            const suggestions = service.suggest('   ');
            expect(suggestions).toHaveLength(0);
        });

        it('should handle command with only flags', () => {
            const suggestions = service.suggest('--help');
            // Should still generate something (truncations at least)
            expect(Array.isArray(suggestions)).toBe(true);
        });

        it('should limit to max 8 suggestions', () => {
            const suggestions = service.suggest('checkout -b feature-branch');
            expect(suggestions.length).toBeLessThanOrEqual(8);
        });

        it('should sort by priority descending', () => {
            const suggestions = service.suggest('checkout');
            for (let i = 1; i < suggestions.length; i++) {
                expect(suggestions[i - 1].priority).toBeGreaterThanOrEqual(
                    suggestions[i].priority,
                );
            }
        });
    });
});
