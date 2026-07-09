#!/usr/bin/env python3
"""REF-NNN regression eval — template."""
import subprocess
from pathlib import Path

def find_root():
    cur = Path.cwd().resolve()
    while cur != cur.parent:
        if (cur / ".cyberos/memory/store").is_dir():
            return cur
        cur = cur.parent
    raise RuntimeError("no .cyberos/memory/store")

def test_regression():
    """Pre-REF memories still validate after REF lands."""
    root = find_root()
    validator = root / "runtime" / "tools" / "cyberos_validate.py"
    r = subprocess.run(["python3", str(validator), str(root)],
                       capture_output=True, text=True)
    # Expect zero CRITICAL in output
    critical_count = r.stdout.count("CRITICAL")
    assert critical_count == 0, f"validator returned {critical_count} CRITICAL findings"

if __name__ == "__main__":
    test_regression()
    print("PASS — regression eval")
