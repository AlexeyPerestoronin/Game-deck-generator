"""Реальные тесты DefaultTools git-patch функций без mock."""

import os
import shutil
import subprocess
import tempfile
import unittest

from .. import default_tools


class TestPatchFile(unittest.TestCase):
    def setUp(self):
        self._old_cwd = os.getcwd()
        self._repo = tempfile.mkdtemp(prefix="default_tools_git_")
        self._allowed_dir = os.path.join(self._repo, "allowed")
        self._forbidden_dir = os.path.join(self._repo, "forbidden")
        os.makedirs(self._allowed_dir)
        os.makedirs(self._forbidden_dir)

        # Use the real fixture files from the test directory (per task requirement)
        tests_dir = os.path.dirname(os.path.abspath(__file__))
        for name in ("test_file1.txt", "test_file2.txt", "test_file3.txt"):
            src = os.path.join(tests_dir, name)
            dst = os.path.join(self._allowed_dir, name)
            with open(src, "r", encoding="utf-8", newline=None) as f_in:
                content = f_in.read()
            with open(dst, "w", encoding="utf-8", newline="\n") as f_out:
                f_out.write(content)

        self._rel = os.path.join("allowed", "test_file1.txt")
        self._file = os.path.join(self._repo, self._rel)

        self._forbidden_rel = os.path.join("forbidden", "secret.py")
        self._forbidden_file = os.path.join(self._repo, self._forbidden_rel)

        self._write(self._forbidden_file, "SECRET = 1\n")

        try:
            self._git("init")
            self._git("config", "user.name", "Test User")
            self._git("config", "user.email", "test@example.com")
            self._git("config", "core.autocrlf", "false")
            self._git("add", ".")
            self._git("-c", "commit.gpgsign=false", "commit", "-m", "init")

            # originals from the real committed fixtures
            self._original = self._read(self._file)
            self._modified = self._original.replace("BETA = 2", "BETA = 9", 1)

            self._tools = default_tools.DefaultTools(False, {
                "available-file-extensions": [".py", ".txt", ".md"],
                "dirs": [
                    f"w:{self._allowed_dir}",
                    f"r:{self._repo}",
                ],
                "command-execution-limit": 30,
            })
            os.chdir(self._repo)
        except Exception:
            os.chdir(self._old_cwd)
            shutil.rmtree(self._repo, ignore_errors=True)
            raise

    def tearDown(self):
        os.chdir(self._old_cwd)
        shutil.rmtree(self._repo, ignore_errors=True)

    def _write(self, path, content):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            f.write(content)

    def _read(self, path):
        with open(path, "r", encoding="utf-8") as f:
            return f.read()

    def _git(self, *args):
        env = os.environ.copy()
        env["GIT_AUTHOR_NAME"] = "Test User"
        env["GIT_AUTHOR_EMAIL"] = "test@example.com"
        env["GIT_COMMITTER_NAME"] = "Test User"
        env["GIT_COMMITTER_EMAIL"] = "test@example.com"
        env["GIT_TERMINAL_PROMPT"] = "0"
        result = subprocess.run(
            ["git", *args],
            cwd=self._repo,
            capture_output=True,
            encoding="utf-8",
            errors="replace",
            env=env,
        )
        if result.returncode != 0:
            raise RuntimeError(f"git {args} failed: {(result.stderr or result.stdout).strip()}")
        return result

    def test_git_tools_are_registered(self):
        names = [t.name for t in self._tools.list]
        for name in ("is_file_under_git", "apply_diff_patch", "get_file_diff", "discard_file_changes"):
            self.assertIn(name, names)

    def test_apply_diff_patch_parameters_do_not_include_path(self):
        tool = next(t for t in self._tools.list if t.name == "apply_diff_patch")
        self.assertEqual(["patch"], tool.parameters["required"])
        self.assertIn("patch", tool.parameters["properties"])
        self.assertNotIn("path", tool.parameters["properties"])

    def test_is_file_under_git_tracked(self):
        result = self._tools.is_file_under_git(self._rel)
        self.assertEqual(result, f"file '{self._rel}' is under git")

    def test_is_file_under_git_untracked(self):
        rel = os.path.join("allowed", "new.py")
        self._write(os.path.join(self._repo, rel), "X = 1\n")
        result = self._tools.is_file_under_git(rel)
        self.assertEqual(result, f"file '{rel}' is not under git")

    def test_is_file_under_git_access_denied(self):
        outside = os.path.join(tempfile.gettempdir(), "default_tools_outside.py")
        with self.assertRaises(Exception) as ctx:
            self._tools.is_file_under_git(outside)
        self.assertIn("cannot check if file", str(ctx.exception))

    def test_get_file_diff_without_changes(self):
        self.assertEqual(self._tools.get_file_diff(self._rel), "")

    def test_get_file_diff_with_changes(self):
        self._write(self._file, self._modified)
        diff = self._tools.get_file_diff(self._rel)
        self.assertIn("-BETA = 2", diff)
        self.assertIn("+BETA = 9", diff)

    def test_discard_file_changes_restores_head(self):
        self._write(self._file, self._modified)
        result = self._tools.discard_file_changes(self._rel)
        self.assertIn("successfully discarded", result)
        self.assertEqual(self._read(self._file), self._original)

    def test_discard_file_changes_requires_w_access(self):
        with self.assertRaises(Exception) as ctx:
            self._tools.discard_file_changes(self._forbidden_rel)
        self.assertIn("cannot discard changes", str(ctx.exception))

    def test_apply_diff_patch_roundtrip(self):
        self._write(self._file, self._modified)
        diff = self._tools.get_file_diff(self._rel)
        self._tools.discard_file_changes(self._rel)
        self.assertEqual(self._read(self._file), self._original)
        result = self._tools.apply_diff_patch(diff)
        self.assertIn("successfully applied patch", result)
        self.assertEqual(self._read(self._file), self._modified)

    def test_apply_diff_patch_rejects_w_outside_targets(self):
        patch = (
            "diff --git a/forbidden/secret.py b/forbidden/secret.py\n"
            "--- a/forbidden/secret.py\n"
            "+++ b/forbidden/secret.py\n"
            "@@ -1 +1 @@\n"
            "-SECRET = 1\n"
            "+SECRET = 2\n"
        )
        with self.assertRaises(Exception) as ctx:
            self._tools.apply_diff_patch(patch)
        self.assertIn("cannot apply diff patch", str(ctx.exception))
        self.assertEqual(self._read(self._forbidden_file), "SECRET = 1\n")

    def test_apply_diff_patch_invalid_or_empty(self):
        with self.assertRaises(Exception) as ctx:
            self._tools.apply_diff_patch("")
        self.assertIn("cannot apply diff patch", str(ctx.exception))
        with self.assertRaises(Exception) as ctx:
            self._tools.apply_diff_patch("not a patch")
        self.assertIn("cannot apply diff patch", str(ctx.exception))

    def test_apply_diff_patch_hunk_mismatch(self):
        self._write(self._file, self._modified)
        diff = self._tools.get_file_diff(self._rel)
        self._write(self._file, "ALPHA = 0\nBETA = 0\nGAMMA = 0\n")
        with self.assertRaises(Exception) as ctx:
            self._tools.apply_diff_patch(diff)
        self.assertIn("cannot apply diff patch", str(ctx.exception))

    def test_full_workflow_via_call(self):
        status = self._tools.call("is_file_under_git", path=self._rel)
        self.assertEqual(status, f"file '{self._rel}' is under git")
        self._write(self._file, self._modified)
        diff = self._tools.call("get_file_diff", path=self._rel)
        self._tools.call("discard_file_changes", path=self._rel)
        self._tools.call("apply_diff_patch", patch=diff)
        self.assertEqual(self._read(self._file), self._modified)
        self._tools.call("discard_file_changes", path=self._rel)
        self.assertEqual(self._read(self._file), self._original)
