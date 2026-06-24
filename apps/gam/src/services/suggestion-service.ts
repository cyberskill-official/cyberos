/**
 * Alias Suggestion Service — adapted from alman's suggestion schemes.
 *
 * Generates multiple alias-name candidates for a given git command,
 * ranked by priority and filtered against existing alias names.
 */

// ── Types ────────────────────────────────────────────────────────

export type T_SuggestionScheme
    = | 'semantic'
        | 'abbreviation'
        | 'vowel-removal'
        | 'first-letter'
        | 'truncation';

export interface I_AliasSuggestion {
    alias: string;
    command: string;
    scheme: T_SuggestionScheme;
    reason: string;
    priority: number;
}

// ── Priority map ─────────────────────────────────────────────────

const SCHEME_PRIORITY: Record<T_SuggestionScheme, number> = {
    'semantic': 100,
    'abbreviation': 90,
    'vowel-removal': 80,
    'first-letter': 70,
    'truncation': 60,
};

// ── Service ──────────────────────────────────────────────────────

export class SuggestionService {
    private existingNames: Set<string>;

    constructor(existingNames: string[] = []) {
        this.existingNames = new Set(existingNames);
    }

    /** Update the set of existing alias names (call after alias list refreshes). */
    setExistingNames(names: string[]): void {
        this.existingNames = new Set(names);
    }

    /** Generate suggestions for a git subcommand string (e.g. "checkout -b"). */
    suggest(command: string): I_AliasSuggestion[] {
        if (!command.trim()) {
            return [];
        }

        const all: I_AliasSuggestion[] = [
            ...this.semantic(command),
            ...this.abbreviation(command),
            ...this.vowelRemoval(command),
            ...this.firstLetter(command),
            ...this.truncation(command),
        ];

        // Filter conflicts & duplicates
        const seen = new Set<string>();
        const filtered = all.filter((s) => {
            if (s.alias.length < 1) {
                return false;
            }

            if (seen.has(s.alias)) {
                return false;
            }

            if (this.existingNames.has(s.alias)) {
                return false;
            }
            seen.add(s.alias);

            return true;
        });

        // Sort by priority descending, then shorter alias first
        filtered.sort((a, b) => {
            if (b.priority !== a.priority) {
                return b.priority - a.priority;
            }

            return a.alias.length - b.alias.length;
        });

        return filtered.slice(0, 8); // top 8 suggestions
    }

    /* ── Scheme 1: Semantic (git-aware) ──────────────────── */

    private semantic(command: string): I_AliasSuggestion[] {
        const parts = command.trim().split(/\s+/);

        if (parts.length === 0) {
            return [];
        }

        const sub = parts[0];
        const flags = parts.slice(1);
        const make = (alias: string, reason: string): I_AliasSuggestion => ({
            alias,
            command,
            scheme: 'semantic',
            reason,
            priority: SCHEME_PRIORITY.semantic,
        });

        const suggestions: I_AliasSuggestion[] = [];

        // Well-known git subcommand mappings
        const knownMap: Record<string, { alias: string; reason: string }> = {
            status: { alias: 'st', reason: 'Git status' },
            checkout: { alias: 'co', reason: 'Git checkout' },
            commit: { alias: 'ci', reason: 'Git commit' },
            branch: { alias: 'br', reason: 'Git branch' },
            push: { alias: 'ps', reason: 'Git push' },
            pull: { alias: 'pl', reason: 'Git pull' },
            merge: { alias: 'mg', reason: 'Git merge' },
            rebase: { alias: 'rb', reason: 'Git rebase' },
            stash: { alias: 'sh', reason: 'Git stash' },
            diff: { alias: 'df', reason: 'Git diff' },
            log: { alias: 'lg', reason: 'Git log' },
            fetch: { alias: 'fe', reason: 'Git fetch' },
            remote: { alias: 'rm', reason: 'Git remote' },
            reset: { alias: 'rs', reason: 'Git reset' },
            cherry: { alias: 'cp', reason: 'Git cherry-pick' },
        };

        const entry = knownMap[sub];
        if (entry) {
            suggestions.push(make(entry.alias, entry.reason));
        }

        // Flag-aware variants
        if (sub === 'commit' && flags.includes('-m')) {
            suggestions.push(make('cm', 'Git commit -m'));
        }
        if (sub === 'checkout' && flags.includes('-b')) {
            suggestions.push(make('cb', 'Git checkout -b'));
        }
        if (
            sub === 'log'
            && (flags.includes('--oneline') || flags.includes('--graph'))
        ) {
            suggestions.push(make('lo', 'Git log --oneline'));
        }
        if (sub === 'push' && (flags.includes('--force') || flags.includes('-f'))) {
            suggestions.push(make('pf', 'Git push --force'));
        }
        if (sub === 'stash' && flags[0] === 'pop') {
            suggestions.push(make('sp', 'Git stash pop'));
        }

        return suggestions;
    }

