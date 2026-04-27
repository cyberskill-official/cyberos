/**
 * Tenant middleware — Layer 2 of the 3-layer isolation model.
 *
 * Reads `x-tenant-id` header forwarded by the Apollo Router (which validates
 * the JWT and propagates claims). Sets `SET LOCAL app.tenant_id` so PostgreSQL
 * RLS policies (Layer 3) are automatically enforced per-request.
 *
 * NEVER skip this middleware. No route bypasses tenant scoping.
 * See SRS §11.2 and architecture principle #3.
 */

import type { Request, Response, NextFunction } from 'express';
import { db } from '../db.js';

export async function tenantMiddleware(
  req: Request,
  res: Response,
  next: NextFunction,
): Promise<void> {
  const tenantId = req.headers['x-tenant-id'];

  if (!tenantId || typeof tenantId !== 'string') {
    res.status(401).json({ error: 'Missing x-tenant-id header' });
    return;
  }

  // Set RLS context for every DB query in this request lifecycle
  await db.$executeRawUnsafe(`SET LOCAL app.tenant_id = '${tenantId}'`);

  req.tenantId = tenantId;
  next();
}

// Augment Express Request type
declare global {
  namespace Express {
    interface Request {
      tenantId: string;
    }
  }
}
