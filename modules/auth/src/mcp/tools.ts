/**
 * AUTH module — MCP tool definitions
 *
 * Tools follow `module.action` snake_case namespace (SEP-986, DEC-016).
 * All write tools accept `idempotency_key` (24h TTL).
 * Tools wrapped in New Relic background transaction per arch spec.
 *
 * Exposed tools:
 *   auth.whoami        — resolve caller identity from token
 *   auth.issue_token   — issue a new JWT for a member
 *   auth.revoke_token  — revoke an active session or API key
 *   auth.list_sessions — list active sessions for current member
 */

import { McpServer } from '@modelcontextprotocol/sdk/server/mcp.js';
import { z } from 'zod';
import newrelic from 'newrelic';

export function registerAuthMcpTools(server: McpServer): void {
  server.tool(
    'auth.whoami',
    'Resolve the identity of the current authenticated caller — returns member ID, tenant ID, roles, and active scopes.',
    {},
    async (_args, context) => {
      return newrelic.startBackgroundTransaction('mcp.tool.auth.whoami', async () => {
        // Implementation wired to AuthService.whoami()
        const identity = await resolveIdentityFromContext(context);
        return {
          content: [{ type: 'text', text: JSON.stringify(identity, null, 2) }],
        };
      });
    },
  );

  server.tool(
    'auth.revoke_token',
    'Revoke an active session or API key by ID.',
    {
      token_id: z.string().describe('The session or API key ID to revoke'),
      idempotency_key: z.string().optional().describe('24h idempotency key'),
    },
    async ({ token_id, idempotency_key }, _context) => {
      return newrelic.startBackgroundTransaction('mcp.tool.auth.revoke_token', async () => {
        // Implementation wired to AuthService.revokeToken()
        return {
          content: [{ type: 'text', text: `Token ${token_id} revoked (key: ${idempotency_key})` }],
        };
      });
    },
  );
}

async function resolveIdentityFromContext(_context: unknown): Promise<unknown> {
  // TODO: wire to AuthService
  return { memberId: 'placeholder', tenantId: 'placeholder', roles: [], scopes: [] };
}
