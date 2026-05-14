# CUO routing — heuristics and design

## Phase 1 — rule-based router

### Why rules first

A Phase-1 LLM router would have a hard dependency on a model endpoint, an inference budget, and non-determinism. Before adding that surface, we want to prove the *shape* of the CUO works: it discovers skills from disk, scores them against a natural-language query, dispatches to the skill module, records the decision in BRAIN. A rule-based router gets us there in two hundred lines and is deterministic enough to be unit-tested against fixtures.

### The scoring formula

For each candidate skill in the catalog, the router computes:

```
score  = 5.0 if skill_name_normalised is substring of query_normalised else 0
       + 3.0 * (count of keyword-bank hits)
       + 2.0 if query has Vietnamese diacritics and skill.region == "VN" else 0
```

Where:

* `*_normalised` is the input passed through Unicode NFKD decomposition with combining marks stripped, lowercased, with non-alphanumeric runs collapsed to single spaces.
* The keyword bank is a per-skill list of short tokens (English and Vietnamese-without-diacritics).

A query scores against many skills; the top scorer wins if and only if its score is at least the confidence threshold (`3.0`). Below threshold, the router returns `None` and CUO surfaces "no match — please clarify" with the top three alternatives.

### Why these weights

* `5.0` for verbatim name match — the strongest signal short of an exact-string match. A user typing `vn-mst-validate` knows what they want.
* `3.0` per keyword — multiple distinct keywords compounding is itself a confidence signal. Two keywords (`6.0`) saturates over the threshold even without a name match.
* `2.0` for region match — a tiebreaker, not a primary signal. Vietnamese diacritics in a query are weak evidence that a VN-region skill is preferred over an unmarked one.
* `10.0` saturation — confidence reported as `score / 10.0`, clamped to `1.0`. Two keyword hits → confidence `0.6`; a name match plus two keywords → `1.0`.

### The keyword bank

The bank is intentionally small and skill-specific. It lives in `cuo/core/router.py::_KEYWORD_BANK`. Adding a new skill means adding 4–8 keywords there; in Phase 2 the bank becomes redundant (the LLM reads the SKILL.md description directly).

| Skill | Keywords (representative sample) |
|---|---|
| `vn-mst-validate` | mst, tax code, ma so thue, validate tax, kiem tra mst |
| `vn-vat-invoice` | invoice, hoa don, vat, gtgt, e-invoice, xuat hoa don |
| `vn-bank-transfer` | transfer, qr, chuyen khoan, vietqr, napas, ma qr |
| `vneid-integration` | cccd, citizen id, can cuoc, vneid, id card, danh tinh |
| `vn-tax-filing` | filing, return, to khai, ke khai thue, monthly vat, quarterly vat |
| `vn-legal-compliance` | compliance, law, decree, nghi dinh, thong tu, pdpd, cybersecurity |

### Argument extraction

Per-skill regex extractors live in the same module. They are pure, deterministic, and intentionally conservative — a Phase 2 LLM extractor is expected to supersede them.

* `vn-mst-validate` — pulls the first `\d{10}(-\d{3})?` substring.
* `vneid-integration` — pulls the first `\d{12}` substring.
* `vn-bank-transfer` — pulls the first Napas bank short-code, the first 6–19-digit run as account, and a fuzzy amount tail.
* `vn-vat-invoice` — flags presence of an amount; structured extraction is deliberately left to Phase 2.

## Phase 2 — LLM-driven router (design)

### Shape

```
prompt = render(query, catalog_summaries)
response = model.chat(prompt, response_format="json")
decision = parse(response)
```

### Prompt elements

1. The user query verbatim.
2. The catalog: every skill's name + description + when-to-use snippet (trimmed to fit a token budget; large catalogs are summarised by category first).
3. The output schema: `{skill_name, arguments, rationale, confidence}`.
4. A safety rail: if no skill applies, return `{skill_name: null, rationale: "..."}`.

### Why we still want the rule-based router after Phase 2

Three reasons:

1. **Latency.** Common, unambiguous queries (a single MST in the message body) deserve a sub-millisecond decision, not a 200ms model call.
2. **Determinism.** Audit replay benefits from a reproducible path. The rule-based router is the canonical fallback for "what would CUO have done last week?".
3. **Cost.** The skill catalog gets large quickly. Routing 90% of obvious cases through rules and only escalating the ambiguous 10% to a model is the right cost trade-off.

So Phase 2 layers on top: rules first; only escalate when no rule scores above `0.5` confidence *and* one or more skills score above `0.1`.

## Phase 3 — multi-skill chains (sketch)

The MVP chain is `vn-mst-validate` → `vn-vat-invoice`: validate the buyer + seller MSTs before generating the invoice. CUO recognises chains by reading the chosen skill's frontmatter `depends_on:` field and walking dependencies in topological order.

```
decision_root = route(query, catalog)
decisions = [decision_root]
for dep_name in catalog[root].depends_on:
    decisions.append(route_for_dep(dep_name, decision_root.arguments))
execute_in_order(decisions)
```

The audit-chain entry for the chain is a single composite row; each leaf invocation is its own sub-row so per-step failure is recoverable.

## Phase 4 — persona switching

Per the PRD §6.1, CUO is itself a composite of sub-personas (CPO, CTO, CFO, CMO, COO). Phase 4 splits the keyword bank by persona and dispatches to the appropriate persona-router first; each persona then runs its own intra-persona routing against the relevant skill subset (the `skill/skills/cuo/<persona>/` directories already segregate skills this way).
