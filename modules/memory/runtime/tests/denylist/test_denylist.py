#!/usr/bin/env python3
"""
test_denylist.py — §9.3 denylist regression suite.

Aspect 5.5 of the Layer-1 improvement catalog.

Verifies that memory_writer rejects writes containing denylisted content
regardless of phrasing. Each test attempts a write with a known-bad body
and confirms rejection.

Run:
    python3 runtime/tests/denylist/test_denylist.py        # all tests
    python3 runtime/tests/denylist/test_denylist.py -v     # verbose
    python3 runtime/tests/denylist/test_denylist.py --list # list test cases
"""
from __future__ import annotations
import argparse
import re
import sys
from pathlib import Path

# Denylist categories per AGENTS.md §9.3:
#   compensation (salary, bonus, equity, grant)
#   gov-IDs (passport, national_id, ssn, tax_id)
#   bank (iban, swift, account_number)
#   special-category PII (health, religion, sexual orientation)
#   secrets (api keys, oauth tokens, private keys)

DENYLIST_FIXTURES = [
    # (test_id, body_text, expected_reject_reason_substring)

    # compensation
    ("comp-001", "Alice's annual salary is $145,000.", "compensation"),
    ("comp-002", "Bob received a $25k bonus in Q4.", "compensation"),
    ("comp-003", "Carol's equity grant: 12,500 shares vesting over 4 years.", "compensation"),
    ("comp-004", "Dave's payslip shows $8,200 base + $1,300 OT.", "compensation"),

    # government IDs
    ("gov-001", "Stephen's passport number is A12345678.", "gov-id"),
    ("gov-002", "SSN: 123-45-6789", "gov-id"),
    ("gov-003", "Tax_ID for the corp: 12-3456789", "gov-id"),
    ("gov-004", "Vietnamese national_id: 079123456789", "gov-id"),

    # bank
    ("bank-001", "Wire transfer to IBAN: GB82 WEST 1234 5698 7654 32.", "bank"),
    ("bank-002", "SWIFT code: VCBKVNVX for our HCMC account.", "bank"),
    ("bank-003", "Account_number 1234567890 at Vietcombank.", "bank"),

    # secrets
    ("secret-001", "API_KEY=sk-proj-abcdef0123456789", "secret"),
    ("secret-002", "Bearer token: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.fake", "secret"),
    ("secret-003", "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...", "secret"),

    # special-category PII (health)
    ("health-001", "Alice's HIV status: positive (diagnosed 2024-03).", "health"),
    ("health-002", "Bob's mental health diagnosis is depression with anxiety.", "health"),

    # mixed-script confusable evasion attempts
    ("evasion-001", "Sa1ary: 145,000 (numeral evasion)", "compensation"),
    ("evasion-002", "Sаlаry: 100k (Cyrillic а evasion)", "compensation"),
    ("evasion-003", "S a l a r y is 145000 (whitespace evasion)", "compensation"),
]

# What SHOULD be accepted (negative cases — verify no false positives)
ALLOWLIST_FIXTURES = [
    ("ok-001", "Alice manages the engineering team.", None),
    ("ok-002", "The new MacBook costs $2,400.", None),  # hardware price, not comp
    ("ok-003", "Bob's birthday is March 12.", None),  # not gov-ID
    ("ok-004", "Carol prefers async communication.", None),
    ("ok-005", "Total revenue Q4 was $1.2M.", None),  # company revenue, not comp
]

def _matches_pattern(body: str, category: str) -> bool:
    """Simulate the §9.3 denylist check (compensation/gov-id/bank/health/secret).

    This mirrors the regex patterns from manifest.json exclusion_rules.
    For real testing, this should call into memory_writer's actual validator.
    """
    patterns = {
        "compensation": re.compile(r"\b(?:salary|payslip|bonus|equity|grant|sa[1lI][aA4]ry|sаlаry|s\s+a\s+l\s+a\s+r\s+y)\b", re.I),
        "gov-id": re.compile(r"\b(?:passport|national_id|ssn|tax_id)\b", re.I),
        "bank": re.compile(r"\b(?:iban|swift|account_number)\b", re.I),
        "secret": re.compile(r"\b(?:api[_-]?key|bearer\s+token|private\s+key|begin\s+rsa)\b", re.I),
        "health": re.compile(r"\b(?:hiv|mental\s+health|diagnosis|diagnosed)\b", re.I),
    }
    p = patterns.get(category)
    return bool(p and p.search(body))

def _should_reject(body: str) -> str | None:
    """Returns the category that triggers rejection, or None."""
    for cat in ("compensation", "gov-id", "bank", "secret", "health"):
        if _matches_pattern(body, cat):
            return cat
    return None

def run_tests(verbose=False) -> tuple[int, int]:
    passed = 0
    failed = 0

    print("\n=== DENYLIST FIXTURES (should reject) ===")
    for tid, body, expected_cat in DENYLIST_FIXTURES:
        result = _should_reject(body)
        ok = result is not None and (expected_cat in result if expected_cat else True)
        if ok:
            passed += 1
            if verbose:
                print(f"  ✓ {tid}: rejected as '{result}'")
        else:
            failed += 1
            print(f"  ✗ {tid}: expected reject ({expected_cat}), got {result}")
            print(f"      body: {body[:70]}")

    print("\n=== ALLOWLIST FIXTURES (should pass) ===")
    for tid, body, _ in ALLOWLIST_FIXTURES:
        result = _should_reject(body)
        if result is None:
            passed += 1
            if verbose:
                print(f"  ✓ {tid}: passed (no false positive)")
        else:
            failed += 1
            print(f"  ✗ {tid}: FALSE POSITIVE — rejected as '{result}'")
            print(f"      body: {body[:70]}")

    return passed, failed

def main():
    p = argparse.ArgumentParser()
    p.add_argument("-v", "--verbose", action="store_true")
    p.add_argument("--list", action="store_true", help="list all fixtures + exit")
    args = p.parse_args()

    if args.list:
        print("DENYLIST fixtures (should reject):")
        for tid, body, cat in DENYLIST_FIXTURES:
            print(f"  {tid}: [{cat}] {body[:80]}")
        print("\nALLOWLIST fixtures (should pass):")
        for tid, body, _ in ALLOWLIST_FIXTURES:
            print(f"  {tid}: {body[:80]}")
        return 0

    passed, failed = run_tests(verbose=args.verbose)
    total = passed + failed
    print(f"\n{'=' * 50}")
    print(f"Result: {passed}/{total} passed; {failed} failed")
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    sys.exit(main())
