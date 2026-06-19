"""Statistical core for trustworthy agent evaluation.

Why this module exists
----------------------
"On Randomness in Agentic Evals" (Bjarnason, Silva, Monperrus; KTH; arXiv 2602.07150)
collected 60,000 trajectories on SWE-bench Verified and found single-run pass@1
varies by 2.2-6.0 percentage points depending on which run you pick, with a std
dev > 1.5 points *even at temperature 0*. Their recommendations, implemented here:

  1. estimate pass@1 from multiple independent runs            -> pass_at_k / mean
  2. size the number of runs with statistical power analysis   -> seeds_for_power
  3. report optimistic AND pessimistic bounds                  -> pass_at_k / pass_hat_k

`pass_at_k` is the unbiased estimator from the Codex/HumanEval paper
(Chen et al., 2021). `pass_hat_k` is the consistency metric used by tau-bench
(Yao et al., arXiv:2406.12045): the probability that k independently sampled
trajectories *all* pass.

Pure standard library — no numpy/scipy — so it runs anywhere the agent runs.
"""

from __future__ import annotations

import math
from dataclasses import dataclass

__all__ = [
    "pass_at_k",
    "pass_hat_k",
    "wilson_interval",
    "two_proportion_power",
    "seeds_for_power",
    "PowerResult",
    "inv_norm_cdf",
]


def pass_at_k(n: int, c: int, k: int) -> float:
    """Unbiased estimate of pass@k (optimistic: at least one of k samples passes).

    Codex/HumanEval estimator: 1 - C(n-c, k) / C(n, k), evaluated in the
    numerically stable product form to avoid huge binomials.

    Args:
        n: total samples drawn for the task.
        c: number of correct samples among them.
        k: the k in pass@k (1 <= k <= n).

    Returns:
        Probability in [0, 1] that at least one of k uniformly-without-replacement
        sampled trajectories is correct.
    """
    if k < 1:
        raise ValueError("k must be >= 1")
    if n < 1:
        raise ValueError("n must be >= 1")
    if not 0 <= c <= n:
        raise ValueError("require 0 <= c <= n")
    if k > n:
        raise ValueError("k must be <= n")
    if n - c < k:
        # Fewer than k incorrect samples -> every k-subset contains a correct one.
        return 1.0
    # Product form of 1 - C(n-c, k)/C(n, k).
    prob_all_fail = 1.0
    for i in range(k):
        prob_all_fail *= (n - c - i) / (n - i)
    return 1.0 - prob_all_fail


def pass_hat_k(n: int, c: int, k: int) -> float:
    """Unbiased estimate of pass^k (pessimistic: all k samples pass) -- consistency.

    Estimator: C(c, k) / C(n, k), the probability that a uniformly chosen
    k-subset of the n trajectories is entirely correct. This is the tau-bench
    "pass^k" reliability metric, which drops far faster than pass@k and exposes
    flakiness that a single run hides.

    Args:
        n: total samples drawn for the task.
        c: number of correct samples among them.
        k: the k in pass^k (1 <= k <= n).
    """
    if k < 1:
        raise ValueError("k must be >= 1")
    if n < 1:
        raise ValueError("n must be >= 1")
    if not 0 <= c <= n:
        raise ValueError("require 0 <= c <= n")
    if k > n:
        raise ValueError("k must be <= n")
    if c < k:
        return 0.0
    return math.comb(c, k) / math.comb(n, k)


@dataclass(frozen=True)
class Interval:
    """A closed confidence interval [low, high] around a point estimate."""

    point: float
    low: float
    high: float

    def __iter__(self):  # allow tuple-unpacking: low, high = ...
        yield self.low
        yield self.high


def wilson_interval(successes: int, n: int, z: float = 1.959963984540054) -> Interval:
    """Wilson score confidence interval for a binomial proportion.

    Preferred over the normal (Wald) interval because it behaves well for small
    n and proportions near 0/1 -- exactly the regime of a ~20-50 task golden set.

    Args:
        successes: number of passing runs.
        n: total runs.
        z: standard-normal critical value (default 1.96 ~ 95% two-sided).

    Returns:
        Interval(point=p_hat, low, high), all in [0, 1].
    """
    if n <= 0:
        raise ValueError("n must be > 0")
    if not 0 <= successes <= n:
        raise ValueError("require 0 <= successes <= n")
    if not (math.isfinite(z) and z > 0):
        raise ValueError("z must be a finite, positive critical value")
    p = successes / n
    z2 = z * z
    denom = 1.0 + z2 / n
    center = (p + z2 / (2 * n)) / denom
    margin = (z * math.sqrt((p * (1 - p) + z2 / (4 * n)) / n)) / denom
    low = max(0.0, center - margin)
    high = min(1.0, center + margin)
    # p_hat is in the Wilson interval by construction (the score statistic is 0 at p);
    # enforce that against float error so low <= point <= high always holds.
    return Interval(point=p, low=min(low, p), high=max(high, p))


