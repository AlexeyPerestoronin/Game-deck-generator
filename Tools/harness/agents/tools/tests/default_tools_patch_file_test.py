import os
import git
import unittest

from unittest.mock import patch

from .. import default_tools


class TestPatchFile(unittest.TestCase):

    def setUp(self) -> None:
        self._dir = "sandbox"
        self._tools = default_tools.DefaultTools(False, {
            "available-file-extensions": [".py"],
            "dirs": [f"w:{self._dir}"],
        })

    def _apply(self, path: str, patch_text: str) -> tuple[str, str]:
        captured: dict[str, str] = {}

        def apply(istream=None, **kwargs) -> None:
            captured["payload"] = istream.read().decode("utf-8")

        with patch.object(git, "Git") as mock_git:
            mock_git.return_value.apply.side_effect = apply
            result = self._tools.apply_diff_patch(path, patch_text)
            mock_git.assert_called_once_with(os.path.dirname(os.path.abspath(path)))
        return result, captured["payload"]

    def test_applies_hunk_without_headers(self) -> None:
        path = os.path.join(self._dir, "app.py")
        patch_text = "@@ -1,3 +1,3 @@\n print('a')\n-print('b')\n+print('c')\n print('d')\n"
        result, payload = self._apply(path, patch_text)
        self.assertEqual(result, f"success: patched {path}")
        self.assertEqual(
            payload,
            "diff --git a/app.py b/app.py\n--- a/app.py\n+++ b/app.py\n" + patch_text,
        )

    def test_retargets_headers_to_destination_file(self) -> None:
        path = os.path.join(self._dir, "target.py")
        patch_text = ("diff --git a/other.py b/other.py\n"
                      "--- a/other.py\n"
                      "+++ b/other.py\n"
                      "@@ -1,1 +1,1 @@\n"
                      "-x = 1\n"
                      "+x = 2\n")
        _, payload = self._apply(path, patch_text)
        self.assertEqual(
            payload,
            "diff --git a/target.py b/target.py\n"
            "--- a/target.py\n"
            "+++ b/target.py\n"
            "@@ -1,1 +1,1 @@\n"
            "-x = 1\n"
            "+x = 2\n",
        )

    def test_preserves_dev_null_headers(self) -> None:
        path = os.path.join(self._dir, "new.py")
        patch_text = ("diff --git a/old.py b/old.py\n"
                      "--- /dev/null\n"
                      "+++ b/old.py\n"
                      "@@ -0,0 +1,1 @@\n"
                      "+hello\n")
        _, payload = self._apply(path, patch_text)
        self.assertEqual(
            payload,
            "diff --git a/new.py b/new.py\n"
            "--- /dev/null\n"
            "+++ b/new.py\n"
            "@@ -0,0 +1,1 @@\n"
            "+hello\n",
        )

    def test_denies_path_outside_w_dirs(self) -> None:
        with self.assertRaises(Exception) as ctx:
            self._tools.apply_diff_patch("outside.py", "@@ -1 +1 @@\n-a\n+b\n")
        self.assertIn("w-access denied", str(ctx.exception))
