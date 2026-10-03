"""Unit-тесты DefaultTools.find_str и find_str_by_regex на файлах test_file*.txt."""

import os
import sys
import tempfile
import unittest
from io import StringIO
from unittest.mock import patch

from .. import default_tools


class _NoopLogger:
    """Минимальная заглушка ILogger для тестов (не пишет логи)."""

    def log_line(self, message: str = "") -> "_NoopLogger":
        return self


class TestFindStr(unittest.TestCase):

    def setUp(self):
        self.cwd = os.getcwd()
        self.tests_dir = os.path.dirname(os.path.abspath(__file__))
        up = self.tests_dir
        for _ in range(5):
            up = os.path.dirname(up)
        self.repo_root = up
        os.chdir(self.repo_root)

        self.rel_tests = "tools/harness/agents/tools/tests"
        self.rel_f1 = f"{self.rel_tests}/test_file1.txt"
        self.rel_f2 = f"{self.rel_tests}/test_file2.txt"
        self.rel_f3 = f"{self.rel_tests}/test_file3.txt"

        settings = {
            "available-file-extensions": [".txt"],
            "dirs": [f"w:{self.rel_tests}"],
            "cwd": self.repo_root,
            "temp-dir": ".log/tests",
            "command-execution-limit": 30,
        }
        self._tools = default_tools.DefaultTools(False, _NoopLogger(), settings)

    def tearDown(self):
        os.chdir(self.cwd)

    def _norm(self, path: str) -> str:
        return path.replace("\\", "/")

    def _find_str(self, root: str, string: str) -> str:
        with patch.object(sys, "stdout", new_callable=StringIO):
            return self._tools.find_str(root, string)

    def _find_re(self, root: str, regex: str) -> str:
        with patch.object(sys, "stdout", new_callable=StringIO):
            return self._tools.find_str_by_regex(root, regex)

    def _result_lines(self, result: str) -> list:
        return [ln for ln in result.splitlines() if ln]

    def _paths_in(self, result: str) -> list:
        return [self._norm(ln.split(":", 1)[0]) for ln in self._result_lines(result)]

    def test_tools_are_registered(self):
        names = [t.name for t in self._tools.list]
        self.assertIn("find_str", names)
        self.assertIn("find_str_by_regex", names)

    def test_find_str_parameters(self):
        tool = next(t for t in self._tools.list if t.name == "find_str")
        self.assertEqual(["root", "string"], tool.parameters["required"])
        self.assertIn("root", tool.parameters["properties"])
        self.assertIn("string", tool.parameters["properties"])

    def test_find_str_by_regex_parameters(self):
        tool = next(t for t in self._tools.list if t.name == "find_str_by_regex")
        self.assertEqual(["root", "regex"], tool.parameters["required"])
        self.assertIn("root", tool.parameters["properties"])
        self.assertIn("regex", tool.parameters["properties"])

    def test_find_str_unique_in_test_file1(self):
        needle = "На холме над бухтой стоял старый маяк."
        result = self._find_str(self.rel_tests, needle)
        lines = self._result_lines(result)
        self.assertEqual(len(lines), 1)
        self.assertIn(needle, lines[0])
        self.assertTrue(self._paths_in(result)[0].endswith("test_file1.txt"))

    def test_find_str_unique_in_test_file3(self):
        needle = "Фюрстенберга"
        result = self._find_str(self.rel_tests, needle)
        lines = self._result_lines(result)
        self.assertGreaterEqual(len(lines), 1)
        for ln in lines:
            self.assertIn(needle, ln)
            self.assertIn("test_file3.txt", self._norm(ln))

    def test_find_str_multiple_matches_in_test_file2(self):
        needle = "аленький цветочек"
        result = self._find_str(self.rel_f2, needle)
        lines = self._result_lines(result)
        self.assertGreaterEqual(len(lines), 2)
        for ln in lines:
            self.assertIn(needle, ln)
            self.assertIn("test_file2.txt", self._norm(ln))

    def test_find_str_single_file_root(self):
        result = self._find_str(self.rel_f1, "GAMMA = 3")
        lines = self._result_lines(result)
        self.assertEqual(len(lines), 1)
        path, content = lines[0].split(":", 1)
        self.assertTrue(self._norm(path).endswith("test_file1.txt"))
        self.assertEqual(content, "GAMMA = 3")

    def test_find_str_no_match(self):
        result = self._find_str(self.rel_tests, "this-string-does-not-exist-xyz-98765")
        self.assertEqual(result, "")

    def test_find_str_access_denied(self):
        outside = os.path.join(tempfile.gettempdir(), "default_tools_find_str_outside")
        with self.assertRaises(Exception) as ctx:
            self._find_str(outside, "x")
        self.assertIn("cannot find_str", str(ctx.exception))

    def test_find_str_by_regex_in_test_file1(self):
        result = self._find_re(self.rel_f1, r"BETA = \d+")
        lines = self._result_lines(result)
        self.assertEqual(len(lines), 1)
        self.assertIn("BETA = 2", lines[0])
        self.assertIn("test_file1.txt", self._norm(lines[0]))

    def test_find_str_by_regex_in_test_file3(self):
        result = self._find_re(self.rel_f3, r"2\*3\*5\*7\*11 \+ 1 = 2311")
        lines = self._result_lines(result)
        self.assertEqual(len(lines), 1)
        self.assertIn("2311", lines[0])
        self.assertIn("test_file3.txt", self._norm(lines[0]))

    def test_find_str_by_regex_across_files(self):
        result = self._find_re(self.rel_tests, r"аленький цветочек")
        lines = self._result_lines(result)
        self.assertGreaterEqual(len(lines), 2)
        for ln in lines:
            self.assertIn("test_file2.txt", self._norm(ln))

    def test_find_str_by_regex_invalid(self):
        with self.assertRaises(Exception) as ctx:
            self._find_re(self.rel_tests, r"[unclosed")
        self.assertIn("cannot find_str_by_regex", str(ctx.exception))

    def test_find_str_progress_summary(self):
        buf = StringIO()
        with patch.object(sys, "stdout", buf):
            self._tools.find_str(self.rel_f1, "ALPHA = 1")
        out = buf.getvalue()
        self.assertIn("find_str(", out)
        self.assertIn("root =", out)
        self.assertIn("string =", out)
        self.assertIn("files observed:", out)
        self.assertIn("match detected:", out)
        self.assertIn("active scanning from:", out)

    def test_find_str_by_regex_progress_summary(self):
        buf = StringIO()
        with patch.object(sys, "stdout", buf):
            self._tools.find_str_by_regex(self.rel_f3, r"Евклид")
        out = buf.getvalue()
        self.assertIn("find_str_by_regex(", out)
        self.assertIn("root =", out)
        self.assertIn("regex =", out)
        self.assertIn("files observed:", out)
        self.assertIn("match detected:", out)
        self.assertIn("active scanning from:", out)

    def test_find_str_via_call(self):
        with patch.object(sys, "stdout", new_callable=StringIO):
            result = self._tools.call("find_str", root=self.rel_f1, string="END_MARKER")
        self.assertIn("END_MARKER", result)
        self.assertIn("test_file1.txt", self._norm(result))