    /* ── Scheme 2: Abbreviation (first letters) ──────────── */

    private abbreviation(command: string): I_AliasSuggestion[] {
        const parts = command
            .trim()
            .split(/\s+/)
            .filter(p => !p.startsWith('-'));

        if (parts.length < 2) {
            return [];
        }

        const abbr = parts.map(w => w[0]).join('');

        if (abbr.length < 2 || abbr.length > 4) {
            return [];
        }

        return [
            {
                alias: abbr,
                command,
                scheme: 'abbreviation',
                reason: 'First-letter abbreviation',
                priority: SCHEME_PRIORITY.abbreviation,
            },
        ];
    }

    /* ── Scheme 3: Vowel Removal ─────────────────────────── */

    private vowelRemoval(command: string): I_AliasSuggestion[] {
        const parts = command
            .trim()
            .split(/\s+/)
            .filter(p => !p.startsWith('-'));
        if (parts.length === 0) {
            return [];
        }

        const processed = parts
            .map((word) => {
                const consonants = word.replace(/[aeiou]/gi, '');

                return consonants.slice(0, 3);
            })
            .filter(Boolean);

        if (processed.length === 0) {
            return [];
        }

        let alias = processed.join('');
        if (alias.length > 6) {
            alias = alias.slice(0, 6);
        }

        if (alias.length < 2 || alias === command.replace(/\s+/g, '')) {
            return [];
        }

        return [
            {
                alias,
                command,
                scheme: 'vowel-removal',
                reason: 'Vowel removal',
                priority: SCHEME_PRIORITY['vowel-removal'],
            },
        ];
    }

    /* ── Scheme 4: First-Letter Combination ──────────────── */

    private firstLetter(command: string): I_AliasSuggestion[] {
        const parts = command
            .trim()
            .split(/\s+/)
            .filter(p => !p.startsWith('-'));

        if (parts.length < 2) {
            return [];
        }

        const suggestions: I_AliasSuggestion[] = [];

        // first char of first word + second word
        const combo = parts[0][0] + parts.slice(1).join('');

        if (combo.length >= 2 && combo.length <= 8) {
            suggestions.push({
                alias: combo,
                command,
                scheme: 'first-letter',
                reason: 'First-letter combination',
                priority: SCHEME_PRIORITY['first-letter'],
            });
        }

        // first char of first word + first 2 chars of second word
        if (parts[1] && parts[1].length >= 2) {
            const short = parts[0][0] + parts[1].slice(0, 2);
            suggestions.push({
                alias: short,
                command,
                scheme: 'first-letter',
                reason: 'Short combination',
                priority: SCHEME_PRIORITY['first-letter'] - 5,
            });
        }

        return suggestions;
    }

    /* ── Scheme 5: Smart Truncation ──────────────────────── */

    private truncation(command: string): I_AliasSuggestion[] {
        const parts = command
            .trim()
            .split(/\s+/)
            .filter(p => !p.startsWith('-'));

        if (parts.length === 0) {
            return [];
        }

        const word = parts[0];

        if (word.length <= 2) {
            return [];
        }

        const suggestions: I_AliasSuggestion[] = [];

        for (let len = 2; len <= Math.min(word.length - 1, 4); len++) {
            const trunc = word.slice(0, len);
            suggestions.push({
                alias: trunc,
                command,
                scheme: 'truncation',
                reason: `Truncated to ${len} chars`,
                priority: SCHEME_PRIORITY.truncation - (len - 2),
            });
        }

        return suggestions;
    }
}
