#!/usr/bin/env bash
# check_defensive_asserts.sh - TASK-IMP-022 §1. The mechanical half of TRACE-008: scans the
# GATED test corpus for assertions that cannot fail, so a green suite cannot mean "nothing was
# checked". Origin R13 (docs/strategy/cyberos-deep-audit-and-auto-evolution-plan-2026-07-06.md
# line 50): the memory-writer contract bug shipped because a test asserted
# `processed == 3 || failed > 0` - true whenever the writer failed, so the suite stayed green
# through the defect it existed to catch.
#
# exit 0 clean | exit 10 with `DEFENSIVE <file>:<line> [<rule>] <detail>` lines | exit 2 unusable.
# --list prints every finding AND every active waiver with status, always exit 0.
#
# CORPUS = exactly the suites run_all.sh globs (scripts/tests, tools/install/tests,
# tools/docs-site/tests) plus every `tests/` tree under modules/. Deriving the shell roots from
# the runner's own glob is deliberate: a suite the runner cannot reach is not gated, and a suite
# this lint cannot reach is not audited - one list, so the two sets cannot drift apart.
#
# OUT OF SCOPE, stated rather than silently narrowed: services/** (Rust/Go/Python platform code)
# is outside CyberOS 1.x's payload per docs/batches/batch-10e-imp-stub-wont-do.md. Rust `assert!`
# / `assert_eq!` are not scanned by any rule here.
#
# WAIVER: `# defensive-assert-ok: <reason>` on the flagged line or the line immediately above.
# The reason MUST be >= 12 characters; a waiver without one is itself a finding (DA-005). The
# waiver is inline rather than in a sibling exemptions file (the doc-anchor-exemptions.txt
# precedent) because an assert waiver is line-granular: the reviewer reading the assertion has
# to see why the disjunction is legitimate without opening a second file.
set -uo pipefail
repo="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
mode="check"; [ "${1:-}" = "--list" ] && mode="list"
python3 - "$repo" "$mode" <<'PY'
import ast, os, re, sys

repo, mode = sys.argv[1], sys.argv[2]

# The three shell roots below are the three globbed by scripts/tests/run_all.sh. Keep in sync:
# a root added there without being added here is gated but unaudited.
SHELL_ROOTS = ["scripts/tests", "tools/install/tests", "tools/docs-site/tests"]
PY_ROOT = "modules"

WAIVER = re.compile(r"#\s*defensive-assert-ok:\s*(.*)$")
MIN_REASON = 12

findings, waivers = [], []


def rel(path):
    return os.path.relpath(path, repo)


def waiver_reason(lines, lineno):
    """Reason from a waiver on `lineno` (1-based) or the line immediately above, else None."""
    for probe in (lineno, lineno - 1):
        if 1 <= probe <= len(lines):
            m = WAIVER.search(lines[probe - 1])
            if m:
                return m.group(1).strip()
    return None


def record(path, lineno, rule, detail, lines):
    reason = waiver_reason(lines, lineno)
    if reason is None:
        findings.append(f"DEFENSIVE {rel(path)}:{lineno} [{rule}] {detail}")
        return
    if len(reason) < MIN_REASON:
        findings.append(
            f"DEFENSIVE {rel(path)}:{lineno} [DA-005] waiver reason is {len(reason)} chars, "
            f"minimum {MIN_REASON}: {reason!r}"
        )
        return
    waivers.append(f"waived      {rel(path)}:{lineno} [{rule}] {reason}")


# ── Python: AST, never grep ────────────────────────────────────────────────────────────────
# A regex for `assert .* or .*` flags `assert sources == {"dropbox-or-gdrive"}` (a hyphen inside
# a string literal) and misses a disjunction split across lines inside parentheses. The AST sees
# the BoolOp and nothing else, so both classes are handled by construction.

def is_len_call(node):
    return isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id == "len"


def vacuous_reason(test):
    """Why `assert <test>` is true for every input, or None when it can genuinely fail."""
    if isinstance(test, ast.Constant):
        if test.value:
            return f"`assert {ast.unparse(test)}` is a truthy constant"
        return None  # `assert False` / `assert 0` is an intentional unreachable marker
    if isinstance(test, (ast.Tuple, ast.List, ast.Dict, ast.Set)) and (
        getattr(test, "elts", None) or getattr(test, "keys", None)
    ):
        return "non-empty literal container is always truthy (missing comma turns the message into a tuple?)"
    if isinstance(test, (ast.Lambda, ast.JoinedStr)):
        return "a lambda / f-string object is always truthy"
    if isinstance(test, ast.Compare) and len(test.ops) == 1 and len(test.comparators) == 1:
        left, op, right = test.left, test.ops[0], test.comparators[0]
        rv = right.value if isinstance(right, ast.Constant) else None
        if is_len_call(left):
            if isinstance(op, ast.GtE) and rv == 0:
                return "`len(...) >= 0` holds for every sequence, including the empty one"
            if isinstance(op, ast.Gt) and rv == -1:
                return "`len(...) > -1` holds for every sequence, including the empty one"
    return None


