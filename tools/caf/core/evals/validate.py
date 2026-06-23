#!/usr/bin/env python3
"""Compatibility shim — the implementation lives in code_audit_validator.py
(named so the PyPI wheel doesn't claim the generic top-level module name
`validate`). Every documented invocation works unchanged through this file:
    python3 core/evals/validate.py --all | --run <dir> [--report json|sarif]
"""
from code_audit_validator import *  # noqa: F401,F403
from code_audit_validator import main, validate_run, parse_tables, col, norm  # noqa: F401

if __name__ == "__main__":
    main()
