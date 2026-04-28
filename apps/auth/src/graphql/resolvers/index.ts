/**
 * Resolver tree for @cyberos/auth.
 *
 * Convention: one resolver file per top-level type, re-exported here.
 * Keep this file tiny; logic lives in the per-type files and `services/`.
 */

import { Query } from "./query.ts";

export const resolvers = {
  Query,
};
