"""Release verification consumes committed campaigns without network or builds."""
import copy
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
SPEC = importlib.util.spec_from_file_location("evidence", ROOT / "scripts/benchmark/check_release_evidence.py")
evidence = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(evidence)


class ReleaseEvidenceTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        shutil.copytree(ROOT / "scripts/benchmark", self.root / "scripts/benchmark", ignore=shutil.ignore_patterns("__pycache__"))
        self.git("init", "-q")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "Evidence test")
        (self.root / ".gitignore").write_text("__pycache__/\n")
        (self.root / "source.rs").write_text("source\n")
        self.commit("source")
        self.source = self.git("rev-parse", "HEAD").strip()
        self.report_dir = self.root / "site/reports/benchmark-monty"
        self.report_dir.mkdir(parents=True)
        original = ROOT / "site/reports/benchmark-monty/20261028T230213Z-Mac.campaign.json"
        self.payload = json.loads(original.read_text())
        self.payload.update(host="rand-m5", campaign_id="20261028T230213Z-rand-m5", git_sha=self.source,
                            report_html="20261028T230213Z-rand-m5.html")
        self.json_path = self.report_dir / (self.payload["campaign_id"] + ".campaign.json")
        self.save()
        self.commit("evidence")

    def git(self, *args):
        return subprocess.run(["git", *args], cwd=self.root, text=True, capture_output=True, check=True, timeout=30).stdout

    def commit(self, message):
        self.git("add", ".")
        self.git("commit", "-qm", message)

    def save(self):
        self.json_path.write_text(json.dumps(self.payload))
        (self.report_dir / self.payload["report_html"]).write_text(
            " ".join(str(self.payload[key]) for key in ("campaign_id", "host", "git_sha", "status")))
        script = self.root / "scripts/benchmark/monty/campaign.py"
        spec = importlib.util.spec_from_file_location("fixture_campaign", script)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        try:
            module.write_index(self.report_dir)
        except module.CampaignError:
            pass  # Malformed fixtures must fail verification, not fixture setup.

    def rejects(self, text=None):
        with self.assertRaisesRegex(evidence.EvidenceError, text or "."):
            evidence.verify(self.root)

    def test_report_only_commit_is_accepted(self):
        result = evidence.verify(self.root)
        self.assertEqual(len(result), 1)
        self.assertIn(self.source, result[0])
        self.assertIn("MEASURED", result[0])

    def test_squash_history_with_identical_source_is_accepted(self):
        self.git("checkout", "--orphan", "squashed")
        self.commit("squash source and evidence")
        unrelated = subprocess.run(["git", "merge-base", "--is-ancestor", self.source, "HEAD"],
                                   cwd=self.root, check=False, timeout=30)
        self.assertNotEqual(unrelated.returncode, 0)
        self.assertEqual(len(evidence.verify(self.root)), 1)

    def test_changed_source_and_baseline_invalidate_evidence(self):
        for relative in ("source.rs", "scripts/benchmark/monty/baselines.json"):
            with self.subTest(path=relative):
                path = self.root / relative
                previous = path.read_text()
                path.write_text(previous + "\n")
                self.commit("changed release input")
                self.rejects("differs from measured source")
                path.write_text(previous)
                self.commit("restore exact release input")
                self.assertEqual(len(evidence.verify(self.root)), 1)

    def test_unknown_source_and_bad_metadata_are_rejected(self):
        for key, value in (("git_sha", "f" * 40), ("host", "Mac"), ("cargo_profile", "debug"),
                           ("rustc", "rustc 1.95.0"), ("schema", "unknown"), ("python_sc_compose", False),
                           ("utc", "tomorrow"), ("status", "PASS")):
            with self.subTest(key=key):
                original = self.payload[key]
                self.payload[key] = value
                self.save()
                self.commit("bad metadata")
                self.rejects()
                self.payload[key] = original
                self.save()
                self.commit("restore metadata")

    def test_missing_case_skips_and_bad_measurements_are_rejected(self):
        original = copy.deepcopy(self.payload)
        mutations = [lambda p: p["cases"].pop(), lambda p: p["cases"].append(p["cases"][0]),
                     lambda p: p["cases"][0].update(skipped=True),
                     lambda p: p["cases"][0].update(ms_per_run=-1),
                     lambda p: p["cases"][0].update(ms_per_run=float("nan")),
                     lambda p: p["cases"][0].update(timed_iters=0),
                     lambda p: p["cases"][0].update(total_ms=1),
                     lambda p: p["cases"][0].update(status="FAIL"),
                     lambda p: p.update(status="FAIL"), lambda p: p.update(status="INCOMPLETE")]
        for mutate in mutations:
            self.payload = copy.deepcopy(original)
            mutate(self.payload)
            self.save()
            self.commit("bad measurement")
            self.rejects()

    def test_untracked_and_dirty_evidence_are_rejected(self):
        (self.report_dir / "untracked.html").write_text("untracked")
        self.rejects("untracked")
        (self.report_dir / "untracked.html").unlink()
        self.json_path.write_text("{}")
        self.rejects("working tree/index")

    def test_missing_html_and_stale_index_are_rejected(self):
        (self.report_dir / self.payload["report_html"]).unlink()
        self.commit("missing html")
        with self.assertRaises(Exception):
            evidence.verify(self.root)
        self.save()
        self.commit("restore")
        (self.report_dir / "index.html").write_text("stale")
        self.commit("stale index")
        with self.assertRaisesRegex(Exception, "stale"):
            evidence.verify(self.root)

    def test_existing_campaign_cannot_be_rewritten_after_measurement(self):
        # Evidence that existed at the measured source must remain immutable.
        self.payload["git_sha"] = self.git("rev-parse", "HEAD").strip()
        self.save()
        self.commit("rewrite existing campaign")
        self.rejects("differs from measured source")


if __name__ == "__main__":
    unittest.main()
