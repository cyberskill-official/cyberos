/**
 * @cyberos/okr — Objectives & Key Results subgraph entry point.
 *
 * Phase: P3 · Port: 4019 · GraphQL namespace: okr
 *
 * Owns FRs: see docs/feature-requests/P3/OKR/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_OKR ?? 4019);

await startSubgraph({
  module: "OKR",
  port: PORT,
  typeDefs,
  resolvers,
});
