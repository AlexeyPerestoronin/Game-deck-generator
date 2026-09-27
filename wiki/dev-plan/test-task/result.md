# Результат

## `Tools/harness/agents/tools/default_tools.py`
- `apply_diff_patch` через `whatthepatch` + обязательный `path` → `git apply` по тексту `patch` без `path`, цели патча проверяются на `w`-доступ (вся информация в патче; единообразие с `get_file_diff`)
- `is_file_under_git` не реализован → `git ls-files --error-unmatch` (нужно знать, можно ли вызывать diff/patch/discard)
- `discard_file_changes` не реализован → `git checkout HEAD -- file` (отмена изменений после HEAD)
- `whatthepatch` импортировался → импорт удалён (больше не используется)
- git-вызовы дублировали `subprocess.run` → общие `_run_git` / `_git_toplevel` / `_git_path_cwd` (DRY)
- `apply_diff_patch` / `is_file_under_git` / `discard_file_changes` не были в `list` → зарегистрированы как tools

## `Tools/harness/agents/tools/tests/default_tools_patch_file_test.py`
- пустой `TestPatchFile` с TODO → комплексные тесты без mock на временном git-репозитории (регистрация tools, tracked/untracked, roundtrip get/discard/apply, отказ вне `w`-dir, невалидный патч, hunk mismatch, `call()`)

## `Tools/harness/agents/tools/tool_help.md`
- не было описания новых git-инструментов / `apply_diff_patch` с `path` → добавлены `is_file_under_git`, `apply_diff_patch` (только `patch`), `discard_file_changes`; уточнён `get_file_diff`
