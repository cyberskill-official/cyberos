import type { CyberOSContext } from "@cyberos/subgraph-kit";

export const Query = {
  crmHealth: (_root: unknown, _args: unknown, _ctx: CyberOSContext) => ({
    ok: true,
    module: "CRM",
    version: "0.1.0",
  }),
};
