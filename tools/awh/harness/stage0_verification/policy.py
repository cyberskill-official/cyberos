"""Deterministic PreToolUse policy engine.

A *deny rule wins over any allow* — the canonical Claude Code containment pattern.
This module is the pure, side-effect-free decision core so it can be unit-tested
exhaustively; the hook script in ``hooks/pretooluse_deny.py`` is a thin shell
around ``evaluate_event``.

Two default deny classes, both grounded in published incidents:

  * Destructive shell commands — ``rm -rf /``, force-pushes, ``mkfs``, fork bombs,
    ``DROP DATABASE``, ``curl … | sh``.
  * Secret reads/exfiltration — ``.env``, ``~/.aws/credentials``, ``*.pem``, ``id_rsa``,
    ``.ssh/`` via ``cat``/``cp``/``tar``/``scp``/``source``/``curl -d @…`` and friends.
    A phishing prompt got Claude to exfiltrate ``~/.aws/credentials`` in 24/25 retries
    (Anthropic, "How we contain Claude across products", 2026).

SCOPE — read this honestly. A regex/glob deny-list catches the common, obvious
cases; it is high-signal defense-in-depth, NOT a complete sandbox. The survey's
own lesson is that patterns are unreliable and *structure* wins, so a determined
bypass (an unusual verb, an obfuscated path, ``base64`` round-tripping) can evade
any blocklist. The real containment boundary is the default-deny egress sandbox in
``sandbox/`` (the only thing that held in Anthropic's exfil incident). Run BOTH:
this gate stops the lazy/obvious failure fast; the sandbox stops the rest.
"""

from __future__ import annotations

import fnmatch
import posixpath
import re
from dataclasses import dataclass, field

__all__ = ["Decision", "Policy", "default_policy", "evaluate_event"]

# Tools whose file_path argument should be checked against secret globs.
_FILE_TOOLS = ("Read", "Edit", "Write", "NotebookEdit", "MultiEdit")
# Tools that MODIFY a file (subset of _FILE_TOOLS) — subject to read-only/deny-write rules.
_WRITE_TOOLS = ("Edit", "Write", "NotebookEdit", "MultiEdit")
# Shell tokens that indicate a command writes/mutates a target path. Includes the
# copy/link verbs (cp/ln/rsync/install) — copying INTO a read-only path is a write.
_WRITE_INDICATORS = (
    ">",
    ">>",
    "tee",
    "sed",
    "truncate",
    "chmod",
    "dd",
    "mv",
    "rm",
    "cp",
    "ln",
    "rsync",
    "install",
)

# Default destructive-command signatures (case-insensitive, matched on the command string).
_DEFAULT_DENY_COMMANDS: tuple[str, ...] = (
    # recursive+force rm (flags contiguous OR split: -rf, -fr, -r -f) targeting a
    # dangerous root. Absolute/system dirs may be followed by "/" (e.g. /home/user);
    # but "." and "*" must be followed by space/end so "./build" stays allowed.
    r"\brm\b(?=[^\n]*\s(?:-\w*r|--recursive))(?=[^\n]*\s(?:-\w*f|--force))[^\n]*\s"
    r"(?:(/|~|\$HOME|/etc|/usr|/var|/bin|/lib|/boot|/sys|/proc|/opt|/home)(\s|/|$)"
    r"|/\*"
    r"|(\*|\.)(\s|$))",
    r"\bfind\s+(/|~|\$HOME)[^\n]*-(delete|exec\s+rm)\b",  # find / ... -delete / -exec rm
    r"\bgit\s+push\b[^\n]*\s(--force\b|-f\b)",  # force push
    r"\bgit\s+push\b[^\n]*\s\+",  # refspec force (+branch)
    r"\bgit\s+reset\s+--hard\b[^\n]*\borigin/",  # hard reset to remote
    r"\bgit\s+clean\s+-[a-z]*f[a-z]*d",  # git clean -fd
    r"\bgit\s+branch\s+-D\b",  # force-delete branch
    r":\s*\(\s*\)\s*\{\s*:\s*\|\s*:",  # fork bomb :(){ :|:& };:
    r"\bmkfs\b",  # format filesystem
    r"\bdd\b[^\n]*\bof=/dev/",  # dd to a device
    r">\s*/dev/sd[a-z]",  # redirect to a disk
    r"\bchmod\s+-[a-z]*R[a-z]*\s+0*777\s+/",  # chmod -R 777 /
    r"\bsudo\b",  # privilege escalation
    r"\bDROP\s+(DATABASE|TABLE|SCHEMA)\b",  # destructive SQL
    r"\bTRUNCATE\s+TABLE\b",
    r"(curl|wget)\b[^\n|]*\|\s*(sudo\s+)?(ba)?sh\b",  # curl … | sh
)

