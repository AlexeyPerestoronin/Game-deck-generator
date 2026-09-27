# Note

Проверьте, пожалуйста, два допущения — в задаче их не было:

1. Локальный web-сайт ищется как процесс в состоянии LISTENING на порту **8080** (дефолт trunk). Если сайт на другом порту — напишите номер.
2. Release-бинарник копируется как `Projects/target/release/deck_gen` (или `deck_gen.exe`). Если имя бинарника или `target-dir` другие — напишите.

Код не запускал (по указанию). Сборку проверьте сами из корня репозитория:
- `invoke tools.build-deck-gen`
- `invoke tools.build-deck-gen --debug=False`
- `invoke tools.build-deck-gen-wasm`
