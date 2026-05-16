---
id: FR-CHAT-004
title: "PGroonga + custom Vietnamese bigram tokeniser — VN message search with ≥ 80% recall CI gate and dual-path (VN-bigram / EN-PGroonga) hybrid routing"
module: CHAT
priority: MUST
status: accepted
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-05-16
shipped: null
brain_chain_hash: null
related_frs: [FR-CHAT-003, FR-CHAT-005, FR-BRAIN-108]
depends_on: [FR-CHAT-003]
blocks: []

source_pages:
  - website/docs/modules/chat.html#search
  - website/docs/legal/vn-text-indexing.html
source_decisions:
  - DEC-450 (PGroonga + custom VN bigram tokeniser; PostgreSQL-native; no Elasticsearch)
  - DEC-451 (recall ≥ 80% on labelled VN corpus; CI gate fails build below threshold)
  - DEC-452 (English uses default PGroonga tokeniser; VN detected via UTF-8 codepoint heuristic with > 5% threshold)
  - DEC-453 (bigram chosen over trigram: 87% vs 91% recall but 3× index size — bigram wins on space/recall tradeoff)

language: sql + go + python (test fixture only)
service: cyberos/services/chat/
new_files:
  - services/chat/sql/init-pgroonga.sql
  - services/chat/sql/cyberos-vn-tokenizer.sql
  - services/chat/plugins/cyberos-vn-search/main.go
  - services/chat/plugins/cyberos-vn-search/plugin.json
  - services/chat/plugins/cyberos-vn-search/Makefile
  - services/chat/plugins/cyberos-vn-search/tokenizer_test.go
  - services/chat/plugins/cyberos-vn-search/search_test.go
  - services/chat/test-fixtures/vn-message-corpus.jsonl
  - services/chat/test-fixtures/en-message-corpus.jsonl
  - services/chat/scripts/measure-recall.py
  - .github/workflows/chat-search-recall-gate.yml
modified_files:
  - infra/terraform/modules/tenant_chat/rds.tf       # enable pgroonga extension
  - services/chat/patches/020-search-route.patch     # route /api/v4/posts/search to our plugin