# Default secret/credential path signatures (glob form; matched on full path and basename).
_DEFAULT_DENY_PATHS: tuple[str, ...] = (
    "*.env",
    ".env",
    ".env.*",
    "*/.env",
    "*/.env.*",
    "*/secrets/*",
    "secrets/*",
    "*.pem",
    "*.key",
    "*.p12",
    "*.pfx",
    "id_rsa",
    "id_dsa",
    "id_ecdsa",
    "id_ed25519",
    "*/.ssh/*",
    "*/.aws/credentials",
    "*/.aws/config",
    "*.netrc",
    ".netrc",
    "*credentials.json",
    "*service-account*.json",
    "*.kdbx",
)

# Commands that read OR move/transmit a file — used to extend secret denial to Bash.
# Covers viewing (cat/head), copying (cp/tar/zip), transmitting (scp/rsync/curl/wget),
# sourcing (source/.), and encoding round-trips (base64/openssl) of a secret path.
_TOUCH_CMDS = (
    "cat",
    "less",
    "more",
    "head",
    "tail",
    "bat",
    "nl",
    "xxd",
    "od",
    "strings",
    "grep",
    "cp",
    "mv",
    "scp",
    "rsync",
    "tar",
    "zip",
    "gzip",
    "install",
    "ln",
    "dd",
    "curl",
    "wget",
    "base64",
    "openssl",
    "source",
    ".",
)


@dataclass(frozen=True)
class Decision:
    """Result of evaluating one tool event."""

    block: bool
    reason: str = ""
    rule: str = ""  # which rule fired, for auditability

    @property
    def allow(self) -> bool:
        return not self.block


@dataclass
class Policy:
    """A deny-list policy. Deny always wins; there is intentionally no allow-list
    that can override a deny (that is the whole point of containment)."""

    deny_command_patterns: list[str] = field(default_factory=lambda: list(_DEFAULT_DENY_COMMANDS))
    deny_path_globs: list[str] = field(default_factory=lambda: list(_DEFAULT_DENY_PATHS))
    # Paths that may be READ but never WRITTEN — the read-only test/scoring "middle
    # ground" from ImpossibleBench. Populated by the readonly tool or operator config.
    deny_write_globs: list[str] = field(default_factory=list)
    # Extra paths the operator explicitly allows reading (rarely needed; never overrides a secret).
    extra_command_patterns: list[str] = field(default_factory=list)
    extra_path_globs: list[str] = field(default_factory=list)

    def all_command_patterns(self) -> list[str]:
        return [*self.deny_command_patterns, *self.extra_command_patterns]

    def all_path_globs(self) -> list[str]:
        return [*self.deny_path_globs, *self.extra_path_globs]

    @classmethod
    def from_dict(cls, data: dict) -> Policy:
        # Fail safe: a non-object policy, or a field that isn't a list[str], falls
        # back to the strict default for THAT field rather than silently disabling
        # containment (e.g. a bare string "rm" must not become list("rm")=['r','m']).
        if not isinstance(data, dict):
            return cls()

        def _str_list(key: str, default) -> list[str]:
            v = data.get(key, default)
            if not isinstance(v, list) or not all(isinstance(x, str) for x in v):
                return list(default)
            return list(v)

        return cls(
            deny_command_patterns=_str_list("deny_command_patterns", _DEFAULT_DENY_COMMANDS),
            deny_path_globs=_str_list("deny_path_globs", _DEFAULT_DENY_PATHS),
            deny_write_globs=_str_list("deny_write_globs", []),
            extra_command_patterns=_str_list("extra_command_patterns", []),
            extra_path_globs=_str_list("extra_path_globs", []),
        )


def default_policy() -> Policy:
    """The shipped defaults — safe for an unattended `auto/` session."""
    return Policy()


# Split a shell command into tokens. Includes redirection operators (< > ) so that
# `cat<.env` / `echo x>tests/y` tokenize their target path out (a known evasion).
_TOKEN_SPLIT = re.compile(r"[\s;|&<>]+")


def _candidate_paths(path: str) -> list[str]:
    """All path forms a glob should be tested against, so a deny can't be dodged by a
    ``./`` prefix, an absolute path, ``..`` segments, or surrounding quotes.

    Produces the quote-stripped path, its ``posixpath.normpath`` form, and every
    ``/``-suffix of both (so ``/repo/tests/x.py`` still matches a relative glob like
    ``tests/*``). Over-matching here only ever *adds* denials — the safe direction.
    """
    p = path.strip().strip('"').strip("'")
    if not p:
        return [""]
    cands: set[str] = {p, posixpath.normpath(p)}
    for s in (p, posixpath.normpath(p)):
        parts = [seg for seg in s.split("/") if seg not in ("", ".")]
        for i in range(len(parts)):
            cands.add("/".join(parts[i:]))
    return [c for c in cands if c]


