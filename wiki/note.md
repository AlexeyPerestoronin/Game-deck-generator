# Note after phase-III/stage-4 vscode_like_ui

## Требуемые действия от пользователя (после мерджа этой задачи)
- Заменить placeholder иконки на нарисованные (как указано в плане):
  - deck_gen_wasm/icons/buttons/explorer/{off,on,click,active}.drawio.png
  - deck_gen_wasm/icons/buttons/games/{off,on,click,active}.drawio.png
  - deck_gen_wasm/icons/buttons/settings/{off,on,click}.drawio.png
  (текущие — копии split/clear для работоспособности)

## Что проверить вручную в браузере (serve.bat → http://127.0.0.1:8080)
1. Старт: левая панель скрыта (нет explorer), activity: Explorer | Games наверху.
2. Клик Explorer → появляется дерево (как раньше), кнопка подсвечена active.
3. Клик Explorer ещё раз → панель скрывается (width=0).
4. Клик Games → панель показывает Local + Global (по умолчанию обе открыты, делят ~пополам).
5. В Local: кнопка "Load Local Games" → должен открыться confirm (как раньше Load), после выбора грузит.
6. В Global: при входе в режим — запрос к GitHub, список папок (new-game, monopoly-2.0, ...). Для каждой — кнопка Settings → меню "Load Games" → грузит именно эту игру с GH (retarget + unique).
7. Свернуть/развернуть Local/Global по клику на заголовок (chevron).
8. Перетащить горизонтальный разделитель между Local и Global — меняет пропорции высот.
9. Клик Split Workspace — active как раньше, не влияет на sidebar mode.
10. Клик Settings (внизу) → меню с 3 пунктами: Send e-mail, Change theme, Localization (действия работают, как раньше отдельные кнопки).
11. Feedback/Theme/Locale/ New/Load с activity bar исчезли.
12. Drag вертикального resizer слева от editor меняет ширину sidebar (когда активен Explorer/Games).
13. Prepare Html/Pdf, Download, Clear — работают без изменения sidebar.

## Известные нюансы (stage-1, KISS)
- Начальная ширина при показе панели = 260 или последняя.
- При width=0 и скрытом режиме drag resizer расширит ширину, но режим останется Hidden (кликни кнопку чтобы активировать).
- Список Global кэшируется только на время визита в режим Games (переключение прячет/показывает).
- Placeholder иконки (будут заменены).
- Нет persist SidebarMode (по плану).
- Ошибки сети GitHub идут в Alert (как и раньше для new-game).

После отрисовки иконок и ручной проверки в браузере — обновить эту заметку или закрыть.
