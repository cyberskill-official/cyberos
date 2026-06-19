"""git worktree-per-task manager for isolated parallel work.

Each task gets its own branch + working directory sharing one ``.git`` object
store. Git structurally refuses to check out the same branch in two worktrees — a
built-in collision guard we surface explicitly. Failure modes the manager guards
against (Augment / GitButler / Cursor field reports):

  * self-inflicted merge conflicts when agents touch the same files
    -> merges are serialized through the integration branch, never agent-to-agent;
       a conflicting merge is aborted cleanly and reported, never left half-applied.
  * disk bloat (one report: 9.82 GB in 20 min) -> ``disk_usage`` + ``prune``.
  * shared .env/port/DB state -> ``write_isolation`` gives each worktree its own
    .env.local with a unique port.

All operations shell out to real git (injectable ``runner`` for the few pure
bits); the suite exercises them against a real temporary repository.
"""

from __future__ import annotations

import os
import re
import subprocess
import zlib
from dataclasses import dataclass
from pathlib import Path

__all__ = ["Worktree", "WorktreeError", "MergeResult", "WorktreeManager"]


class WorktreeError(RuntimeError):
    """A git worktree operation failed (message carries git's stderr)."""


# A task id becomes BOTH a filesystem path segment and a git branch/ref name, so it
# must be restricted: no '/' or '..' (path traversal out of the worktrees dir), no
# leading '-' (git argument injection like `-D`), no shell/ref metacharacters.
_TASK_ID_RE = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]*$")


def _check_task_id(task_id: str) -> str:
    if not isinstance(task_id, str) or ".." in task_id or not _TASK_ID_RE.match(task_id):
        raise WorktreeError(
            f"invalid task_id {task_id!r}: allowed chars [A-Za-z0-9._-], must start "
            "alphanumeric, no '..', no '/', no leading '-'"
        )
    return task_id


@dataclass(frozen=True)
class Worktree:
    task_id: str
    path: Path
    branch: str
    head: str = ""


@dataclass(frozen=True)
class MergeResult:
    ok: bool
    branch: str
    conflicts: list[str]
    message: str

    def __bool__(self) -> bool:
        return self.ok


