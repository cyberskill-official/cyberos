/**
 * @cyberos/inv — Invoicing subgraph entry point.
 *
 * Phase: P2 · Port: 4016 · GraphQL namespace: inv
 *
 * Owns FRs: see docs/feature-requests/P2/INV/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_INV ?? 4016);

await startSubgraph({
  module: "INV",
  port: PORT,
  typeDefs,
  resolvers,
});
