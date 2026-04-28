/**
 * @cyberos/esop — Phantom Stock subgraph entry point.
 *
 * Phase: P2 · Port: 4017 · GraphQL namespace: esop
 *
 * Owns FRs: see docs/feature-requests/P2/ESOP/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_ESOP ?? 4017);

await startSubgraph({
  module: "ESOP",
  port: PORT,
  typeDefs,
  resolvers,
});
