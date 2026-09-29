"""Тесты DefaultTools.run_invoke: регистрация и вызов subprocess."""

import os
import subprocess
import unittest
from unittest.mock import MagicMock, patch

from .. import default_tools


class _NoopLogger:
    def log_line(self, message: str = "") -> "_NoopLogger":
        return self


class TestRunInvoke(unittest.TestCase):

    def setUp(self):
        self.cwd = os.path.abspath(os.getcwd())
        settings = {
            "available-file-extensions": [],
            "dirs": [f"w:{self.cwd}"],
            "cwd": self.cwd,
            "temp-dir": ".log/tests",
            "command-execution-limit": 30,
        }
        self._tools = default_tools.DefaultTools(False, _NoopLogger(), settings)

    def test_run_invoke_is_registered(self):
        names = [t.name for t in self._tools.list]
        self.assertIn("run_invoke", names)
        tool = next(t for t in self._tools.list if t.name == "run_invoke")
        self.assertEqual(["command"], tool.parameters["required"])
        self.assertIn("command", tool.parameters["properties"])

    def test_run_invoke_runs_invoke_in_cwd(self):
        mock_result = MagicMock()
        mock_result.stdout = b"Available tasks:\n\n"
        mock_result.stderr = b""
        with patch.object(default_tools.subprocess, "run", return_value=mock_result) as mock_run:
            result = self._tools.run_invoke("--list")
        self.assertEqual(result, "Available tasks:\n\n")
        mock_run.assert_called_once()
        args, kwargs = mock_run.call_args
        self.assertEqual(args[0], "invoke --list")
        self.assertEqual(kwargs["cwd"], self.cwd)
        self.assertTrue(kwargs["shell"])
        self.assertTrue(kwargs["capture_output"])
        self.assertEqual(kwargs["timeout"], 30)

    def test_run_invoke_empty_output(self):
        mock_result = MagicMock()
        mock_result.stdout = b""
        mock_result.stderr = b""
        with patch.object(default_tools.subprocess, "run", return_value=mock_result):
            result = self._tools.run_invoke("noop")
        self.assertEqual(result, "(command finished without output)")

    def test_run_invoke_timeout(self):
        with patch.object(default_tools.subprocess, "run", side_effect=subprocess.TimeoutExpired("invoke --list", 30)):
            with self.assertRaises(Exception) as ctx:
                self._tools.run_invoke("--list")
        self.assertIn("exceed the limit", str(ctx.exception))
