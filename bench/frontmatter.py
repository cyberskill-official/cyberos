"""
bench/frontmatter.py — msgspec vs PyYAML parse benchmark.

Hypothesis (audit report §5):

    msgspec is ≥ 15× faster than PyYAML at p50
    msgspec is ≥ 10× faster than PyYAML at p99

Run:

    python -m bench.frontmatter --parser pyyaml
    python -m bench.frontmatter --parser msgspec
    python -m bench.frontmatter --compare        # runs both, asserts targets

Inputs: 10,000 synthetic memory files, frontmatter sizes 200B–2KB.
Output: JSON to stdout for CI consumption.
"""

from __future__ import annotations

import argparse
import gc
import json
import random
import secrets
import statistics
import sys
import time
from pathlib import Path

_DEFAULT_N: int = 10_000


def gen_corpus(out: Path, n: int) -> None:
    out.mkdir(parents=True, exist_ok=True)
    rng = random.Random(0xC0DE)
    for i in range(n):
        fm = {
            "id": secrets.token_hex(8),
            "kind": rng.choice(
                ["decision", "fact", "person", "project", "preference"],
            ),
            "ts_ns": time.time_ns(),
            "actor": rng.choice(["stephen", "coding-agent", "ops-agent"]),
            "tags": [secrets.token_hex(4) for _ in range(rng.randint(0, 6))],
            "extra": {"v": rng.randint(0, 1 << 32)},
        }
        body = b"# " + secrets.token_hex(16).encode() + b"\n\n" + b"x" * rng.randint(200, 2000)
        (out / f"{i:06d}.md").write_bytes(
            b"---\n" + json.dumps(fm).encode("utf-8") + b"\n---\n" + body
        )


def bench_pyyaml(files: list[bytes]) -> list[float]:
    import re
    import yaml

    pattern = re.compile(rb"^---\n(.*?)\n---\n", re.DOTALL)
    out: list[float] = []
    for raw in files:
        gc.disable()
        t0 = time.perf_counter_ns()
        match = pattern.match(raw)
        if match is None:
            gc.enable()
            continue
        yaml.safe_load(match.group(1).decode("utf-8"))
        _ = raw[match.end():]
        out.append((time.perf_counter_ns() - t0) / 1000.0)
        gc.enable()
    return out


def bench_msgspec(files: list[bytes]) -> list[float]:
    from cyberos.core.frontmatter import parse

    out: list[float] = []
    for raw in files:
        gc.disable()
        t0 = time.perf_counter_ns()
        parse(raw)
        out.append((time.perf_counter_ns() - t0) / 1000.0)
        gc.enable()
    return out


def pct(xs: list[float], p: int) -> float:
    if not xs:
        return 0.0
    quantiles = statistics.quantiles(xs, n=100, method="inclusive")
    return quantiles[p - 1]


def summary(parser: str, times: list[float]) -> dict:
    return {
        "parser": parser,
        "n": len(times),
        "mean_us": statistics.mean(times) if times else 0.0,
        "p50_us": pct(times, 50),
        "p95_us": pct(times, 95),
        "p99_us": pct(times, 99),
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--parser", choices=["pyyaml", "msgspec"])
    ap.add_argument("--files", type=int, default=_DEFAULT_N)
    ap.add_argument("--corpus", default="/tmp/cyberos-fm-corpus")
    ap.add_argument("--compare", action="store_true", help="run both and assert targets")
    args = ap.parse_args(argv)

    corpus = Path(args.corpus)
    if not corpus.exists() or len(list(corpus.glob("*.md"))) < args.files:
        gen_corpus(corpus, args.files)
    files = [p.read_bytes() for p in sorted(corpus.glob("*.md"))[: args.files]]

    if args.compare:
        r_yaml = summary("pyyaml", bench_pyyaml(files))
        r_msg = summary("msgspec", bench_msgspec(files))
        out = {"pyyaml": r_yaml, "msgspec": r_msg}
        out["speedup_p50"] = (r_yaml["p50_us"] / r_msg["p50_us"]) if r_msg["p50_us"] else 0
        out["speedup_p99"] = (r_yaml["p99_us"] / r_msg["p99_us"]) if r_msg["p99_us"] else 0
        print(json.dumps(out, indent=2))
        ok = out["speedup_p50"] >= 15.0 and out["speedup_p99"] >= 10.0
        return 0 if ok else 1

    if not args.parser:
        ap.print_help()
        return 2
    times = bench_pyyaml(files) if args.parser == "pyyaml" else bench_msgspec(files)
    print(json.dumps(summary(args.parser, times), indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
