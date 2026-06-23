import { describe, expect, it } from 'vitest';

// Test the exported types exist (compile-time check)
import type {
    I_AliasFormData,
    I_ExportData,
    I_GitAlias,
    I_IpcResult,
    I_ValidationResult,
} from '#/types/index';

describe('type Definitions', () => {
    it('should define I_GitAlias with correct properties', () => {
        const alias: I_GitAlias = {
            name: 'co',
            command: 'checkout',
            scope: 'global',
        };
        expect(alias.name).toBe('co');
        expect(alias.command).toBe('checkout');
        expect(alias.scope).toBe('global');
    });

    it('should define I_AliasFormData with correct properties', () => {
        const form: I_AliasFormData = {
            name: 'st',
            command: 'status',
            scope: 'local',
        };
        expect(form.scope).toBe('local');
    });

    it('should define I_IpcResult with success and data', () => {
        const result: I_IpcResult<string> = {
            success: true,
            data: 'test',
        };
        expect(result.success).toBe(true);
        expect(result.data).toBe('test');
    });

    it('should define I_IpcResult with error', () => {
        const result: I_IpcResult = {
            success: false,
            error: 'Something went wrong',
        };
        expect(result.success).toBe(false);
        expect(result.error).toBe('Something went wrong');
    });

    it('should define I_ValidationResult', () => {
        const result: I_ValidationResult = {
            valid: true,
            warnings: ['force push detected'],
            errors: [],
        };
        expect(result.valid).toBe(true);
        expect(result.warnings).toHaveLength(1);
    });

    it('should define I_ExportData with version and aliases', () => {
        const data: I_ExportData = {
            version: '1.0.0',
            exportedAt: '2024-01-01T00:00:00Z',
            aliases: [{ name: 'co', command: 'checkout', scope: 'global' }],
        };
        expect(data.version).toBe('1.0.0');
        expect(data.aliases).toHaveLength(1);
    });
});
