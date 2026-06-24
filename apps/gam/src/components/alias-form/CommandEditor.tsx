interface I_CommandEditorProps {
    command: string;
    onCommandChange: (value: string) => void;
    commandError: string;
    name: string;
    onNameChange: (value: string) => void;
    nameError: string;
    isEditing: boolean;
}

export function CommandEditor({
    command,
    onCommandChange,
    commandError,
    name,
    onNameChange,
    nameError,
    isEditing,
}: I_CommandEditorProps) {
    return (
        <div className="flex items-start gap-4">
            <div className="flex-[2] flex flex-col ink-underline">
                <label className="text-[11px] font-bold uppercase tracking-wider mb-2" style={{ color: 'var(--color-text-muted)' }} htmlFor="aliasCommand">Command</label>
                <textarea
                    id="aliasCommand"
                    className="w-full border-2 rounded py-2 px-3 text-[15px] transition-all duration-200 focus-glow focus:outline-none resize-y font-mono min-h-[40px] theme-input"
                    style={{
                        backgroundColor: 'var(--color-surface)',
                        borderColor: commandError ? 'var(--color-danger)' : 'var(--color-border)',
                        color: 'var(--color-text)',
                    }}
                    placeholder="e.g. checkout, status -sb"
                    value={command}
                    onChange={e => onCommandChange(e.target.value)}
                    autoFocus={!isEditing}
                    rows={command.includes('\n') ? Math.min(command.split('\n').length, 6) : 1}
                    aria-invalid={!!commandError}
                    aria-describedby={commandError ? 'command-error' : undefined}
                />
                {commandError && (
                    <div id="command-error" className="text-xs mt-1 font-bold animate-fade-in" style={{ color: 'var(--color-danger)' }} aria-live="polite">
                        ✗
                        {' '}
                        {commandError}
                    </div>
                )}
            </div>

            <div className="flex-1 flex flex-col ink-underline">
                <label className="text-[11px] font-bold uppercase tracking-wider mb-2" style={{ color: 'var(--color-text-muted)' }} htmlFor="aliasName">Alias Name</label>
                <div className="relative flex items-center">
                    <span className="absolute left-3 font-mono text-sm font-bold pointer-events-none" style={{ color: 'var(--color-text-muted)' }}>git</span>
                    <input
                        id="aliasName"
                        type="text"
                        className="w-full border-2 rounded py-2 pl-10 pr-3 text-[15px] transition-all duration-200 focus-glow focus:outline-none theme-input"
                        style={{
                            backgroundColor: 'var(--color-surface)',
                            borderColor: nameError ? 'var(--color-danger)' : 'var(--color-border)',
                            color: 'var(--color-text)',
                        }}
                        placeholder="e.g. co"
                        value={name}
                        onChange={e => onNameChange(e.target.value)}
                        autoFocus={isEditing}
                        aria-invalid={!!nameError}
                        aria-describedby={nameError ? 'name-error' : undefined}
                    />
                </div>
                {nameError && (
                    <div id="name-error" className="text-xs mt-1 font-bold animate-fade-in" style={{ color: 'var(--color-danger)' }} aria-live="polite">
                        ✗
                        {' '}
                        {nameError}
                    </div>
                )}
            </div>
        </div>
    );
}
