/**
 * @cyberos/shared — cross-cutting types and utilities for every CyberOS module.
 *
 * Public surface only. Anything that doesn't have multiple consumers stays
 * inside the module that uses it. This package must remain dependency-light
 * (zero runtime deps at v0.1) so subgraphs cold-start fast.
 */

export * from "./errors/index.ts";
export * from "./tenancy/index.ts";
export * from "./types/index.ts";

export const SHARED_VERSION = "0.1.0" as const;
