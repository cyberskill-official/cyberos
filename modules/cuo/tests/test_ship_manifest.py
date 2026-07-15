"""TASK-CUO-206 acceptance tests - ship-manifest@1 (one test per AC)."""
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import unittest

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "modules", "cuo"))
from cuo import ship_manifest as sm  # noqa: E402

WORKFLOW_DOC = os.path.join(ROOT, "modules", "cuo", "chief-technology-officer",
                            "workflows", "ship-tasks.md")
CONTRACT_DOC = os.path.join(ROOT, "modules", "skill", "contracts",
                            "task", "SHIP-MANIFEST.md")
TASK_DOC = os.path.join(ROOT, "docs", "tasks", "cuo",
                      "TASK-CUO-206-ship-run-state-manifest.md")


def _manifest(steps=None, wf="2.4.0", frsha="a" * 64):
    return {
        "manifest_version": "ship-manifest@1", "task_id": "TASK-TEN-208",
        "task_sha256": frsha, "workflow_version": wf,
        "started_at": "2026-07-12T10:00:00+07:00",
        "updated_at": "2026-07-12T11:42:10+07:00",
        "current_step": 11, "routed_back_count": 0,
        "steps": steps or [], "hitl": {"gate": None, "requested_at": None},
    }


def _done_steps(n, art_dir):
    steps = []
    for i in range(1, n + 1):
        p = os.path.join(art_dir, f"step{i}.md")
        with open(p, "w") as f:
            f.write(f"artefact {i}\n")
        steps.append({"index": i, "skill": f"skill-{i}", "status": "done",
                      "artefact_path": p,
                      "artefact_sha256": hashlib.sha256(open(p, "rb").read()).hexdigest(),
                      "verdict": "pass", "completed_at": "2026-07-12T10:12:00+07:00"})
    return steps


def _hash_of(path):
    try:
        with open(path, "rb") as f:
            return hashlib.sha256(f.read()).hexdigest()
    except (TypeError, OSError):
        return None


