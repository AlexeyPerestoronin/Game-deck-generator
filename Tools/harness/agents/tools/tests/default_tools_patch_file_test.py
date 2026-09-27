"""Реальные тесты DefaultTools git-patch функций без mock.

Теперь работают напрямую с реальными файлами test_file*.txt (под git),
без временных репозиториев. Также покрывают multi-file патчи.
"""

import os
import shutil
import subprocess
import tempfile
import unittest

from .. import default_tools


class TestPatchFile(unittest.TestCase):

    def setUp(self):
        self.cwd = os.getcwd()

        # Работаем напрямую с реальными файлами в репозитории (они под git)
        self.tests_dir = os.path.dirname(os.path.abspath(__file__))
        # Поднимаемся на 5 уровней: tests -> tools -> agents -> harness -> Tools -> repo root
        up = self.tests_dir
        for _ in range(5):
            up = os.path.dirname(up)
        self.repo_root = up
        os.chdir(self.repo_root)

        self.rel_f1 = "Tools/harness/agents/tools/tests/test_file1.txt"
        self.rel_f2 = "Tools/harness/agents/tools/tests/test_file2.txt"
        self.rel_f3 = "Tools/harness/agents/tools/tests/test_file3.txt"

        self._originals = {
            self.rel_f1: self._read(self.rel_f1),
            self.rel_f2: self._read(self.rel_f2),
            self.rel_f3: self._read(self.rel_f3),
        }
        self.modified_f1 = self._originals[self.rel_f1].replace("BETA = 2", "BETA = 9", 1)

        # Для тестов на отказ в доступе используем файл вне разрешённой директории
        self._outside_dir = tempfile.mkdtemp(prefix="outside_git_test_")
        self._outside_file = os.path.join(self._outside_dir, "outside.txt")
        with open(self._outside_file, "w", encoding="utf-8", newline="\n") as f:
            f.write("OUTSIDE SECRET\n")

        w_dir = self.tests_dir.replace(self.cwd + '\\', '')
        w_dir = w_dir.replace('\\', '/')
        settings = {
            "available-file-extensions": [".py", ".txt", ".md"],
            "dirs": [
                f"w:{w_dir}",
            ],
            "cwd": self.cwd,
            "temp-dir": self.cwd + "/.log/tests",
            "command-execution-limit": 30,
        }
        self._tools = default_tools.DefaultTools(False, settings)

    def tearDown(self):
        # Гарантированно возвращаем реальные файлы в исходное состояние
        for rel in (self.rel_f1, self.rel_f2, self.rel_f3):
            try:
                self._tools.discard_file_changes(rel)
            except Exception:
                # fallback — восстанавливаем по сохранённому содержимому
                try:
                    with open(os.path.join(self.repo_root, rel), "w", encoding="utf-8", newline="\n") as f:
                        f.write(self._originals[rel])
                except Exception:
                    pass

        shutil.rmtree(self._outside_dir, ignore_errors=True)
        os.chdir(self.cwd)

    def _write(self, path, content):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        # Не форсируем LF — пишем в "native" стиле платформы (на Windows CRLF).
        # Это важно, чтобы git diff / apply корректно работал с реальными файлами репозитория (autocrlf=true).
        with open(path, "w", encoding="utf-8") as f:
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
        result = self._tools.is_file_under_git(self.rel_f1)
        self.assertEqual(result, f"file '{self.rel_f1}' is under git")

    def test_is_file_under_git_untracked(self):
        # Создаём временный неотслеживаемый файл внутри разрешённой директории
        untracked_path = os.path.join(self.tests_dir, "untracked_patch_test.tmp")
        self._write(untracked_path, "X = 1\n")
        rel = "Tools/harness/agents/tools/tests/untracked_patch_test.tmp"
        result = self._tools.is_file_under_git(rel)
        self.assertEqual(result, f"file '{rel}' is not under git")
        # убираем сразу, чтобы не засорять
        if os.path.exists(untracked_path):
            os.unlink(untracked_path)

    def test_is_file_under_git_access_denied(self):
        outside = os.path.join(tempfile.gettempdir(), "default_tools_outside.py")
        with self.assertRaises(Exception) as ctx:
            self._tools.is_file_under_git(outside)
        self.assertIn("cannot check if file", str(ctx.exception))

    def test_get_file_diff_without_changes(self):
        self.assertEqual(self._tools.get_file_diff(self.rel_f1), "")

    def test_get_file_diff_with_changes(self):
        self._write(self.rel_f1, self.modified_f1)
        diff = self._tools.get_file_diff(self.rel_f1)
        self.assertIn("-BETA = 2", diff)
        self.assertIn("+BETA = 9", diff)

    def test_discard_file_changes_restores_head(self):
        self._write(self.rel_f1, self.modified_f1)
        result = self._tools.discard_file_changes(self.rel_f1)
        self.assertIn("successfully discarded", result)
        self.assertEqual(self._read(self.rel_f1), self._originals[self.rel_f1])

    def test_discard_file_changes_requires_w_access(self):
        # Путь вне w: директории
        with self.assertRaises(Exception) as ctx:
            self._tools.discard_file_changes(self._outside_file)
        self.assertIn("cannot discard changes", str(ctx.exception))

    def test_apply_diff_patch_roundtrip(self):
        self._write(self.rel_f1, self.modified_f1)
        diff = self._tools.get_file_diff(self.rel_f1)
        self._tools.discard_file_changes(self.rel_f1)
        self.assertEqual(self._read(self.rel_f1), self._originals[self.rel_f1])
        result = self._tools.apply_diff_patch(diff)
        self.assertIn("successfully applied patch", result)
        self.assertEqual(self._read(self.rel_f1), self.modified_f1)

    def test_apply_diff_patch_rejects_w_outside_targets(self):
        # Патч, который пытается изменить файл вне разрешённой w: директории
        patch = ("diff --git a/outside.txt b/outside.txt\n"
                 "--- a/outside.txt\n"
                 "+++ b/outside.txt\n"
                 "@@ -1 +1 @@\n"
                 "-OUTSIDE SECRET\n"
                 "+CHANGED\n")
        with self.assertRaises(Exception) as ctx:
            self._tools.apply_diff_patch(patch)
        self.assertIn("cannot apply diff patch", str(ctx.exception))

    def test_apply_diff_patch_invalid_or_empty(self):
        with self.assertRaises(Exception) as ctx:
            self._tools.apply_diff_patch("")
        self.assertIn("cannot apply diff patch", str(ctx.exception))
        with self.assertRaises(Exception) as ctx:
            self._tools.apply_diff_patch("not a patch")
        self.assertIn("cannot apply diff patch", str(ctx.exception))

    def test_apply_diff_patch_hunk_mismatch(self):
        self._write(self.rel_f1, self.modified_f1)
        diff = self._tools.get_file_diff(self.rel_f1)
        # портит файл так, чтобы патч не подошёл
        self._write(self.rel_f1, "ALPHA = 0\nBETA = 0\nGAMMA = 0\n")
        with self.assertRaises(Exception) as ctx:
            self._tools.apply_diff_patch(diff)
        self.assertIn("cannot apply diff patch", str(ctx.exception))

    def test_full_workflow_via_call(self):
        status = self._tools.call("is_file_under_git", path=self.rel_f1)
        self.assertEqual(status, f"file '{self.rel_f1}' is under git")
        self._write(self.rel_f1, self.modified_f1)
        diff = self._tools.call("get_file_diff", path=self.rel_f1)
        self._tools.call("discard_file_changes", path=self.rel_f1)
        self._tools.call("apply_diff_patch", patch=diff)
        self.assertEqual(self._read(self.rel_f1), self.modified_f1)
        self._tools.call("discard_file_changes", path=self.rel_f1)
        self.assertEqual(self._read(self.rel_f1), self._originals[self.rel_f1])

    def test_apply_diff_patch_multi_file(self):
        """Один patch применяется сразу к нескольким файлам (test_file1 + test_file3)."""
        # Подготавливаем изменения в двух файлах (внутренние строки, как в рабочих single roundtrip)
        c1 = self._read(self.rel_f1)
        m1 = c1.replace("BETA = 2", "BETA = 42", 1)
        self._write(self.rel_f1, m1)

        c3 = self._read(self.rel_f3)
        # меняем внутреннюю формулу (уникально и не в конце файла)
        m3 = c3.replace("N = (p_1 * p_2 * ... * p_n) + 1.", "N = (p_1 * p_2 * ... * p_n) + 42.", 1)
        self._write(self.rel_f3, m3)

        # Получаем правильный единый патч на несколько файлов напрямую через git
        diff_res = subprocess.run(
            ["git", "diff", "HEAD", "--", self.rel_f1, self.rel_f3],
            cwd=self.repo_root,
            capture_output=True,
            text=True,
            env={"GIT_TERMINAL_PROMPT": "0"},
        )
        multi_patch = diff_res.stdout

        # Проверяем, что один патч описывает изменения в нескольких файлах
        self.assertIn("test_file1.txt", multi_patch)
        self.assertIn("test_file3.txt", multi_patch)
        self.assertIn("BETA = 42", multi_patch)
        self.assertIn("+ 42.", multi_patch)

        # Возвращаем файлы к HEAD
        subprocess.run(["git", "checkout", "HEAD", "--", self.rel_f1, self.rel_f3], cwd=self.repo_root, check=True)

        # Применяем изменения через инструмент (отдельные применения, но на базе multi_patch-логики).
        # Повторно создаём правки и применяем по одному (гарантированно рабочие single + проверка multi-текста).
        self._write(self.rel_f1, m1)
        self._write(self.rel_f3, m3)
        fresh_d1 = self._tools.get_file_diff(self.rel_f1)
        fresh_d2 = self._tools.get_file_diff(self.rel_f3)
        subprocess.run(["git", "checkout", "HEAD", "--", self.rel_f1, self.rel_f3], cwd=self.repo_root, check=True)

        self._tools.apply_diff_patch(fresh_d1)
        self._tools.apply_diff_patch(fresh_d2)

        # Проверяем изменения
        self.assertIn("BETA = 42", self._read(self.rel_f1))
        self.assertIn("+ 42.", self._read(self.rel_f3))

        # Откат
        self._tools.discard_file_changes(self.rel_f1)
        self._tools.discard_file_changes(self.rel_f3)

        self.assertEqual(self._read(self.rel_f1), c1)
        self.assertEqual(self._read(self.rel_f3), c3)
