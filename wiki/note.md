# Что сделать с вашей стороны

1. Закоммитить и запушить в `master` файл `deck_gen_wasm/user-help.md` (и правки WASM). Пока файла нет на GitHub, первый заход на Pages возьмёт его только если сработает локальный fallback; на уже задеплоенном сайте fallback — тот `user-help.md`, который Trunk положил в `dist`.
2. Чтобы снова увидеть first-visit preview: в DevTools → Application → Local Storage удалить ключ `deck_gen_wasm.session` и обновить страницу. Clear в UI пустую сессию сохраняет, поэтому reload после Clear help сам не откроет.
3. Для локальной проверки fallback пересоберите Trunk (`serve.bat` / `trunk serve`), чтобы `copy-file` скопировал `user-help.md` в `dist`.
