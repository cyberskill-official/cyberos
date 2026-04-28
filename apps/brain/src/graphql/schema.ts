/**
 * Federated GraphQL schema for @cyberos/brain.
 *
 * Each module declares its types under the namespace `brain` to avoid
 * cross-module name collisions. Federation directives (@key, @shareable,
 * @inaccessible, @tag) are how modules cross-reference each other.
 */

import gql from "graphql-tag";

export const typeDefs = gql`
  extend schema
    @link(
      url: "https://specs.apollo.dev/federation/v2.5"
      import: ["@key", "@shareable", "@external", "@tag", "@inaccessible"]
    )

  """
  Health probe — every subgraph exposes this for the GraphOS Router liveness check.
  """
  type BrainHealthStatus {
    ok: Boolean!
    module: String!
    version: String!
  }

  type Query {
    """Probe the brain subgraph. Always returns ok: true."""
    brainHealth: BrainHealthStatus!
  }
`;
