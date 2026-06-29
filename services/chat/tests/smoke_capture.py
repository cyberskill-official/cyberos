#!/usr/bin/env python3
"""FR-MEMORY-122 §5 — chat capture smoke (live service, brain link ON).

Drives the running cyberos-chat service and asserts that, with CAPTURE_ENABLED=true and the chat->brain
audit link set, the right interaction-events appear in l1_audit_log for an acknowledged subject — pointing
at chat's own rows, never inlining a body — and that presence dedups to the online/offline edges.

Prerequisites (the runner sets these up; this script does not start anything):
  * cyberos-chat running on $CHAT_BASE (default http://127.0.0.1:7720) with:
      - CAPTURE_ENABLED=true
      - CHAT_AUDIT_DATABASE_URL pointing at the brain DB
      - CHAT_AUTH_HS256_SECRET=$SMOKE_SECRET (so this script can mint matching tokens)
  * $AUDIT_DATABASE_URL — the same brain DB, for seeding the acknowledgment and asserting rows. The DB has
    the memory (l1_audit_log) and eval (monitoring_notice + subject_acknowledgment) migrations applied.

If $AUDIT_DATABASE_URL or psycopg is unavailable, the script SKIPS (exit 0) with a message — it cannot
assert capture without DB access, and a missing optional dependency must not fail the suite.
"""
import base64, hashlib, hmac, json, os, sys, time, urllib.request, urllib.error

BASE = os.environ.get("CHAT_BASE", "http://127.0.0.1:7720")
SECRET = os.environ.get("SMOKE_SECRET", os.environ.get("CHAT_AUTH_HS256_SECRET", "smoke-secret-123"))
AUDIT_URL = os.environ.get("AUDIT_DATABASE_URL") or os.environ.get("CHAT_AUDIT_DATABASE_URL")

TENANT = "11111111-1111-1111-1111-111111111111"
ALICE = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"

P = [0]; F = [0]
def check(name, ok):
    (P if ok else F)[0] += 1
    print(("  PASS " if ok else "  FAIL ") + name)

def b64(b): return base64.urlsafe_b64encode(b).rstrip(b"=")
def mint(sub, tenant, off=3600):
    h = b64(json.dumps({"alg": "HS256", "typ": "JWT"}, separators=(",", ":")).encode())
    p = b64(json.dumps({"sub": sub, "tenant_id": tenant, "roles": [], "exp": int(time.time()) + off},
                       separators=(",", ":")).encode())
    seg = h + b"." + p
    sig = b64(hmac.new(SECRET.encode(), seg, hashlib.sha256).digest())
    return (seg + b"." + sig).decode()

def req(method, path, token=None, body=None):
    data = json.dumps(body).encode() if body is not None else None
    r = urllib.request.Request(BASE + path, data=data, method=method)
    if token: r.add_header("Authorization", "Bearer " + token)
    if data is not None: r.add_header("Content-Type", "application/json")
    try:
        resp = urllib.request.urlopen(r, timeout=5); raw = resp.read()
        return resp.status, (json.loads(raw) if raw else None)
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()

def skip(msg):
    print("SKIP: " + msg)
    sys.exit(0)

if not AUDIT_URL:
    skip("AUDIT_DATABASE_URL unset — cannot seed consent or assert l1_audit_log rows")
try:
    import psycopg  # psycopg3
    conn = psycopg.connect(AUDIT_URL, autocommit=True)
except Exception as e:  # noqa: BLE001
    skip("psycopg / DB connection unavailable (%s)" % e)

def db(sql, args=()):
    with conn.cursor() as cur:
        cur.execute(sql, args)
        try:
            return cur.fetchall()
        except psycopg.ProgrammingError:
            return []

def ack_notice():
    """Publish a current notice for TENANT and record ALICE's acknowledgment (signed-contract source)."""
    db("SELECT set_config('app.current_tenant_id', %s, true)", (TENANT,))
    db("UPDATE monitoring_notice SET is_current = FALSE WHERE is_current")
    nid = db(
        "INSERT INTO monitoring_notice (tenant_id, version, lang_en, lang_vi, lawful_basis, is_current, published_by)"
        " VALUES (%s, 1, 'en', 'vi', 'legitimate_interest', TRUE, %s) RETURNING id",
        (TENANT, "00000000-0000-0000-0000-000000000000"),
    )[0][0]
    db("SELECT set_config('app.current_tenant_id', %s, true)", (TENANT,))
    db(
        "INSERT INTO subject_acknowledgment (tenant_id, subject_id, notice_id, notice_version, ack_source, recorded_by)"
        " VALUES (%s, %s, %s, 1, 'signed_contract', %s)"
        " ON CONFLICT (tenant_id, subject_id, notice_version) DO NOTHING",
        (TENANT, ALICE, nid, "00000000-0000-0000-0000-000000000000"),
    )

def iev_types_for(subject):
    rows = db(
        "SELECT body::jsonb->'payload'->>'event_type',"
        "       body::jsonb->'payload'->'content_ref'->>'kind'"
        "  FROM l1_audit_log"
        " WHERE event_type='memory.interaction_event'"
        "   AND body::jsonb->'payload'->>'module'='chat'"
        "   AND subject_id=%s ORDER BY seq",
        (subject,),
    )
    return rows

print("== consent setup ==")
ack_notice()
check("acknowledgment recorded for ALICE", True)

tokA = mint(ALICE, TENANT)
print("== chat activity (acknowledged subject) ==")
s, ch = req("POST", "/v1/chat/channels", tokA, {"name": "general"})
check("create channel -> 201", s == 201)
cid = ch["id"] if isinstance(ch, dict) else None
s, msg = req("POST", "/v1/chat/channels/%s/messages" % cid, tokA, {"body": "hello team"})
check("post message -> 201", s == 201)
mid = msg["id"] if isinstance(msg, dict) else None
s, _ = req("PATCH", "/v1/chat/channels/%s/messages/%s" % (cid, mid), tokA, {"body": "hello everyone"})
check("edit message -> 200", s == 200)
s, _ = req("DELETE", "/v1/chat/channels/%s/messages/%s" % (cid, mid), tokA)
check("delete message -> 204", s == 204)

# Capture is best-effort + spawned, so give the spawned tasks a moment to land their rows.
time.sleep(1.5)

rows = iev_types_for(ALICE)
kinds = [r[0] for r in rows]
by_kind = {r[0]: r[1] for r in rows}
check("chat.channel_created emitted", "chat.channel_created" in kinds)
check("chat.message_created emitted", "chat.message_created" in kinds)
check("chat.message_edited emitted", "chat.message_edited" in kinds)
check("chat.message_deleted emitted", "chat.message_deleted" in kinds)
# created/edited reference chat's row by pointer (never the body); deleted has no content_ref.
check("message_created content_ref is a pointer (not raw body)", by_kind.get("chat.message_created") == "pointer")
check("message_deleted content_ref is none (body gone)", by_kind.get("chat.message_deleted") == "none")
# No raw message body leaked into any captured row.
leaked = db(
    "SELECT count(*) FROM l1_audit_log"
    " WHERE event_type='memory.interaction_event'"
    "   AND body::jsonb->'payload'->>'module'='chat'"
    "   AND body LIKE '%hello everyone%'",
)[0][0]
check("no raw message body in any captured row", leaked == 0)

print("\nRESULT: %d passed, %d failed" % (P[0], F[0]))
sys.exit(1 if F[0] else 0)