def scan_python(path):
    src = open(path, encoding="utf-8", errors="replace").read()
    lines = src.splitlines()
    try:
        tree = ast.parse(src, filename=path)
    except SyntaxError as exc:
        findings.append(f"DEFENSIVE {rel(path)}:{exc.lineno or 0} [DA-000] unparsable: {exc.msg}")
        return
    for node in ast.walk(tree):
        if not isinstance(node, ast.Assert):
            continue
        ors = [n for n in ast.walk(node.test) if isinstance(n, ast.BoolOp) and isinstance(n.op, ast.Or)]
        if ors:
            record(
                path, node.lineno, "DA-001",
                f"disjunctive assertion - passes on either arm: assert {ast.unparse(ors[0])}",
                lines,
            )
            continue
        why = vacuous_reason(node.test)
        if why:
            record(path, node.lineno, "DA-002", f"assertion cannot fail - {why}", lines)


# ── Shell: line scanner, no shell parsing ──────────────────────────────────────────────────
# Only two shapes are claimed, both decidable from one line. Polarity inference (`if <A||B>;
# then ok` is weak, `if <A||B>; then fail` is a STRONGER conjunctive assertion) is deliberately
# NOT attempted: it needs the then-branch, and guessing it wrong would flag the correct form.
# TRACE-008 covers the rest by judgment - see modules/skill/task-audit/RUBRIC.md §9.

PROBE = r"(?:grep|rg|test|diff|cmp|\[\[?)"
SWALLOWED = re.compile(rf"(?:^|;|&&|\|\||\bthen\b|\{{)\s*{PROBE}\s[^|;]*\|\|\s*(?:true|:)\s*(?:#.*)?$")
ASSIGN = re.compile(r"^\s*(?:local\s+|export\s+|declare\s+)?[A-Za-z_][A-Za-z0-9_]*=")
VACUOUS_NUM = re.compile(r"-ge\s+0(?:\s|\]|$)|-gt\s+-1(?:\s|\]|$)")


# A heredoc body is DATA to the shell, not shell it executes: `grep -q x f || true` inside
# `cat >fixture <<'EOF'` is a fixture, and DA-003/DA-004 are claims about executed shell. Skipping
# heredoc bodies is therefore correctness, not an exemption — and it is what lets a lint against
# defensive assertions carry the negative fixtures that prove it fires. Stated bound: assertions
# inside an embedded interpreter heredoc (`python3 - <<'PY'`) are outside the mechanical floor;
# TRACE-008's judgment half is what covers them.
HEREDOC = re.compile(r"<<-?\s*(?:'([^']+)'|\"([^\"]+)\"|([A-Za-z_][A-Za-z0-9_]*))")


def scan_shell(path):
    lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
    terminator = None
    for i, line in enumerate(lines, 1):
        if terminator is not None:
            if line.strip() == terminator:
                terminator = None
            continue
        code = line.split("#", 1)[0] if not WAIVER.search(line) else line
        stripped = code.strip()
        if not stripped or stripped.startswith("#"):
            continue
        m = HEREDOC.search(code)
        if m:
            terminator = m.group(1) or m.group(2) or m.group(3)
            continue
        # An assignment capturing a probe's OUTPUT legitimately discards its status
        # (`n="$(grep -c x f)" || true` - grep exits 1 on zero matches). Not an assertion.
        if SWALLOWED.search(code) and not ASSIGN.match(code):
            record(path, i, "DA-003", "probe's failure is swallowed by `|| true` / `|| :`", lines)
            continue
        if VACUOUS_NUM.search(code):
            record(path, i, "DA-004", "numeric comparison holds at zero - a count test that passes on nothing", lines)


# ── Walk ───────────────────────────────────────────────────────────────────────────────────
scanned = 0
for root in SHELL_ROOTS:
    base = os.path.join(repo, root)
    if not os.path.isdir(base):
        print(f"WARN shell root missing: {root}", file=sys.stderr)
        continue
    for dp, _, files in os.walk(base):
        for f in sorted(files):
            if f.startswith("test_") and f.endswith(".sh"):
                scan_shell(os.path.join(dp, f)); scanned += 1

py_base = os.path.join(repo, PY_ROOT)
for dp, dirs, files in os.walk(py_base):
    dirs[:] = [d for d in dirs if d not in (".git", "__pycache__", "node_modules")]
    if os.path.basename(dp) != "tests" and f"{os.sep}tests{os.sep}" not in dp + os.sep:
        continue
    for f in sorted(files):
        if f.startswith("test_") and f.endswith(".py"):
            scan_python(os.path.join(dp, f)); scanned += 1

findings.sort()
waivers.sort()
if mode == "list":
    print("\n".join(findings + waivers))
    print(f"files={scanned} findings={len(findings)} waivers={len(waivers)}")
    sys.exit(0)
for w in waivers:
    print(w, file=sys.stderr)
if findings:
    print("\n".join(findings))
    print(
        f"\n{len(findings)} assertion(s) that cannot fail. Assert the ONE behaviour the code has "
        "(run it and look), or waive with `# defensive-assert-ok: <reason>` (>= 12 chars).",
        file=sys.stderr,
    )
    sys.exit(10)
print(f"defensive-assert OK: {scanned} gated test files scanned, {len(waivers)} waived")
PY
exit $?