class WorktreeManager:
    BRANCH_PREFIX = "auto/wt/"

    def __init__(
        self,
        repo_root: str | os.PathLike[str],
        worktrees_dir: str | os.PathLike[str] | None = None,
    ):
        self.repo_root = Path(repo_root).resolve()
        if not (self.repo_root / ".git").exists():
            raise WorktreeError(f"{self.repo_root} is not a git repository")
        # default: a sibling dir so worktrees never pollute the repo's own status
        self.worktrees_dir = (
            Path(worktrees_dir).resolve()
            if worktrees_dir
            else self.repo_root.parent / f".awh-worktrees-{self.repo_root.name}"
        )

    # ----------------------------- git plumbing ----------------------------- #
    def _git(self, *args: str, cwd: Path | None = None, check: bool = True, timeout: float = 120.0):
        try:
            proc = subprocess.run(
                ["git", *args],
                cwd=str(cwd or self.repo_root),
                capture_output=True,
                text=True,
                timeout=timeout,
            )
        except subprocess.TimeoutExpired as e:
            raise WorktreeError(f"git {' '.join(args)} timed out after {timeout}s") from e
        if check and proc.returncode != 0:
            raise WorktreeError(f"git {' '.join(args)} failed: {proc.stderr.strip()}")
        return proc

    def _branch_exists(self, branch: str) -> bool:
        return (
            self._git(
                "rev-parse", "--verify", "--quiet", f"refs/heads/{branch}", check=False
            ).returncode
            == 0
        )

    def _current_ref(self) -> str:
        """Current branch name, or the commit sha if HEAD is detached."""
        p = self._git("symbolic-ref", "--quiet", "--short", "HEAD", check=False)
        if p.returncode == 0 and p.stdout.strip():
            return p.stdout.strip()
        return self._git("rev-parse", "HEAD", check=False).stdout.strip()

    def branch_for(self, task_id: str) -> str:
        return f"{self.BRANCH_PREFIX}{_check_task_id(task_id)}"

    # ------------------------------- commands ------------------------------- #
    def create(self, task_id: str, base_ref: str = "HEAD") -> Worktree:
        """Create an isolated worktree + branch for ``task_id``.

        Raises WorktreeError if the branch is already checked out in another
        worktree (git's structural collision guard).
        """
        branch = self.branch_for(task_id)
        path = self.worktrees_dir / task_id
        self.worktrees_dir.mkdir(parents=True, exist_ok=True)
        if path.exists():
            raise WorktreeError(f"worktree path already exists: {path}")
        if self._branch_exists(branch):
            # attach existing branch (will fail loudly if checked out elsewhere)
            self._git("worktree", "add", str(path), branch)
        else:
            self._git("worktree", "add", "-b", branch, str(path), base_ref)
        head = self._git("rev-parse", "HEAD", cwd=path).stdout.strip()
        return Worktree(task_id=task_id, path=path, branch=branch, head=head)

    def list(self) -> list[Worktree]:
        """List worktrees managed under our worktrees dir."""
        out = self._git("worktree", "list", "--porcelain").stdout
        items: list[Worktree] = []
        cur: dict[str, str] = {}

        def flush():
            if not cur:
                return
            p = Path(cur.get("worktree", ""))
            try:
                p.resolve().relative_to(self.worktrees_dir)
            except ValueError:
                return  # not one of ours (e.g. the main worktree)
            branch = cur.get("branch", "").replace("refs/heads/", "")
            items.append(Worktree(task_id=p.name, path=p, branch=branch, head=cur.get("HEAD", "")))

        for line in out.splitlines():
            if not line.strip():
                flush()
                cur = {}
                continue
            key, _, val = line.partition(" ")
            cur[key] = val
        flush()
        return items

    def remove(self, task_id: str, force: bool = False, delete_branch: bool = False) -> None:
        _check_task_id(task_id)
        path = self.worktrees_dir / task_id
        args = ["worktree", "remove"]
        if force:
            args.append("--force")
        args.append(str(path))
        self._git(*args)
        if delete_branch:
            self._git("branch", "-D", self.branch_for(task_id), check=False)

    def merge(self, task_id: str, into: str, message: str | None = None) -> MergeResult:
        """Serialized merge of a task branch INTO the integration branch.

        Runs in the main repo (the integration layer). On conflict the merge is
        aborted cleanly and the conflicting files are reported — never left
        half-applied for the next task to trip over.
        """
        branch = self.branch_for(task_id)
        if not self._branch_exists(branch):
            return MergeResult(False, branch, [], f"merge failed: branch {branch} does not exist")
        message = message or f"merge {branch} into {into}"
        prior = self._current_ref()
        self._git("checkout", into)
        try:
            proc = self._git("merge", "--no-ff", "-m", message, branch, check=False)
            if proc.returncode == 0:
                return MergeResult(True, branch, [], f"merged {branch} into {into}")
            conflicts = [
                ln
                for ln in self._git(
                    "diff", "--name-only", "--diff-filter=U", check=False
                ).stdout.splitlines()
                if ln
            ]
            self._git("merge", "--abort", check=False)
            return MergeResult(False, branch, conflicts, f"CONFLICT merging {branch}: {conflicts}")
        finally:
            # never leave the main repo parked on the integration branch
            if prior and prior != into:
                self._git("checkout", prior, check=False)

    def prune(self) -> None:
        """Prune stale worktree admin files (after manual dir deletion / disk reclaim)."""
        self._git("worktree", "prune")

    def disk_usage(self) -> int:
        """Total bytes used by the worktrees dir (disk-bloat awareness)."""
        total = 0
        if not self.worktrees_dir.exists():
            return 0
        for dirpath, _dirs, files in os.walk(self.worktrees_dir):
            for f in files:
                fp = Path(dirpath) / f
                if fp.is_file() and not fp.is_symlink():
                    total += fp.stat().st_size
        return total

    def write_isolation(self, task_id: str, base_port: int = 4000) -> dict:
        """Give a worktree its own .env.local with a unique port (avoid shared state).

        Port is derived deterministically from the task id so parallel worktrees
        don't collide on a shared port/DB.
        """
        _check_task_id(task_id)
        path = self.worktrees_dir / task_id
        if not path.exists():
            raise WorktreeError(f"no worktree for task {task_id!r}")
        # crc32 (not builtin hash(), which is salted per-process) -> stable port.
        # Wide modulus so deterministic ports rarely collide even at high fan-out
        # (1000 slots collided by ~40 parallel tasks; 20000 pushes that out to ~170).
        port = base_port + (zlib.crc32(task_id.encode()) % 20000)
        env = {"PORT": str(port), "AWH_WORKTREE": task_id}
        (path / ".env.local").write_text("".join(f"{k}={v}\n" for k, v in env.items()))
        return env
