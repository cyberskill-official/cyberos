"""Stage 2 — structural verification depth.

  verifier    single-call rubric LLM-as-judge ("never validate your own code in
              the same context window") + robust parsing, pluggable backend
  scorermix   blend ~60% deterministic / ~30% judge / ~10% human (never judge-only)
  pbt         property-based + held-out composition test patterns (SpecBench)
  mutation    mutation-testing runner (catches tautological/weak tests)

Design rule from the survey: a SINGLE judge call (0-1 + pass/fail vs a rubric) was
most consistent; more judges did not help. And LLM-judge is never used alone — it
stacks scorer-side stochasticity on top of agent stochasticity.
"""
