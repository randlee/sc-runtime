"""Release verification consumes committed campaigns without network or builds."""
import copy
import importlib.util
import json
import re
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
        script = self.root / "scripts/benchmark/monty/campaign.py"
        spec = importlib.util.spec_from_file_location("fixture_campaign", script)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        # Start with a real Rust-harness report, then use its canonical packager.
        page = (ROOT / "site/reports/benchmark-monty/20261028T230213Z-Mac.html").read_text()
        for case in self.payload["cases"]:
            pattern = (r'(<tr id="' + re.escape(case["id"]) + r'".*?<td class="num">)[^<]*'
                       r'(</td><td class="num">)[^<]*(</td><td class="num">)[^<]*')
            page = re.sub(pattern, lambda m: (m[1] + f"{case['ms_per_run']:.3f} ms" + m[2]
                          + f"{case['total_ms']}ms total" + m[3] + f"{case['timed_iters']} + 1 warmup"), page)
        (self.report_dir / self.payload["report_html"]).write_text(module.stamp_report_html(page, self.payload))
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

    def test_metadata_only_malformed_and_contradictory_html_are_rejected(self):
        path = self.report_dir / self.payload["report_html"]
        canonical = path.read_text()
        mutations = [" ".join(str(self.payload[key]) for key in ("campaign_id", "host", "git_sha", "rustc")),
                     canonical.replace(f"{self.payload['cases'][0]['ms_per_run']:.3f} ms", "999999.000 ms", 1),
                     canonical.replace("</td>", "</div>", 1),
                     canonical.replace("30 + 1 warmup", "3000 + 1 warmup", 1),
                     canonical.replace("6.47 MiB", "99999 MiB")]
        # The final case corrupts a known actual RSS cell rather than inventing a fixture document.
        rss = self.payload["cases"][0]["subprocess_rss_bytes"] / 1048576
        mutations[-1] = canonical.replace(f"{rss:.2f} MiB", "99999 MiB", 1)
        for page in mutations:
            path.write_text(page)
            self.commit("corrupt real canonical HTML")
            self.rejects()

    def test_newest_applicable_failure_blocks_older_pass(self):
        self.git("rm", "-r", "site/reports/benchmark-monty")
        baseline_path = self.root / "scripts/benchmark/monty/baselines.json"
        baseline = json.loads(baseline_path.read_text())
        for spec in baseline["cases"].values():
            spec["max_ms_per_run"] = 100
        baseline_path.write_text(json.dumps(baseline))
        self.commit("measurement source with ratified test ceilings")
        self.source = self.git("rev-parse", "HEAD").strip()
        self.report_dir.mkdir(parents=True)
        self.payload["git_sha"] = self.source
        self.payload["status"] = "PASS"
        for case in self.payload["cases"]:
            case.update(max_ms_per_run=100, status="PASS")
        self.save()
        self.commit("older passing campaign")
        self.assertEqual(len(evidence.verify(self.root)), 1)
        older = copy.deepcopy(self.payload)
        for kind in ("FAIL", "INCOMPLETE", "malformed"):
            self.payload = copy.deepcopy(older)
            self.payload.update(campaign_id="20261029T230213Z-rand-m5", utc="2026-10-29T23:02:13Z",
                                report_html="20261029T230213Z-rand-m5.html", status=kind)
            self.json_path = self.report_dir / (self.payload["campaign_id"] + ".campaign.json")
            if kind == "FAIL":
                self.payload["cases"][0].update(ms_per_run=101, total_ms=3030, status="FAIL")
            elif kind == "INCOMPLETE":
                self.payload["cases"][0].update(skipped=True, status="SKIPPED")
            else:
                self.payload["cases"].pop()
            self.save()
            self.commit("later applicable negative campaign")
            self.rejects()

    def test_inapplicable_newer_failure_does_not_hide_current_source_evidence(self):
        original = copy.deepcopy(self.payload)
        (self.root / "source.rs").write_text("new source\n")
        self.commit("new source")
        self.source = self.git("rev-parse", "HEAD").strip()
        self.payload.update(campaign_id="20261029T230213Z-rand-m5", utc="2026-10-29T23:02:13Z",
                            report_html="20261029T230213Z-rand-m5.html", git_sha=self.source)
        self.json_path = self.report_dir / (self.payload["campaign_id"] + ".campaign.json")
        self.save()
        self.commit("new source measured")
        self.payload = original
        self.payload.update(campaign_id="20261030T230213Z-rand-m5", utc="2026-10-30T23:02:13Z",
                            report_html="20261030T230213Z-rand-m5.html", status="FAIL")
        self.json_path = self.report_dir / (self.payload["campaign_id"] + ".campaign.json")
        self.save()
        self.commit("historical old-source failure submitted later")
        self.assertIn(self.source, evidence.verify(self.root)[0])

    def test_existing_campaign_cannot_be_rewritten_after_measurement(self):
        # Evidence that existed at the measured source must remain immutable.
        self.payload["git_sha"] = self.git("rev-parse", "HEAD").strip()
        self.save()
        self.commit("rewrite existing campaign")
        self.rejects("differs from measured source")


if __name__ == "__main__":
    unittest.main()
