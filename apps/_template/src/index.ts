/**
 * {{PACKAGE}} — {{NAME}} subgraph entry point.
 *
 * Phase: {{PHASE}} · Port: {{PORT}} · GraphQL namespace: {{NAMESPACE}}
 *
 * Owns FRs: see docs/feature-requests/{{PHASE}}/{{MODULE}}/.
 *
 * Stays minimal on purpose: bootstrap goes through @cyberos/subgraph-kit,
 * which enforces the canonical Apollo + Express + tenant-context shape.
 */

import { startSubgraph } from "@cyberos/subgraph-kit";
import { typeDefs } from "./graphql/schema.ts";
import { resolvers } from "./graphql/resolvers/index.ts";

const PORT = Number(process.env.PORT_{{MODULE}} ?? {{PORT}});

await startSubgraph({
  module: "{{MODULE}}",
  port: PORT,
  typeDefs,
  resolvers,
});
