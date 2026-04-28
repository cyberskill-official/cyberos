/**
 * @cyberos/genie — Company Mascot AI Assistant subgraph entry point.
 *
 * Phase: P0 · Port: 4007 · GraphQL namespace: genie
 *
 * Owns FRs: see docs/feature-requests/P0/GENIE/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_GENIE ?? 4007);

await startSubgraph({
  module: "GENIE",
  port: PORT,
  typeDefs,
  resolvers,
});
