/**
 * @cyberos/rew — Total Rewards subgraph entry point.
 *
 * Phase: P1 · Port: 4014 · GraphQL namespace: rew
 *
 * Owns FRs: see docs/feature-requests/P1/REW/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_REW ?? 4014);

await startSubgraph({
  module: "REW",
  port: PORT,
  typeDefs,
  resolvers,
});