class TestShipManifest(unittest.TestCase):

    def test_schema_fields_and_example_validate(self):  # AC 1
        contract = open(CONTRACT_DOC).read()
        for field in ("manifest_version", "task_id", "task_sha256", "workflow_version",
                      "started_at", "updated_at", "current_step", "routed_back_count",
                      "steps", "hitl", "artefact_sha256", "skipped-conditional",
                      "review_approval", "final_acceptance"):
            self.assertIn(field, contract, f"SHIP-MANIFEST.md missing field {field}")
        self.assertEqual(sm.validate(_manifest()), [])
        bad = _manifest()
        del bad["task_sha256"]
        self.assertTrue(any("task_sha256" in e for e in sm.validate(bad)))
        bad2 = _manifest(steps=[{"index": 99, "skill": "x", "status": "wat"}])
        self.assertEqual(len(sm.validate(bad2)), 2)  # bad status + bad index
        # every root error branch fires exactly once
        wrong = _manifest()
        wrong.update({"manifest_version": "ship-manifest@0", "task_sha256": "zz",
                      "current_step": 0, "routed_back_count": -1})
        wrong["hitl"] = {"gate": "vibes"}
        self.assertEqual(len(sm.validate(wrong)), 5)

    def test_atomic_write_discipline_documented(self):  # AC 2
        doc = open(WORKFLOW_DOC).read()
        self.assertIn("after EVERY completed, failed, or conditionally-skipped", doc)
        self.assertIn(".tmp.<nonce>", doc)
        with tempfile.TemporaryDirectory() as d:
            p = os.path.join(d, "TASK-X.ship.json")
            sm.write_atomic(_manifest(), p)
            self.assertEqual(json.load(open(p))["task_id"], "TASK-TEN-208")
            self.assertEqual([f for f in os.listdir(d) if ".tmp." in f], [])
            # failure path: replace blows up -> tmp file is still cleaned
            real = os.replace
            os.replace = lambda a, b: (_ for _ in ()).throw(OSError("boom"))
            try:
                with self.assertRaises(OSError):
                    sm.write_atomic(_manifest(), os.path.join(d, "x.ship.json"))
            finally:
                os.replace = real
            self.assertEqual([f for f in os.listdir(d) if ".tmp." in f], [])

    def test_resume_plan_intact_and_stale(self):  # AC 3
        with tempfile.TemporaryDirectory() as d:
            m = _manifest(steps=_done_steps(10, d))
            plan = sm.resume_plan(m, "2.4.0", "a" * 64, _hash_of)
            self.assertEqual((plan["action"], plan["start_step"], plan["stale_from"]),
                             ("resume", 11, None))
            with open(m["steps"][4]["artefact_path"], "w") as f:
                f.write("corrupted\n")
            plan2 = sm.resume_plan(m, "2.4.0", "a" * 64, _hash_of)
            self.assertEqual((plan2["action"], plan2["start_step"], plan2["stale_from"]),
                             ("resume", 5, 5))

    def test_workflow_version_mismatch_needs_human(self):  # AC 4
        plan = sm.resume_plan(_manifest(wf="2.3.0"), "2.4.0", "a" * 64, _hash_of)
        self.assertEqual(plan["action"], "needs_human")
        # fr_sha256 mismatch: everything stale, fresh start (not needs_human)
        plan2 = sm.resume_plan(_manifest(), "2.4.0", "b" * 64, _hash_of)
        self.assertEqual((plan2["action"], plan2["start_step"], plan2["stale_from"]),
                         ("resume", 1, 1))

    def test_queue_selection_total_order(self):  # AC 5
        tasks = [
            {"id": "TASK-A-002", "status": "ready_to_implement", "priority": "SHOULD",
             "created": "2026-07-01", "depends_on": []},
            {"id": "TASK-A-003", "status": "ready_to_implement", "priority": "MUST",
             "created": "2026-07-02", "depends_on": ["TASK-A-009"]},  # unmet dep
            {"id": "TASK-B-001", "status": "ready_to_implement", "priority": "MUST",
             "created": "2026-07-02", "depends_on": ["TASK-A-001"]},
            {"id": "TASK-A-004", "status": "ready_to_implement", "priority": "MUST",
             "created": "2026-07-02", "depends_on": []},  # ties TASK-B-001 -> id asc wins
            {"id": "TASK-A-001", "status": "done", "priority": "MUST",
             "created": "2026-06-01", "depends_on": []},
            {"id": "TASK-A-005", "status": "draft", "priority": "MUST",
             "created": "2026-06-01", "depends_on": []},
        ]
        r1 = sm.select_next(tasks)
        self.assertEqual(r1["picked"], "TASK-A-004")
        self.assertEqual(r1["reason"],
                         "queue: picked TASK-A-004 (priority=MUST, created=2026-07-02) "
                         "over 2 other eligible tasks")
        self.assertEqual(sm.select_next(list(reversed(tasks))), r1)  # deterministic
        self.assertIsNone(sm.select_next([f for f in tasks if f["status"] == "draft"])["picked"])

    def test_gitignore_scaffold(self):  # AC 6
        gi = os.path.join(ROOT, "docs", "tasks", ".workflow", ".gitignore")
        self.assertEqual(open(gi).read().strip(), "*.ship.json")
        # PRE-EXISTING failure, unrelated to the fr->task rename: init.sh was
        # deleted in bb0f2392e ("1.0.0 CLI surface") and replaced by install.sh.
        # The test kept opening the dead path and has been red ever since.
        init_sh = open(os.path.join(ROOT, "tools", "cyberos-init", "install.sh")).read()
        self.assertIn(".workflow/.gitignore", init_sh)
        self.assertIn("*.ship.json", init_sh)
        out = subprocess.run(["git", "check-ignore",
                              "docs/tasks/.workflow/TASK-X.ship.json"],
                             cwd=ROOT, capture_output=True)
        self.assertEqual(out.returncode, 0, "manifest path is not gitignored")

    def test_done_deletes_routeback_keeps(self):  # AC 7
        m = _manifest()
        self.assertEqual(sm.finalize(m, "done"), {"action": "delete_manifest"})
        kept = sm.finalize(m, "route_back")
        self.assertEqual(kept["action"], "keep_manifest")
        self.assertEqual(kept["manifest"]["routed_back_count"], 1)
        self.assertEqual(m["routed_back_count"], 0)  # input not mutated
        with self.assertRaises(ValueError):
            sm.finalize(m, "explode")

    def test_hitl_reask_on_resume(self):  # AC 8
        with tempfile.TemporaryDirectory() as d:
            m = _manifest(steps=_done_steps(18, d))
            m["hitl"] = {"gate": "review_approval",
                         "requested_at": "2026-07-12T11:00:00+07:00"}
            plan = sm.resume_plan(m, "2.4.0", "a" * 64, _hash_of)
            self.assertEqual(plan["start_step"], 19)
            self.assertEqual(plan["gate_pending"], "review_approval")
            self.assertIn("re-requested", plan["reason"])
        doc = open(WORKFLOW_DOC).read()
        self.assertIn("NOT be treated as approval", doc)


if __name__ == "__main__":
    unittest.main(verbosity=2)
