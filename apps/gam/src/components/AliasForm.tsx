import * as React from 'react';
import { useCallback, useEffect, useMemo, useState } from 'react';

import type { I_LibraryAlias } from '#/services/gitalias-library';
import type { I_GitAlias } from '#/types';

import { SuggestionService } from '#/services/suggestion-service';

import { CommandEditor } from './alias-form/CommandEditor';
import { DangerZoneWarnings } from './alias-form/DangerZoneWarnings';
import { ScopeSelector } from './alias-form/ScopeSelector';
import { AliasLibraryPicker } from './AliasLibraryPicker';
import { SuggestionChips } from './SuggestionChips';

const DANGEROUS_PATTERNS = [
    { pattern: /rm\s+(-rf|-fr|--recursive)/, message: 'Contains recursive delete (rm -rf)' },
    { pattern: /push\s+(?:\S.*)?--force/, message: 'Contains force push (--force)' },
    { pattern: /push\s+(?:\S.*)?-f\b/, message: 'Contains force push (-f)' },
    { pattern: /reset\s+--hard/, message: 'Contains hard reset (reset --hard)' },
    { pattern: /clean\s+(?:\S.*)?-fd/, message: 'Contains force clean (clean -fd)' },
];

interface I_AliasFormProps {
    alias?: I_GitAlias | null;
    existingNames?: string[];
    onSave: (name: string, command: string, scope: 'global' | 'local', localPath?: string) => Promise<void>;
    onClose: () => void;
    currentScope: 'global' | 'local' | 'all';
    localPath?: string;
    onSelectFolder?: () => Promise<void>;
}

