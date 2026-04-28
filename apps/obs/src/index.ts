/**
 * @cyberos/obs — Observability subgraph entry point.
 *
 * Phase: P0 · Port: 4004 · GraphQL namespace: obs
 *
 * Owns FRs: see docs/feature-requests/P0/OBS/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_OBS ?? 4004);

await startSubgraph({
  module: "OBS",
  port: PORT,
  typeDefs,
  resolvers,
});