allowed_tools:
  - file_read: services/chat/**
  - file_write: services/chat/plugins/cyberos-vn-search/**, services/chat/sql/**, services/chat/test-fixtures/**, services/chat/scripts/**, .github/workflows/**
  - bash: cd services/chat/plugins/cyberos-vn-search && go test
  - bash: python3 services/chat/scripts/measure-recall.py
disallowed_tools:
  - integrate Elasticsearch or any external search service (per DEC-450 — Postgres-native)
  - use the default PGroonga tokeniser for VN-detected messages (per DEC-452 bigram required)
  - skip the recall CI gate on PRs touching the tokeniser (per DEC-451)

effort_hours: 12
sub_tasks:
  - "0.5h: rds.tf — enable pgroonga extension on RDS parameter group"
  - "0.5h: init-pgroonga.sql — CREATE EXTENSION pgroonga; create indexes on posts.message"
  - "1.5h: cyberos-vn-tokenizer.sql — detect_vn() + bigram_split() PL/pgSQL functions"
  - "1.0h: plugin.json + Makefile + plugin skeleton"
  - "1.5h: main.go — Mattermost /search interception with hybrid routing"
  - "0.5h: VN detection heuristic (≥ 5% tonal-mark characters → VN path)"
  - "1.5h: vn-message-corpus.jsonl — 500 labelled VN queries with expected hits + tonal-mark coverage"
  - "0.5h: en-message-corpus.jsonl — 100 labelled EN queries (smoke test for default path)"
  - "1.5h: measure-recall.py — corpus runner + recall/precision/FP-rate computation"
  - "1.0h: chat-search-recall-gate.yml — CI workflow on PR; blocks merge below 0.80"
  - "1.0h: tokenizer_test.go — 30+ bigram correctness fixtures + detect_vn boundary cases"
  - "0.5h: search_test.go — RLS team isolation + deleted_at filter assertions"
  - "0.5h: latency budget test — 100K-msg corpus p95 ≤ 200ms"
risk_if_skipped: "Default PostgreSQL tokenisation splits Vietnamese on whitespace; tonal vowels become unsearchable. Without bigram, 'cà phê' indexes as ['cà', 'phê'] and a search for 'phê đen' misses 'cà phê đen'. Without the recall CI gate, regressions ship undetected — operators only notice when users complain 'search is broken' weeks later. Without the measure script, 'good enough' is subjective and varies by reviewer. Without VN detection, English messages get tokenised twice (wasted index space + write latency). Without hybrid routing, the plugin ships one-size-fits-all degradation."
---

## §1 — Description (BCP-14 normative)

The CHAT search layer **MUST** provide Vietnamese-aware full-text search via PGroonga + custom bigram tokeniser, with a recall-gated CI workflow. The contract:

1. **MUST** enable the PGroonga extension on the tenant_chat RDS instance via `CREATE EXTENSION IF NOT EXISTS pgroonga` in the init migration.
2. **MUST** define two PL/pgSQL functions in `services/chat/sql/cyberos-vn-tokenizer.sql`:
    - `detect_vn(input TEXT) -> BOOLEAN` — returns true iff > 5% of input characters fall in the Vietnamese tonal-mark Unicode range. Pure function; `IMMUTABLE`.
    - `bigram_split(input TEXT) -> TEXT[]` — returns overlapping 2-character shingles of the lowercased input. Pure function; `IMMUTABLE`.
3. **MUST** create two PostgreSQL indexes on the `posts` table:
    - `posts_message_pgroonga_idx` — `USING pgroonga (message)` with the default `TokenBigramSplitSymbolAlphaDigit` tokeniser; serves English + mixed-language queries.
    - `posts_vn_bigrams_idx` — `USING gin (bigram_split(message)) WHERE detect_vn(message)` — partial index for VN-detected messages.
4. **MUST** intercept Mattermost's `POST /api/v4/posts/search` endpoint via the `cyberos-vn-search` plugin (FR-CHAT-001 patch `020-search-route.patch` redirects to plugin handler).
5. **MUST** route incoming queries through hybrid detection:
    - VN detected (≥ 5% tonal-mark codepoints) → use bigram-array overlap (`bigram_split(message) && <query_bigrams>`).
    - Otherwise → use PGroonga `&@~` full-text operator.
6. **MUST** respect Mattermost ACL filters in every query:
    - `team_id` MUST match caller's team membership (FR-CHAT-002 propagates from JWT).
    - `deleted_at IS NULL` MUST filter soft-deleted posts.
    - Results MUST honour FR-AUTH-003 RLS (channel privacy + tenant scope).
7. **MUST** support pagination via Mattermost-standard `page` + `per_page` parameters with default `per_page=20`, max `per_page=100`.
8. **MUST** order results by `create_at DESC` (newest first); secondary sort by `id` for stability.
9. **MUST** complete query in ≤ 200ms p95 for a 100K-message corpus on the standard tier (FR-CHAT-003 `cache.t4g.small` + RDS `db.t4g.small`).
10. **MUST** maintain a labelled VN corpus at `services/chat/test-fixtures/vn-message-corpus.jsonl` with ≥ 500 labelled cases. Each line is `{"query": "<str>", "expected_post_ids": ["p1", ...]}`. Fixture covers single-word, multi-word, compound, accented-vs-unaccented forms, partial matches, and known-edge-case patterns (Vietnamese loan-words, diacritic-stripping inputs).
11. **MUST** pass a CI gate (`chat-search-recall-gate.yml`) running `measure-recall.py` against the VN corpus:
    - Recall ≥ 0.80 (true-positive / (true-positive + false-negative)).
    - False-positive rate ≤ 0.05 (false-positive / (false-positive + true-negative)).
    - Both thresholds enforced; either failure = CI red.
12. **MUST** emit BRAIN audit row `chat.search_query` per non-trivial search (queries ≥ 3 chars) with payload `{user_id, team_id, query_hash, query_lang (vn|en), result_count, latency_ms, trace_id}`. Query hashed (not raw) for PII safety.
13. **MUST** emit OTel metrics:
    - `chat_search_queries_total{lang, outcome}` (counter; outcome ∈ ok | empty | error | rate_limited).
    - `chat_search_latency_seconds{lang}` (histogram, FR-OBS-003 standardised buckets).
    - `chat_search_recall` (gauge, set by CI gate run; alerts FR-OBS-007 sev-2 below 0.80).
14. **MUST** enforce per-tenant rate limits (governor crate or PostgreSQL `pg_stat_statements`-aware throttle): 100 queries/min sustained, 300 burst. Over-limit returns `429 TOO_MANY_REQUESTS`.
15. **SHOULD** support `cyberos chat search debug --query "<str>" --tenant <id>` CLI for operator diagnostics; prints detected language + chosen index + result IDs + per-stage latency.

---

## §2 — Why this design (rationale for humans)

**Why PGroonga over Elasticsearch (DEC-450)?** Three reasons. (a) Zero infra cost — PGroonga lives inside Postgres; no second service to provision, monitor, back up. (b) Postgres-native means FR-AUTH-003 RLS works without re-implementation; Elasticsearch needs a parallel ACL layer. (c) PGroonga's bigram tokeniser is the proven choice for CJK + Vietnamese in the open-source Postgres ecosystem. Slight latency cost vs ES is acceptable up to ~1M messages per tenant; beyond that, slice-4 may revisit.

**Why bigram over trigram (DEC-453)?** Empirical benchmarking on the 500-line corpus: bigram = 87% recall, trigram = 91% recall, but trigram index is 3× larger and write latency 2× higher. The recall delta (4 percentage points) is below the user-perceptible threshold; the storage/latency cost is significant. Bigram is the calibrated choice.

**Why detect VN at index time AND query time (§1 #3 + #5)?** The partial index `WHERE detect_vn(message)` keeps non-VN messages out of the bigram index (saves ~70% space in an English-mixed tenant). The query-time check routes the query to the matching index — VN query against bigram index, EN query against PGroonga's default tokeniser. Without dual-path routing, EN queries would scan the bigram index and produce poor results.

**Why 5% threshold for VN detection (§1 #5)?** Empirical: VN tonal-mark characters appear ~15-20% of an average VN sentence; ~0% of pure-English. A 5% threshold catches mixed-language messages (Vietnamese text with English brand names: "cập nhật Slack") while excluding pure-English with an accidental "café" or "naïve."

**Why recall ≥ 80% gate (DEC-451)?** Below 80% users notice "search doesn't work" → support tickets + churn risk. Above 80% users tolerate misses (results are sorted by relevance + recency anyway). 80% is the calibrated point from VN-market beta feedback. The 5% false-positive cap prevents over-eager bigram matches (e.g. "cà" matching anything containing "ca").

**Why hash the query in audit row (§1 #12)?** Raw query strings can contain PII or sensitive context (operators searching for customer names, contract terms). Hash is sufficient for "did this user search?" (audit trail) but does not expose what they searched for. Operators investigating specific searches use the per-tenant debug CLI.

**Why RLS at query time (§1 #6)?** Mattermost's ACL is its own layer; FR-AUTH-003 RLS is ours. The query MUST satisfy BOTH — Mattermost team membership AND tenant_id RLS. Skipping either is a leak: Mattermost-only allows cross-tenant scan; RLS-only allows cross-team scan within a tenant.

**Why per-tenant rate limit (§1 #14)?** Search is naturally bursty (user opens search modal → types 5 chars → backend hit 5×). 100/min sustained with 300 burst absorbs human typing without throttling. Beyond that suggests programmatic scraping → 429.

**Why latency budget 200ms p95 (§1 #9)?** Sub-200ms feels instant; 200-500ms feels sluggish; 500ms+ feels broken. PGroonga's bigram index is empirically ~50ms at 100K messages; 200ms gives 4× headroom for query parsing + RLS + serialization.

**Why CI gate not just unit tests (§1 #11)?** Unit tests assert the tokeniser produces the right bigrams. The CI gate asserts the end-to-end search USING those bigrams achieves the recall threshold. They catch different failures: a tokeniser bug breaks unit tests; an index-strategy regression (e.g. someone disables the partial index) breaks the recall gate but passes unit tests.

---

## §3 — API contract

### SQL migrations

```sql
-- services/chat/sql/init-pgroonga.sql

CREATE EXTENSION IF NOT EXISTS pgroonga;

-- English + mixed-language index (PGroonga default tokeniser)
CREATE INDEX IF NOT EXISTS posts_message_pgroonga_idx
    ON posts USING pgroonga (message)
    WITH (tokenizer = 'TokenBigramSplitSymbolAlphaDigit');

-- Vietnamese-only partial index (custom bigram_split)
CREATE INDEX IF NOT EXISTS posts_vn_bigrams_idx
    ON posts USING gin (bigram_split(message))
    WHERE detect_vn(message);
```

```sql
-- services/chat/sql/cyberos-vn-tokenizer.sql

CREATE OR REPLACE FUNCTION detect_vn(input TEXT) RETURNS BOOLEAN AS $$
DECLARE
    vn_chars INT := 0; total INT := 0; c CHAR;
BEGIN
    FOR i IN 1..length(input) LOOP
        c := substr(input, i, 1);
        IF c ~ '[áàảãạăắằẳẵặâấầẩẫậéèẻẽẹêếềểễệíìỉĩịóòỏõọôốồổỗộơớờởỡợúùủũụưứừửữựýỳỷỹỵđÁÀẢÃẠĂẮẰẲẴẶÂẤẦẨẪẬÉÈẺẼẸÊẾỀỂỄỆÍÌỈĨỊÓÒỎÕỌÔỐỒỔỖỘƠỚỜỞỠỢÚÙỦŨỤƯỨỪỬỮỰÝỲỶỸỴĐ]'
        THEN vn_chars := vn_chars + 1;
        END IF;
        total := total + 1;
    END LOOP;
    RETURN (total > 0) AND (vn_chars::float / total > 0.05);
END;
$$ LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE;

-- Bigram split: "cà phê" → ['cà', 'à ', ' p', 'ph', 'hê']
CREATE OR REPLACE FUNCTION bigram_split(input TEXT) RETURNS TEXT[] AS $$
DECLARE
    out TEXT[] := '{}'; i INT;
BEGIN
    FOR i IN 1..(length(input) - 1) LOOP
        out := out || array[lower(substr(input, i, 2))];
    END LOOP;
    RETURN out;
END;
$$ LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE;
```

### Mattermost plugin

```go
// services/chat/plugins/cyberos-vn-search/main.go
package main

import (
    "context"
    "database/sql"
    "encoding/json"
    "net/http"
    "strings"
    "time"

    "github.com/mattermost/mattermost/server/public/plugin"
)

type SearchPlugin struct {
    plugin.MattermostPlugin
    db        *sql.DB
    rateLimit *RateLimiter   // governor-style token bucket per tenant
}

type SearchRequest struct {
    Query    string `json:"query"`
    TeamID   string `json:"team_id"`
    Page     int    `json:"page"`
    PerPage  int    `json:"per_page"`
}

type SearchResult struct {
    Posts      []Post `json:"posts"`
    TotalCount int    `json:"total_count"`
    LatencyMs  int64  `json:"latency_ms"`
    QueryLang  string `json:"query_lang"`
}

type Post struct {
    ID        string `json:"id"`
    Message   string `json:"message"`
    UserID    string `json:"user_id"`
    ChannelID string `json:"channel_id"`
    CreateAt  int64  `json:"create_at"`
}

func (p *SearchPlugin) ServeHTTP(c *plugin.Context, w http.ResponseWriter, r *http.Request) {
    if r.URL.Path != "/search" || r.Method != http.MethodPost {
        http.NotFound(w, r)
        return
    }
    var req SearchRequest
    if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
        http.Error(w, "bad request", 400)
        return
    }
    if req.PerPage == 0 { req.PerPage = 20 }
    if req.PerPage > 100 { req.PerPage = 100 }

    subjectID := c.AppContext.Session().UserId
    tenantID  := c.AppContext.Session().Props["tenant_id"]

    if !p.rateLimit.Check(tenantID) {
        http.Error(w, `{"error":"rate_limited"}`, 429)
        return
    }

    start := time.Now()
    isVn := isVietnameseQuery(req.Query)
    var posts []Post
    var err error
    if isVn {
        posts, err = p.searchBigram(r.Context(), req, tenantID)
    } else {
        posts, err = p.searchPGroonga(r.Context(), req, tenantID)
    }
    if err != nil {
        http.Error(w, err.Error(), 500)
        return
    }

    latencyMs := time.Since(start).Milliseconds()
    p.emitBrainAudit(subjectID, tenantID, req, isVn, len(posts), latencyMs)
    p.recordMetric(isVn, "ok", latencyMs)

    json.NewEncoder(w).Encode(SearchResult{
        Posts:      posts,
        TotalCount: len(posts),
        LatencyMs:  latencyMs,
        QueryLang:  ifThenElse(isVn, "vn", "en"),
    })
}

func (p *SearchPlugin) searchBigram(ctx context.Context, req SearchRequest, tenantID string) ([]Post, error) {
    bigrams := bigramSplit(req.Query)
    rows, err := p.db.QueryContext(ctx, `
        SELECT id, message, user_id, channel_id, create_at
        FROM posts
        WHERE team_id = $1
          AND deleted_at IS NULL
          AND tenant_id = $2::uuid
          AND bigram_split(message) && $3
        ORDER BY create_at DESC, id DESC
        LIMIT $4 OFFSET $5
    `, req.TeamID, tenantID, bigrams, req.PerPage, req.Page*req.PerPage)
    if err != nil { return nil, err }
    defer rows.Close()
    var out []Post
    for rows.Next() {
        var p Post
        if err := rows.Scan(&p.ID, &p.Message, &p.UserID, &p.ChannelID, &p.CreateAt); err != nil { return nil, err }
        out = append(out, p)
    }
    return out, nil
}

func (p *SearchPlugin) searchPGroonga(ctx context.Context, req SearchRequest, tenantID string) ([]Post, error) {
    rows, err := p.db.QueryContext(ctx, `
        SELECT id, message, user_id, channel_id, create_at
        FROM posts
        WHERE team_id = $1
          AND deleted_at IS NULL
          AND tenant_id = $2::uuid
          AND message &@~ $3
        ORDER BY create_at DESC, id DESC
        LIMIT $4 OFFSET $5
    `, req.TeamID, tenantID, req.Query, req.PerPage, req.Page*req.PerPage)
    // ... same scan pattern as searchBigram
    return nil, err
}

// isVietnameseQuery: count chars in VN tonal range, return true if > 5%.
func isVietnameseQuery(q string) bool {
    vnChars := 0
    total := 0
    for _, r := range q {
        total++
        if isVietnameseRune(r) { vnChars++ }
    }
    return total > 0 && float64(vnChars)/float64(total) > 0.05
}

func isVietnameseRune(r rune) bool {
    // Hand-curated Vietnamese tonal-mark code points (same as detect_vn SQL)
    return strings.ContainsRune("áàảãạăắằẳẵặâấầẩẫậéèẻẽẹêếềểễệíìỉĩịóòỏõọôốồổỗộơớờởỡợúùủũụưứừửữựýỳỷỹỵđÁÀẢÃẠĂẮẰẲẴẶÂẤẦẨẪẬÉÈẺẼẸÊẾỀỂỄỆÍÌỈĨỊÓÒỎÕỌÔỐỒỔỖỘƠỚỜỞỠỢÚÙỦŨỤƯỨỪỬỮỰÝỲỶỸỴĐ", r)
}

// bigramSplit: same algorithm as the SQL function, in Go for query-side use.
func bigramSplit(s string) []string {
    runes := []rune(strings.ToLower(s))
    if len(runes) < 2 { return nil }
    bigrams := make([]string, 0, len(runes)-1)
    for i := 0; i < len(runes)-1; i++ {
        bigrams = append(bigrams, string(runes[i:i+2]))
    }
    return bigrams
}
```

### plugin.json

```json
{
  "id": "cyberos.vn-search",
  "name": "CyberOS VN Search",
  "version": "1.0.0",
  "min_server_version": "9.0.0",
  "server": { "executables": { "linux-amd64": "server/plugin-linux-amd64",
                                 "linux-arm64": "server/plugin-linux-arm64",
                                 "darwin-amd64": "server/plugin-darwin-amd64",
                                 "darwin-arm64": "server/plugin-darwin-arm64" }}
}
```

### Recall measurement script

```python
# services/chat/scripts/measure-recall.py
import json, sys, os, requests

CORPUS = os.environ.get('CYBEROS_VN_CORPUS', 'services/chat/test-fixtures/vn-message-corpus.jsonl')
ENDPOINT = os.environ.get('CYBEROS_CHAT_URL', 'http://localhost:8065')
TOKEN    = os.environ['CYBEROS_TEST_TOKEN']
TEAM_ID  = os.environ['CYBEROS_TEST_TEAM_ID']

tp = fn = fp = tn = 0
with open(CORPUS) as f:
    for line in f:
        case = json.loads(line)
        query = case['query']
        expected = set(case['expected_post_ids'])
        resp = requests.post(
            f"{ENDPOINT}/plugins/cyberos.vn-search/search",
            headers={'Authorization': f'Bearer {TOKEN}', 'Content-Type': 'application/json'},
            json={'query': query, 'team_id': TEAM_ID, 'page': 0, 'per_page': 100}
        )
        actual = set(p['id'] for p in resp.json()['posts'])

        case_tp = len(expected & actual)
        case_fn = len(expected - actual)
        case_fp = len(actual - expected)
        case_tn = 1 if (not expected and not actual) else 0

        tp += case_tp; fn += case_fn; fp += case_fp; tn += case_tn

recall = tp / (tp + fn) if (tp + fn) > 0 else 0.0
fp_rate = fp / (fp + tn) if (fp + tn) > 0 else 0.0

print(f"recall   = {recall:.3f}    (gate: >= 0.80)")
print(f"fp_rate  = {fp_rate:.3f}   (gate: <= 0.05)")
print(f"counts   tp={tp} fn={fn} fp={fp} tn={tn}")

# Emit a Prometheus-style gauge that the OBS collector can pick up
with open('/tmp/chat_search_recall.prom', 'w') as f:
    f.write(f'chat_search_recall {recall}\n')
    f.write(f'chat_search_fp_rate {fp_rate}\n')

# CI gate
if recall < 0.80 or fp_rate > 0.05:
    print('::error::recall or fp_rate outside acceptable range')
    sys.exit(1)
sys.exit(0)
```

### CI workflow

```yaml
# .github/workflows/chat-search-recall-gate.yml
name: chat search recall gate
on:
  pull_request:
    paths:
      - 'services/chat/plugins/cyberos-vn-search/**'
      - 'services/chat/sql/cyberos-vn-tokenizer.sql'
      - 'services/chat/test-fixtures/vn-message-corpus.jsonl'
jobs:
  recall:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: groonga/pgroonga:latest-alpine-16
        env: { POSTGRES_PASSWORD: test }
        ports: ['5432:5432']
    steps:
      - uses: actions/checkout@v4
      - run: psql -h localhost -U postgres -f services/chat/sql/init-pgroonga.sql
      - run: psql -h localhost -U postgres -f services/chat/sql/cyberos-vn-tokenizer.sql
      - run: psql -h localhost -U postgres -f services/chat/test-fixtures/seed-corpus.sql
      - run: cd services/chat/plugins/cyberos-vn-search && go build -o /tmp/plugin
      - run: /tmp/plugin --test-server &
      - run: python3 services/chat/scripts/measure-recall.py
        env:
          CYBEROS_CHAT_URL: http://localhost:8065
          CYBEROS_TEST_TOKEN: ${{ secrets.CYBEROS_TEST_TOKEN }}
          CYBEROS_TEST_TEAM_ID: ${{ secrets.CYBEROS_TEST_TEAM_ID }}
```

### Fixture sample

```jsonl
{"query":"cà phê đen","expected_post_ids":["p1","p3","p7"]}
{"query":"họp với khách hàng","expected_post_ids":["p12","p18","p24"]}
{"query":"hoàn thành","expected_post_ids":["p4","p9","p22","p31"]}
{"query":"thanh toán","expected_post_ids":["p15","p27"]}
{"query":"giao hàng","expected_post_ids":["p33","p41"]}
{"query":"khẩn cấp","expected_post_ids":["p52"]}
{"query":"đã duyệt","expected_post_ids":["p61","p68"]}
{"query":"deadline","expected_post_ids":["p70","p71","p72"]}
{"query":"trao đổi","expected_post_ids":["p85","p93","p101"]}
{"query":"xác nhận","expected_post_ids":["p110","p115"]}
```

---

## §4 — Acceptance criteria

1. **PGroonga extension installed** — `SELECT * FROM pg_extension WHERE extname='pgroonga'` returns 1 row on tenant_chat RDS.
2. **detect_vn() positive** — `SELECT detect_vn('cà phê đen')` returns true.
3. **detect_vn() negative** — `SELECT detect_vn('coffee black')` returns false.
4. **detect_vn() boundary** — `SELECT detect_vn('café')` (5% threshold edge) — accent counts; `detect_vn` returns true if accent fraction > 5%; false otherwise. Documented expected behaviour per input length.
5. **bigram_split() correct shape** — `SELECT bigram_split('cà phê')` returns `['cà', 'à ', ' p', 'ph', 'hê']` (5 bigrams for 6-char input).
6. **bigram_split() empty input** — `SELECT bigram_split('')` returns `'{}'` (empty array).
7. **bigram_split() single-char input** — `SELECT bigram_split('x')` returns `'{}'` (no bigrams possible).
8. **VN query routes to bigram path** — query `"cà phê"` → log shows `searchBigram` called; PGroonga path not invoked.
9. **EN query routes to PGroonga path** — query `"coffee"` → log shows `searchPGroonga` called.
10. **Mixed-language query** — query `"book cà phê meeting"` (≥ 5% tonal) → bigram path.
11. **Search respects team_id** — user in team-A searching → returns only team-A posts; team-B posts invisible.
12. **Search respects deleted_at** — soft-deleted post never appears in results.
13. **Search respects RLS** — tenant A's data not visible to tenant B.
14. **Pagination works** — `per_page=10`, `page=0` returns 10; `page=1` returns next 10 ordered by create_at DESC.
15. **per_page cap** — request `per_page=500` → server clamps to 100.
16. **Latency p95 ≤ 200ms** — 100K-msg corpus benchmark; p95 latency under 200ms.
17. **CI gate passes at recall ≥ 0.80** — measure-recall.py on shipped corpus → exit 0; printed recall ≥ 0.80.
18. **CI gate fails at recall < 0.80** — fixture corpus with intentional regression → exit 1; PR blocked.
19. **CI gate fails at fp_rate > 0.05** — fixture with over-eager matches → exit 1.
20. **BRAIN audit `chat.search_query`** — non-trivial query → 1 row emitted; payload contains `query_hash` (not raw query).
21. **OTel `chat_search_queries_total{lang=vn}` increments** — VN query → counter +1.
22. **OTel `chat_search_latency_seconds{lang=vn}` records** — histogram populated.
23. **Rate limit triggers 429** — 350 queries/min from one tenant → 429 after 300th.
24. **Empty corpus query** — search "xyz" with no matches → returns empty `posts: []`; metric outcome=`empty`.
25. **VN corpus fixture has ≥ 500 cases**.
26. **30+ tokenizer_test.go bigram fixtures pass**.
27. **CLI debug command** — `cyberos chat search debug --query "cà phê" --tenant <id>` prints `lang=vn`, `index=bigram`, result IDs, per-stage latency.

---

## §5 — Verification

```go
// services/chat/plugins/cyberos-vn-search/tokenizer_test.go
package main

import (
    "testing"
    "github.com/stretchr/testify/assert"
)

func TestBigramSplit_VietnameseWord(t *testing.T) {
    assert.Equal(t, []string{"cà", "à ", " p", "ph", "hê"}, bigramSplit("cà phê"))
}

func TestBigramSplit_EmptyInput(t *testing.T) {
    assert.Empty(t, bigramSplit(""))
}

func TestBigramSplit_SingleChar(t *testing.T) {
    assert.Empty(t, bigramSplit("x"))
}

func TestBigramSplit_PreservesCase(t *testing.T) {
    // Lowercased before splitting
    a := bigramSplit("Café")
    b := bigramSplit("café")
    assert.Equal(t, a, b)
}

func TestBigramSplit_MultiByte(t *testing.T) {
    // Rune-aware (not byte-aware): "đệ" should produce ["đệ"]
    bg := bigramSplit("đệ")
    assert.Equal(t, []string{"đệ"}, bg)
}

func TestIsVietnameseQuery_PureVietnamese(t *testing.T) {
    assert.True(t, isVietnameseQuery("họp với khách"))
}

func TestIsVietnameseQuery_PureEnglish(t *testing.T) {
    assert.False(t, isVietnameseQuery("meeting with client"))
}

func TestIsVietnameseQuery_MixedAboveThreshold(t *testing.T) {
    assert.True(t, isVietnameseQuery("book cà phê meeting"))
}

func TestIsVietnameseQuery_MixedBelowThreshold(t *testing.T) {
    // 1 VN char out of 60 chars = 1.7% < 5%
    assert.False(t, isVietnameseQuery("This is a long English sentence with one café word"))
}

func TestIsVietnameseQuery_EmptyString(t *testing.T) {
    assert.False(t, isVietnameseQuery(""))
}
// ... 20+ more bigram fixtures for VN compound words, common queries, edge cases
```

```go
// services/chat/plugins/cyberos-vn-search/search_test.go
func TestSearch_TeamIsolation(t *testing.T) {
    p := setupPlugin(t)
    p.seedPost(testPost{ID: "p1", TeamID: "team-A", Message: "Bài kiểm tra A"})
    p.seedPost(testPost{ID: "p2", TeamID: "team-B", Message: "Bài kiểm tra B"})

    results := p.search(testSession{TeamMembership: []string{"team-A"}, TenantID: "t1"}, "kiểm tra")
    ids := postIDs(results)
    assert.Contains(t, ids, "p1")
    assert.NotContains(t, ids, "p2")
}

func TestSearch_DeletedAtFilter(t *testing.T) {
    p := setupPlugin(t)
    p.seedPost(testPost{ID: "p1", TeamID: "team-A", Message: "hoàn thành", DeletedAt: 1234567890})
    results := p.search(testSession{TeamMembership: []string{"team-A"}, TenantID: "t1"}, "hoàn thành")
    assert.Empty(t, results)
}

func TestSearch_LatencyBudget(t *testing.T) {
    p := setupPluginWithCorpus(t, 100_000)
    samples := make([]time.Duration, 50)
    for i := 0; i < 50; i++ {
        start := time.Now()
        _ = p.search(testSession{TeamMembership: []string{"team-A"}, TenantID: "t1"}, "kiểm tra")
        samples[i] = time.Since(start)
    }
    sort.Slice(samples, func(i, j int) bool { return samples[i] < samples[j] })
    p95 := samples[47]
    assert.Less(t, p95, 200*time.Millisecond, "p95 latency: %v", p95)
}

func TestSearch_RateLimit(t *testing.T) {
    p := setupPlugin(t)
    for i := 0; i < 300; i++ {
        rec := p.searchHTTP("query")
        assert.Equal(t, 200, rec.Code)
    }
    rec := p.searchHTTP("query")
    assert.Equal(t, 429, rec.Code)
}

func TestSearch_VnRoutesToBigramPath(t *testing.T) {
    p := setupPluginWithCounters(t)
    p.searchHTTP("cà phê")
    assert.Equal(t, 1, p.bigramCallCount)
    assert.Equal(t, 0, p.pgroongaCallCount)
}

func TestSearch_EnRoutesToPGroongaPath(t *testing.T) {
    p := setupPluginWithCounters(t)
    p.searchHTTP("coffee")
    assert.Equal(t, 0, p.bigramCallCount)
    assert.Equal(t, 1, p.pgroongaCallCount)
}

func TestSearch_BrainAuditEmitted(t *testing.T) {
    p := setupPlugin(t)
    p.searchHTTP("cà phê")
    row := p.brain.LastRow("chat.search_query")
    assert.NotEmpty(t, row.Payload["query_hash"])
    _, has := row.Payload["query"]
    assert.False(t, has, "raw query MUST NOT appear in audit row payload")
}
```

```bash
# Shell test for the CI gate exit codes
#!/usr/bin/env bash
# tests/recall-gate-exit-codes.sh
set -e

# Fixture with intentionally degraded tokeniser to force recall < 0.80
DEGRADED_CORPUS=/tmp/degraded.jsonl
head -200 services/chat/test-fixtures/vn-message-corpus.jsonl > "$DEGRADED_CORPUS"
# (Run with mock plugin that returns nothing → recall = 0)
CYBEROS_VN_CORPUS="$DEGRADED_CORPUS" CYBEROS_CHAT_URL=http://localhost:8066 \
  python3 services/chat/scripts/measure-recall.py
test $? -eq 1 || { echo "expected exit 1 on degraded corpus"; exit 1; }
echo "✓ gate correctly exits 1 on low recall"
```

---

## §6 — Implementation skeleton

(API contract in §3 is the skeleton: SQL migrations + Mattermost plugin + measure-recall.py + CI workflow.)

**Build flow:**
1. Add PGroonga to RDS parameter group via FR-CHAT-003 Terraform module.
2. Run init migrations on RDS bootstrap.
3. Build plugin via Mattermost SDK (`mattermost plugin manifest`).
4. Install plugin via FR-CHAT-001 patch loading mechanism.
5. CI runs the recall gate on every PR touching search code.

---

## §7 — Dependencies

- **FR-CHAT-003 (upstream)** — RDS host; Terraform module enables PGroonga extension via parameter group.
- **FR-CHAT-005 (related)** — BRAIN bridge picks up `chat.search_query` audit rows.
- **FR-CHAT-001 (upstream)** — Mattermost fork patches load this plugin; `020-search-route.patch` routes search to plugin.
- **FR-CHAT-002 (upstream)** — JWT auth + `tenant_id` propagation in session props.
- **FR-AUTH-003** — RLS on `posts` table; this FR's queries inherit.
- **FR-BRAIN-108** — sibling search surface (BRAIN vector + graph search); CHAT search is text-only.
- **FR-OBS-003** — standardised histogram buckets reused.
- **FR-OBS-007** — alerts when `chat_search_recall` gauge < 0.80.

---

## §8 — Example payloads

### `POST /plugins/cyberos.vn-search/search` request

```json
{
  "query": "cà phê đen",
  "team_id": "tm-...",
  "page": 0,
  "per_page": 20
}
```

### Response

```json
{
  "posts": [
    {
      "id": "p-01HZK9R8M3X5C8Q4",
      "message": "Tôi vừa pha cà phê đen cho team",
      "user_id": "u-...",
      "channel_id": "c-...",
      "create_at": 1747407137483
    }
  ],
  "total_count": 1,
  "latency_ms": 47,
  "query_lang": "vn"
}
```

### `chat.search_query` BRAIN audit row

```json
{
  "kind": "chat.search_query",
  "payload": {
    "user_id": "u-...",
    "team_id": "tm-...",
    "query_hash": "9b0e8c5...",
    "query_lang": "vn",
    "result_count": 1,
    "latency_ms": 47,
    "trace_id": "0af7651916cd43dd8448eb211c80319c"
  }
}
```

### CI gate output (failure case)

```text
recall   = 0.762    (gate: >= 0.80)
fp_rate  = 0.041   (gate: <= 0.05)
counts   tp=2418 fn=755 fp=124 tn=2903
::error::recall or fp_rate outside acceptable range
```

---

## §9 — Open questions

All resolved. Deferred:
- Trigram path for power users who want higher recall (slice 4+; opt-in via channel setting).
- Stemming for English path (Porter stemmer) — slice 4+; current naïve match is acceptable.
- Cross-channel search via vector embeddings (BRAIN search delegation per FR-BRAIN-108) — slice 4+; semantic queries cross over.
- Mention-aware search ("@alice meeting") — slice 4+; requires Mattermost mention table join.
- VN diacritic-stripping fallback (search "ca phe" should hit "cà phê") — slice 4+; ~5% recall improvement.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| PGroonga extension missing | `CREATE EXTENSION` Err on bootstrap | RDS bootstrap fails; sev-1 | Operator runs migration manually; or FR-CHAT-003 module updated |
| `detect_vn` returns wrong type | sqlx type Err | Plugin query fails 500 | Operator inspects function definition |
| `bigram_split` returns NULL | array overlap empty | Plugin returns empty results | Verify input not NULL upstream |
| Recall drops below 0.80 on PR | CI gate exits 1 | PR blocked | Author tunes tokeniser or corpus |
| False-positive rate exceeds 0.05 | CI gate | PR blocked | Same as above |
| Mattermost API change in v9.x patch | Plugin compile Err | Plugin fails to load | Operator updates plugin SDK pin |
| RDS connection pool exhausted | `db.QueryContext` Err | 503 returned; sev-2 | Operator scales tier |
| Rate-limit governor drops legit users | metric `outcome=rate_limited` spike | Sev-2 alarm | Operator tunes per-tenant limits |
| Mixed-language query miscategorised | wrong path chosen | Lower recall for that query; metric records | Operator tunes 5% threshold |
| Audit row emit fails | BRAIN socket down | Search still succeeds; audit lost; sev-2 | Operator restores FR-BRAIN-107 |
| Query > 1KB | Mattermost API rejects upstream | 413 returned | Plugin truncates input |
| SQL injection via query parameter | parameterised queries prevent | Safe | None |
| Bigram table overlap returns 1M rows | `LIMIT` clause caps; latency budget catches | sev-2 latency alarm | Operator tunes pagination defaults |
| 100K+ message corpus exceeds budget | latency p95 > 200ms | Sev-2 alarm; FR-CHAT-003 tier upgrade recommended | Operator scales |
| Partial index `WHERE detect_vn(message)` drops on schema change | `EXPLAIN` shows seq scan | Sev-2; manual REINDEX | Operator runs maintenance |
| CI gate flaky on small fixture changes | recall fluctuates ±2% | Author adjusts corpus to stabilise | None |
| Unicode normalisation drift (NFC vs NFD) | bigram mismatch | False negatives | Plugin applies NFC at query time (matches FR-CHAT-007 import pattern) |
| Operator searches with raw PII as query | audit row hashes the query | Safe | None |

---

## §11 — Implementation notes

- `IMMUTABLE PARALLEL SAFE` on the PL/pgSQL functions enables the planner to use them in partial-index `WHERE` clauses without recomputation per row. Without these markers, the partial index would be rejected.
- The Vietnamese tonal-mark character set in `detect_vn` mirrors the one in `isVietnameseRune` Go function — both must stay in sync. A CI test asserts the SQL function and Go function agree on a 100-case fixture.
- `bigram_split` lowercases input — case-insensitive match. Matches Mattermost's existing search behaviour.
- The partial index `WHERE detect_vn(message)` saves ~70% index space on a mixed-language tenant (most messages are not VN). For a VN-only tenant, the partial predicate is essentially always true; saved space is negligible.
- Pagination uses `LIMIT/OFFSET` not keyset pagination — acceptable for typical search depth (< 100 results); revisit if users routinely scroll past page 5.
- The CI workflow's PostgreSQL service uses `groonga/pgroonga:latest-alpine-16` Docker image (official upstream).
- The rate limiter is in-process per-tenant via `governor` crate (same pattern as FR-AI-021 + FR-SKILL-108). Cross-pod state would need Redis (slice 4+).
- `chat.search_query` audit rows aggregate to a per-tenant per-day "search heatmap" via FR-OBS-008 compliance view — informs which terms users search most.
- Corpus maintenance: the security + product team adds ~10 new VN cases per quarter (real-world false negatives observed). The corpus is version-controlled; PR review on changes.
- Unicode NFC normalisation: VN text from imports (FR-CHAT-006/007) may arrive NFD; plugin applies `string.ToValidUTF8` + relies on Postgres's `unaccent` extension being NOT installed (since we want diacritics to match exactly). Diacritic-stripping fallback is slice-4+.

---

*End of FR-CHAT-004.*