export function AliasForm({
    alias,
    existingNames = [],
    onSave,
    onClose,
    currentScope,
    localPath,
    onSelectFolder,
}: I_AliasFormProps) {
    const isEditing = !!alias;
    const [name, setName] = useState(alias?.name || '');
    const [command, setCommand] = useState(alias?.command || '');
    const [scope, setScope] = useState<'global' | 'local'>(
        alias?.scope || (currentScope === 'local' ? 'local' : 'global'),
    );
    const [saving, setSaving] = useState(false);
    const [nameError, setNameError] = useState('');
    const [commandError, setCommandError] = useState('');
    const [warnings, setWarnings] = useState<string[]>([]);
    const [showLibrary, setShowLibrary] = useState(false);

    const handleLibrarySelect = useCallback((libAlias: I_LibraryAlias) => {
        setCommand(libAlias.command);
        setName(libAlias.name);
        setNameError('');
        setCommandError('');
        setShowLibrary(false);
    }, []);

    const suggestionService = useMemo(() => new SuggestionService(existingNames), [existingNames]);

    const suggestions = useMemo(() => {
        if (isEditing || !command.trim()) {
            return [];
        }

        return suggestionService.suggest(command.trim());
    }, [command, isEditing, suggestionService]);

    const validateName = useCallback((value: string) => {
        if (!value.trim()) {
            setNameError('Alias name is required');

            return false;
        }

        if (!/^[a-z][\w-]*$/i.test(value)) {
            setNameError('Must start with a letter; only letters, numbers, hyphens, and underscores');
            return false;
        }
        setNameError('');

        return true;
    }, []);

    const validateCommand = useCallback((value: string) => {
        if (!value.trim()) {
            setCommandError('Command is required');
            setWarnings([]);

            return false;
        }
        const newWarnings: string[] = [];

        for (const { pattern, message } of DANGEROUS_PATTERNS) {
            if (pattern.test(value)) {
                newWarnings.push(message);
            }
        }
        if (value.startsWith('!')) {
            newWarnings.push('Shell command alias (starts with !). Use with caution.');
        }
        setWarnings(newWarnings);
        setCommandError('');

        return true;
    }, []);

    useEffect(() => {
        if (name) {
            validateName(name);
        }
    }, [name, validateName]);

    useEffect(() => {
        if (command) {
            validateCommand(command);
        }
    }, [command, validateCommand]);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        const nameValid = validateName(name);
        const cmdValid = validateCommand(command);

        if (!nameValid || !cmdValid) {
            return;
        }

        setSaving(true);
        try {
            const targetLocalPath = scope === 'local' ? (alias?.localPath || localPath) : undefined;
            await onSave(name.trim(), command.trim(), scope, targetLocalPath);
            onClose();
        }
        catch (err: unknown) {
            setCommandError(err instanceof Error ? err.message : String(err));
        }
        finally { setSaving(false); }
    };

    useEffect(() => {
        const handler = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                onClose();
            }
        };
        window.addEventListener('keydown', handler);

        return () => window.removeEventListener('keydown', handler);
    }, [onClose]);

    return (
        <div
            className="fixed inset-0 backdrop-blur-[3px] flex items-center justify-center z-[200] animate-fade-in"
            style={{ backgroundColor: 'rgba(0,0,0,0.35)' }}
            onClick={onClose}
        >
            <div
                className="w-[90%] max-w-[560px] flex flex-col overflow-hidden animate-bounce-in rounded-xl border theme-card"
                role="dialog"
                aria-modal="true"
                aria-labelledby="alias-form-title"
                style={{ backgroundColor: 'var(--color-surface-raised)', borderColor: 'var(--color-border)' }}
                onClick={e => e.stopPropagation()}
            >
                {/* Header */}
                <div className="flex justify-between items-center px-6 py-4 border-b" style={{ borderColor: 'var(--color-border)' }}>
                    <h2 className="m-0 text-base font-bold flex items-center gap-2" id="alias-form-title" style={{ color: 'var(--color-text)' }}>
                        <span className="inline-block animate-wiggle">{isEditing ? '✏️' : '📝'}</span>
                        {isEditing
                            ? 'Edit Alias'
                            : 'New Alias'}
                    </h2>
                    <button
                        className="bg-transparent border-none text-lg w-7 h-7 flex items-center justify-center cursor-pointer transition-all duration-200 hover:scale-125 hover:rotate-90"
                        style={{ color: 'var(--color-text-muted)' }}
                        onClick={onClose}
                        aria-label="Close dialog"
                    >
                        ✕
                    </button>
                </div>

                <form onSubmit={handleSubmit}>
                    <div className="flex flex-col gap-5 py-5 px-6 max-h-[70vh] overflow-y-auto">
                        <ScopeSelector
                            scope={scope}
                            onScopeChange={setScope}
                            isEditing={isEditing}
                            localPath={localPath}
                            aliasLocalPath={alias?.localPath}
                            onSelectFolder={onSelectFolder}
                        />

                        {/* Browse Library Button */}
                        <button
                            type="button"
                            className="w-full flex items-center justify-center gap-2 py-2 px-4 text-sm font-bold border border-dashed rounded transition-all duration-200 cursor-pointer btn-press"
                            style={{ color: 'var(--color-badge-global-text)', backgroundColor: 'var(--color-badge-global-bg)', borderColor: 'var(--color-badge-global-border)' }}
                            onClick={() => setShowLibrary(true)}
                        >
                            <span className="text-base">📚</span>
                            Browse Alias Library
                            <span className="text-xs font-normal ml-1" style={{ color: 'var(--color-text-muted)' }}>(GitAlias)</span>
                        </button>

                        <CommandEditor
                            command={command}
                            onCommandChange={setCommand}
                            commandError={commandError}
                            name={name}
                            onNameChange={setName}
                            nameError={nameError}
                            isEditing={isEditing}
                        />

                        {/* Suggestions */}
                        {!isEditing && suggestions.length > 0 && (
                            <SuggestionChips
                                suggestions={suggestions}
                                onSelect={(selectedAlias) => {
                                    setName(selectedAlias);
                                    setNameError('');
                                }}
                            />
                        )}

                        <DangerZoneWarnings warnings={warnings} />
                    </div>

                    {/* Footer */}
                    <div className="flex justify-end gap-3 px-6 py-4 border-t" style={{ borderColor: 'var(--color-border)' }}>
                        <button
                            type="button"
                            className="px-5 py-2 text-sm font-bold bg-transparent border rounded transition-all cursor-pointer btn-press"
                            style={{ color: 'var(--color-text-muted)', borderColor: 'var(--color-border)' }}
                            onClick={onClose}
                        >
                            Cancel
                        </button>
                        <button
                            type="submit"
                            className="px-5 py-2 text-sm font-bold text-white border-none rounded transition-all cursor-pointer btn-press disabled:opacity-40 disabled:cursor-not-allowed"
                            style={{ backgroundColor: 'var(--color-primary)' }}
                            disabled={saving}
                        >
                            {saving ? 'Saving…' : isEditing ? '✓ Save' : '✎ Create Alias'}
                        </button>
                    </div>
                </form>
            </div>

            {showLibrary && (
                <AliasLibraryPicker
                    onSelect={handleLibrarySelect}
                    onClose={() => setShowLibrary(false)}
                />
            )}
        </div>
    );
}
