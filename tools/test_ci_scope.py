"""Verify CI exemptions against actual Git changes, including deleted code."""
from pathlib import Path
import subprocess
import tempfile
import unittest

from ci_scope import classify


class ScopeTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.git("init", "-q", "-b", "main")
        self.write("README.md", "initial docs\n")
        self.write("src/main.rs", "fn main() {}\n")
        self.base = self.commit()

    def git(self, *args):
        return subprocess.check_output(
            ["git", "-c", "user.name=CI", "-c", "user.email=ci@example.invalid",
             "-c", "core.hooksPath=/dev/null", *args], cwd=self.root, text=True
        ).strip()

    def write(self, name, content):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")

    def commit(self):
        self.git("add", ".")
        self.git("commit", "-qm", "fixture")
        return self.git("rev-parse", "HEAD")

    def scope(self, head, **kwargs):
        event = {"before": self.base, "after": head,
                 "pull_request": {"base": {"sha": self.base}, "head": {"sha": head}}}
        return classify(kwargs.pop("event_name", "pull_request"), event,
                        kwargs.pop("ref", "refs/pull/1/merge"), cwd=self.root, **kwargs)[0]

    def test_documents_skip_build_on_pr_and_main_push(self):
        self.write("README.md", "updated docs\n")
        self.write("docs/CI.md", "policy\n")
        self.write("src/CODEMAP.md", "navigation\n")
        head = self.commit()
        self.assertFalse(self.scope(head))
        self.assertFalse(self.scope(head, event_name="push", ref="refs/heads/main"))

    def test_code_mixed_with_documents_runs_full_ci(self):
        self.write("README.md", "updated docs\n")
        self.write("src/main.rs", "fn main() { println!(\"changed\"); }\n")
        self.assertTrue(self.scope(self.commit()))

    def test_workflows_and_unknown_assets_run_full_ci(self):
        for name in (".github/workflows/ci.yml", "docs/helper.py", "Cargo.lock", "assets/new.md"):
            with self.subTest(path=name):
                self.write(name, "changed\n")
                head = self.commit()
                self.assertTrue(self.scope(head))
                self.base = head

    def test_renaming_code_to_document_does_not_hide_deletion(self):
        (self.root / "docs").mkdir()
        self.git("mv", "src/main.rs", "docs/example.md")
        self.assertTrue(self.scope(self.commit()))

    def test_release_schedule_manual_and_forced_runs_are_full(self):
        self.write("README.md", "updated docs\n")
        head = self.commit()
        self.assertTrue(self.scope(head, event_name="push", ref="refs/tags/v0.1.0"))
        for event in ("schedule", "workflow_dispatch"):
            self.assertTrue(self.scope(head, event_name=event))
        self.assertTrue(self.scope(head, force_full=True))

    def test_empty_or_missing_comparison_runs_full_ci(self):
        self.assertTrue(self.scope(self.base))
        self.assertTrue(classify("push", {"before": "0" * 40, "after": self.base},
                                 "refs/heads/main", cwd=self.root)[0])


if __name__ == "__main__":
    unittest.main()
