"""FR-AI-012 §5 — AST-walk lint test.

§1 #12 mandates no network calls in recognizers.
This test catches forbidden imports at PR time.
"""

import ast
from pathlib import Path

FORBIDDEN_IMPORTS = {
    "requests", "urllib", "urllib2", "urllib3", "httpx", "aiohttp",
    "http", "socket", "ssl", "ftplib", "smtplib",
}


def test_no_network_imports_in_recognizers():
    """§1 #12: recognizer modules MUST be pure regex + lookup tables."""
    recognizers_dir = Path(__file__).parent / "recognizers"
    failures = []
    for py_file in recognizers_dir.glob("*.py"):
        if py_file.name == "__pycache__":
            continue
        tree = ast.parse(py_file.read_text())
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    top = alias.name.split(".")[0]
                    if top in FORBIDDEN_IMPORTS:
                        failures.append(
                            f"{py_file.name}: forbidden import: {alias.name}"
                        )
            elif isinstance(node, ast.ImportFrom):
                top = (node.module or "").split(".")[0]
                if top in FORBIDDEN_IMPORTS:
                    failures.append(
                        f"{py_file.name}: forbidden import-from: {node.module}"
                    )
    assert not failures, "Network imports detected in recognizers:\n" + "\n".join(
        failures
    )
