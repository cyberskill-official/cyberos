//! Per-language label injectors (FR-OBS-002 §1 #5).
//!
//! Each injector parses a query for the target language, adds a `tenant_id` matcher to every stream
//! selector, and reserialises - never string concatenation (DEC-146), so escape attempts in user
//! input cannot break out of the injected filter.

pub mod logql;
pub mod promql; // PromQL via promql-parser@0.4
pub mod traceql; // hand-rolled TraceQL subset
