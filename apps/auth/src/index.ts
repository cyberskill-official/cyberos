/**
 * @cyberos/auth — Authentication & Tenancy subgraph entry point.
 *
 * Phase: P0 · Port: 4001 · GraphQL namespace: auth
 *
 * Owns FRs: see docs/feature-requests/P0/AUTH/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_AUTH ?? 4001);

await startSubgraph({
  module: "AUTH",
  port: PORT,
  typeDefs,
  resolvers,
});
