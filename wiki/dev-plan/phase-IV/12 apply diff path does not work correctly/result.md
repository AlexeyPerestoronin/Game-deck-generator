# Результаты исправления apply_diff_patch

## Что было → стало (и почему)

- Запись патча в apply_diff_patch: f.write(patch) → нормализация + trailing \n + newline='\n' в open  
  (было→стало: патч-файл всегда оканчивается на LF; устраняет corrupt patch когда агент прислал строку без финального \n)

- get/discard/is_under_git: basename + subdir-cwd → полный path + main _cwd  
  (было→стало: патчи содержат корректные пути от корня репо; apply работает для вложенных путей)

- Тесты: добавлен _NoopLogger, исправлен cwd/temp/регистр путей на 'Tools/...', добавлен dedicated тест без trailing nl  
  (было→стало: 16/16 TestPatchFile проходят)

Покрыты требования todo.md.
