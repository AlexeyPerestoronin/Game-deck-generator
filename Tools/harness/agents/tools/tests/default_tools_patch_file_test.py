"""Реальные тесты DefaultTools.apply_diff_patch без mock."""

import os
import unittest

from .. import default_tools

_DIR = os.path.dirname(os.path.abspath(__file__))
_SRC = os.path.join(_DIR, "default_tools_patch_file_test.txt")
_COPY = os.path.join(_DIR, "default_tools_patch_file_test_copy.txt")


class TestPatchFile(unittest.TestCase):
    """Сценарии git apply по рабочей копии фикстуры."""

    def setUp(self) -> None:
        with open(_SRC, "r", encoding="utf-8") as src:
            text = src.read()
        with open(_COPY, "w", encoding="utf-8", newline="\n") as dst:
            dst.write(text)
        self._tools = default_tools.DefaultTools(False, {
            "available-file-extensions": [".txt"],
            "dirs": [f"w:{_DIR}"],
        })

    def _read_copy(self) -> str:
        with open(_COPY, "r", encoding="utf-8") as handle:
            return handle.read()

    def _apply_copy(self, patch_text: str) -> str:
        result = self._tools.apply_diff_patch(_COPY, patch_text)
        self.assertEqual(result, f"successfully patched '{_COPY}'")
        return self._read_copy()

    def test_applies_hunk_without_headers(self) -> None:
        content = self._apply_copy("@@ -4,4 +4,4 @@\n"
                                   " def greet(name):\n"
                                   "     message = \"Hello, \" + name\n"
                                   "-    return message\n"
                                   "+    return message.upper()\n"
                                   " \n")
        self.assertIn("return message.upper()", content)
        self.assertNotIn("    return message\n", content)

    def test_retargets_headers_to_destination_file(self) -> None:
        content = self._apply_copy("diff --git a/other.txt b/other.txt\n"
                                   "--- a/other.txt\n"
                                   "+++ b/other.txt\n"
                                   "@@ -14,3 +14,3 @@\n"
                                   "-ALPHA = 1\n"
                                   "+ALPHA = 10\n"
                                   " BETA = 2\n"
                                   " GAMMA = 3\n")
        self.assertIn("ALPHA = 10\n", content)

    def test_applies_standard_unified_diff(self) -> None:
        name = os.path.basename(_COPY)
        content = self._apply_copy(f"diff --git a/{name} b/{name}\n"
                                   f"--- a/{name}\n"
                                   f"+++ b/{name}\n"
                                   "@@ -14,3 +14,3 @@\n"
                                   " ALPHA = 1\n"
                                   "-BETA = 2\n"
                                   "+BETA = 20\n"
                                   " GAMMA = 3\n")
        self.assertIn("BETA = 20\n", content)

    def test_changes_prose_line(self) -> None:
        content = self._apply_copy("@@ -19,3 +19,3 @@\n"
                                   " На холме над бухтой стоял старый маяк.\n"
                                   "-Каждый вечер смотритель зажигал лампу до прихода тумана.\n"
                                   "+Каждый вечер смотритель зажигал лампу до полуночи.\n"
                                   " Моряки верили этому лучу больше, чем любой карте.\n")
        self.assertIn("до полуночи.", content)
        self.assertNotIn("до прихода тумана.", content)

    def test_applies_multiple_hunks(self) -> None:
        content = self._apply_copy("@@ -14,3 +14,3 @@\n"
                                   " ALPHA = 1\n"
                                   " BETA = 2\n"
                                   "-GAMMA = 3\n"
                                   "+GAMMA = 30\n"
                                   "@@ -35,3 +35,3 @@\n"
                                   " TITLE = \"harbor notes\"\n"
                                   " STATUS = \"draft\"\n"
                                   "-END_MARKER = \"ok\"\n"
                                   "+END_MARKER = \"done\"\n")
        self.assertIn("GAMMA = 30\n", content)
        self.assertIn('END_MARKER = "done"\n', content)

    def test_denies_path_outside_w_dirs(self) -> None:
        outside = os.path.join(os.path.dirname(_DIR), "outside_apply_diff_patch.txt")
        with self.assertRaises(Exception) as ctx:
            self._tools.apply_diff_patch(outside, "@@ -1 +1 @@\n-a\n+b\n")
        self.assertIn("w-access denied", str(ctx.exception))

    def test_rejects_non_matching_context(self) -> None:
        before = self._read_copy()
        with self.assertRaises(Exception):
            self._tools.apply_diff_patch(
                _COPY,
                "@@ -1,1 +1,1 @@\n-not a real line\n+still fake\n",
            )
        self.assertEqual(before, self._read_copy())