def _path_matches(path: str, globs: list[str]) -> str | None:
    """Return the first glob that matches ``path`` (any normalized form or basename)."""
    cands = _candidate_paths(path)
    for g in globs:
        for c in cands:
            if fnmatch.fnmatch(c, g) or fnmatch.fnmatch(c.rsplit("/", 1)[-1], g):
                return g
    return None


def _command_hits_secret(command: str, globs: list[str]) -> str | None:
    """Detect a read/copy/transmit command targeting a secret path inside a Bash command."""
    tokens = _TOKEN_SPLIT.split(command)
    if not tokens:
        return None
    saw_touch_cmd = any(t.rsplit("/", 1)[-1] in _TOUCH_CMDS for t in tokens)
    if not saw_touch_cmd:
        return None
    for tok in tokens:
        hit = _path_matches(tok.lstrip("@"), globs)
        if hit:
            return hit
    return None


def _command_writes_path(command: str, globs: list[str]) -> str | None:
    """Detect a shell command that writes/mutates a read-only-protected path."""
    if not globs:
        return None
    tokens = _TOKEN_SPLIT.split(command)
    cmd_words = {t.rsplit("/", 1)[-1] for t in tokens}
    has_write = (">" in command) or bool(cmd_words & set(_WRITE_INDICATORS))
    if not has_write:
        return None
    for tok in tokens:
        hit = _path_matches(tok, globs)
        if hit:
            return hit
    return None


def evaluate_event(event: dict, policy: Policy | None = None) -> Decision:
    """Evaluate a Claude Code PreToolUse event dict and decide allow/deny.

    Args:
        event: the JSON object Claude Code passes on stdin. Relevant keys:
            ``tool_name`` and ``tool_input`` (a dict with e.g. ``command`` for Bash,
            ``file_path`` for Read/Edit/Write).
        policy: deny-list to apply; ``default_policy()`` if None.

    Returns:
        Decision(block, reason, rule). ``block=True`` means the hook must exit 2.
    """
    pol = policy or default_policy()
    tool = event.get("tool_name", "")
    tool_input = event.get("tool_input") or {}

    # 1. Bash commands -> destructive-command + secret-read checks.
    if tool == "Bash":
        command = str(tool_input.get("command", ""))
        # Match destructive signatures against the raw command AND a dequoted copy,
        # so quoting a dangerous target (e.g. rm -rf '/') can't slip past a pattern.
        cmd_variants = (command, command.replace("'", "").replace('"', ""))
        for pat in pol.all_command_patterns():
            if any(re.search(pat, c, flags=re.IGNORECASE) for c in cmd_variants):
                return Decision(
                    block=True,
                    reason=(
                        "Blocked by Stage-0 policy: command matches a destructive "
                        f"signature ({pat!r}). If this is genuinely required, the human "
                        "operator must run it — an unattended agent must not."
                    ),
                    rule=f"deny_command:{pat}",
                )
        hit = _command_hits_secret(command, pol.all_path_globs())
        if hit:
            return Decision(
                block=True,
                reason=(
                    f"Blocked by Stage-0 policy: command reads a secret path ({hit!r}). "
                    "Secrets must be supplied via a secret store, never read into context."
                ),
                rule=f"deny_secret_read_cmd:{hit}",
            )
        whit = _command_writes_path(command, pol.deny_write_globs)
        if whit:
            return Decision(
                block=True,
                reason=(
                    f"Blocked by Stage-0 policy: command modifies a read-only test/scoring "
                    f"file ({whit!r}). Never edit the tests to make code pass — fix the code, "
                    "or mark the task BLOCKED if the test is genuinely wrong."
                ),
                rule=f"deny_write_cmd:{whit}",
            )
        return Decision(block=False)

    # 2. File tools -> secret-path checks on file_path (read OR write of secrets denied).
    if tool in _FILE_TOOLS:
        path = str(tool_input.get("file_path") or tool_input.get("notebook_path") or "")
        hit = _path_matches(path, pol.all_path_globs())
        if hit:
            return Decision(
                block=True,
                reason=(
                    f"Blocked by Stage-0 policy: {tool} touches a secret/credential path "
                    f"({hit!r}). Deny wins over allow — this boundary is what holds when a "
                    "prompt-injection slips through (Anthropic containment, 2026)."
                ),
                rule=f"deny_path:{hit}",
            )
        # Read-only test/scoring files: writes denied, reads still allowed.
        if tool in _WRITE_TOOLS:
            whit = _path_matches(path, pol.deny_write_globs)
            if whit:
                return Decision(
                    block=True,
                    reason=(
                        f"Blocked by Stage-0 policy: {tool} would modify a read-only "
                        f"test/scoring file ({whit!r}). Test files are read-only to prevent "
                        "reward hacking (ImpossibleBench >79% of cheats are test edits). "
                        "Fix the code under test, or mark the task BLOCKED."
                    ),
                    rule=f"deny_write_path:{whit}",
                )
        return Decision(block=False)

    # 3. Everything else is allowed by this gate (other gates may still apply).
    return Decision(block=False)