# --------------------------------------------------------------------------- #
# Inverse normal CDF (Acklam's rational approximation) -- no scipy dependency.
# --------------------------------------------------------------------------- #
def inv_norm_cdf(p: float) -> float:
    """Inverse standard-normal CDF (quantile function) via Acklam's algorithm.

    Absolute error < 1.15e-9 across (0, 1). Used to turn alpha/power into z-values
    for the power analysis so callers can pass arbitrary confidence levels.
    """
    if not 0.0 < p < 1.0:
        raise ValueError("p must be in (0, 1)")
    a = [
        -3.969683028665376e01,
        2.209460984245205e02,
        -2.759285104469687e02,
        1.383577518672690e02,
        -3.066479806614716e01,
        2.506628277459239e00,
    ]
    b = [
        -5.447609879822406e01,
        1.615858368580409e02,
        -1.556989798598866e02,
        6.680131188771972e01,
        -1.328068155288572e01,
    ]
    c = [
        -7.784894002430293e-03,
        -3.223964580411365e-01,
        -2.400758277161838e00,
        -2.549732539343734e00,
        4.374664141464968e00,
        2.938163982698783e00,
    ]
    d = [7.784695709041462e-03, 3.224671290700398e-01, 2.445134137142996e00, 3.754408661907416e00]
    plow, phigh = 0.02425, 1 - 0.02425
    if p < plow:
        q = math.sqrt(-2 * math.log(p))
        return (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5]) / (
            (((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1
        )
    if p > phigh:
        q = math.sqrt(-2 * math.log(1 - p))
        return -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5]) / (
            (((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1
        )
    q = p - 0.5
    r = q * q
    return (
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5])
        * q
        / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1)
    )


@dataclass(frozen=True)
class PowerResult:
    """Outcome of a seeds-for-power calculation."""

    seeds_per_arm: int
    baseline: float
    mde: float
    alpha: float
    power: float

    def summary(self) -> str:
        return (
            f"To detect a {self.mde:+.1%} change from a {self.baseline:.1%} baseline "
            f"at alpha={self.alpha} (two-sided) with power={self.power:.0%}, "
            f"run >= {self.seeds_per_arm} seeds per variant."
        )


def two_proportion_power(baseline: float, mde: float, alpha: float, power: float) -> int:
    """Seeds per arm for a two-proportion z-test (normal approximation).

    n = (z_alpha * sqrt(2*pbar*(1-pbar)) + z_beta * sqrt(p1(1-p1)+p2(1-p2)))^2 / delta^2

    Args:
        baseline: current pass rate p1 in (0, 1).
        mde: minimum detectable effect (signed); p2 = baseline + mde, clamped to (0,1).
        alpha: two-sided significance level (e.g. 0.05).
        power: desired power (e.g. 0.80).

    Returns:
        ceil(n) seeds per arm (>= 1).
    """
    if not 0.0 < baseline < 1.0:
        raise ValueError("baseline must be in (0, 1)")
    if mde == 0:
        raise ValueError("mde must be non-zero")
    if not 0.0 < alpha < 1.0:
        raise ValueError("alpha must be in (0, 1)")
    if not 0.0 < power < 1.0:
        raise ValueError("power must be in (0, 1)")
    p1 = baseline
    p2 = min(1.0 - 1e-9, max(1e-9, baseline + mde))
    delta = abs(p2 - p1)
    if delta == 0:
        raise ValueError("baseline + mde leaves no detectable effect after clamping to (0, 1)")
    pbar = (p1 + p2) / 2
    z_alpha = inv_norm_cdf(1 - alpha / 2)
    z_beta = inv_norm_cdf(power)
    n = (
        z_alpha * math.sqrt(2 * pbar * (1 - pbar))
        + z_beta * math.sqrt(p1 * (1 - p1) + p2 * (1 - p2))
    ) ** 2 / (delta**2)
    return max(1, math.ceil(n))


def seeds_for_power(
    baseline: float, mde: float, alpha: float = 0.05, power: float = 0.80
) -> PowerResult:
    """Convenience wrapper returning a documented PowerResult.

    The KTH study found run-to-run std dev > 1.5 points even at temperature 0;
    use this to refuse to trust an improvement smaller than your eval can resolve.
    """
    seeds = two_proportion_power(baseline, mde, alpha, power)
    return PowerResult(seeds_per_arm=seeds, baseline=baseline, mde=mde, alpha=alpha, power=power)
