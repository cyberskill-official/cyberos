import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  {{NAMESPACE}}Health: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "{{MODULE}}",
    version: "0.1.0",
  }),
};
