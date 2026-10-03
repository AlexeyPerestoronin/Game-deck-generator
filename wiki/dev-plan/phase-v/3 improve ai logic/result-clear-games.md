# result-clear-games

- `Workspace::clear`: `vfs = default` + сброс catalog/tabs/expanded скопом → только `games/` (через `clear_games`), `forget_path("games")`, persist оставшегося дерева (help и `ai-models/` не пропадают вместе с играми)
- `forget_path`: не трогал draft → забывает draft под удаляемым путём (вкладка игры закрывается, черновик не пишется обратно в VFS)
- confirm/tooltip/status: «wipe workspace» → удаляются игры, действие необратимо, help и AI-конфиги остаются
- native-тест: VFS с `games/a/game.json5` + `user-help.md` + `ai-models/x.json5` → после `clear_games` пустая `games/`, help и ключ на месте
