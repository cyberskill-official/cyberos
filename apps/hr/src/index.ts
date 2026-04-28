/**
 * @cyberos/hr — Human Resources subgraph entry point.
 *
 * Phase: P1 · Port: 4012 · GraphQL namespace: hr
 *
 * Owns FRs: see docs/feature-requests/P1/HR/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_HR ?? 4012);

await startSubgraph({
  module: "HR",
  port: PORT,
  typeDefs,
  resolvers,
});
