import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["src/**/*.test.ts", "test/**/*.test.ts"],
    coverage: {
      provider: "v8",
      reporter: ["text", "lcov"],
      thresholds: {
        // Per-module floor; module-specific PRs may raise.
        statements: 70,
        branches: 60,
        functions: 70,
        lines: 70,
      },
    },
  },
});
