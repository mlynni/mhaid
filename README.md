# Mlyns Personal Sources — Aidoku-екстеншени з Mihon

Персональний репозиторій розширень (джерел) для **[Aidoku](https://aidoku.app)** (iOS / iPadOS, версія 0.7+, для Comix — 0.8.4+, для Mangadotnet v10 — 0.9.0+), портованих із розширень Mihon (`keiyoushi/extensions-source`) у формат Rust/WebAssembly (`aidoku-rs`).

## Підтримувані екстеншени (17 джерел)

| Джерело | ID в Aidoku | Мови | Версія | Мін. версія Aidoku | Базовий URL |
|---|---|---|---:|---:|---|
| **Atsumaru** | `en.atsumaru` | `en` | v12 | `0.7.1` | https://atsu.moe |
| **Comix** | `en.comix` | `en` | v29 | `0.8.4` | https://comix.to |
| **Kagane** | `multi.kagane` | 15 мов | v26 | `0.7.1` | https://kagane.moe |
| **MANGA Plus** | `multi.mangaplus` | 9 мов | v6 | `0.7.2` | https://mangaplus.shueisha.co.jp |
| **Mangaball** | `multi.mangaball` | 34 мови (включно з `uk`) | v12 | `0.7.1` | https://mangaball.com |
| **MangaBat** | `en.mangabat` | `en` | v9 | `0.7.1` | https://www.mangabats.com |
| **MangaDex** | `multi.mangadex` | 54 мови (включно з `uk`, `en`) | v16 | `0.7.1` | https://mangadex.org |
| **MangaDotNet** | `en.mangadot` | 43 мови (включно з `uk`) | v20 | `0.8.3` | https://mangadot.net |
| **Mangadotnet** | `multi.mangadotnet` | 43 мови (включно з `uk`) | v10 | `0.9.0` | https://mangadot.net |
| **MangaFire** | `multi.mangafire` | `en`, `es`, `es-419`, `fr`, `ja`, `pt`, `pt-BR` | v9 | `0.7.1` | https://mangafire.to |
| **Mangago** | `en.mangago` | `en` | v4 | `0.7.1` | https://www.mangago.me |
| **MangaKakalot** | `en.mangakakalot` | `en` | v6 | `0.7.1` | https://www.mangakakalot.gg |
| **MangaNato** | `en.manganato` | `en` | v8 | `0.7.1` | https://www.manganato.gg |
| **MangaNelo** | `en.manganelo` | `en` | v4 | `0.7.1` | https://www.nelomanga.net |
| **MangaPill** | `en.mangapill` | `en` | v4 | `0.7.1` | https://mangapill.com |
| **Toonily** | `en.toonily` | `en` | v10 | `0.7.1` | https://toonily.com |
| **Weeb Central** | `en.weebcentral` | `en` | v9 | `0.7.1` | https://weebcentral.com |

## Як додати в додаток Aidoku

1. Відкрийте **Aidoku** на iPhone або iPad.
2. Перейдіть у **Settings → Source Lists**.
3. Натисніть **+** та вставте посилання на ваш GitHub Pages або Raw-індекс:
   - Через GitHub Pages (рекомендовано):
     ```text
     https://mlynni.github.io/mhaid/index.min.json
     ```
   - Або через веб-сторінку `https://mlynni.github.io/mhaid/` кнопкою **Add Repository**.
   - Альтернативно через Raw GitHub (гілка `gh-pages`):
     ```text
     https://raw.githubusercontent.com/mlynni/mhaid/gh-pages/index.min.json
     ```

> **Примітка щодо Comix (`en.comix`):** потребує Aidoku **0.8.4+**. Якщо сайт повертає перевірку WAF/Captcha, відкрийте в додатку **Settings екстеншена Comix → Verify Captcha**.

## Автоматичне оновлення з Mihon (GitHub Actions)

У репозиторії налаштовано повністю автоматичний пайплайн синхронізації [`.github/workflows/sync-upstream.yaml`](.github/workflows/sync-upstream.yaml) + [`scripts/sync-upstream.mjs`](scripts/sync-upstream.mjs):

- **Розклад:** запускається автоматично кожні 12 годин (`0 */12 * * *`), а також доступний вручну через **Actions → Sync Upstream Mihon Sources → Run workflow**.
- **Що робить автоматично:**
  1. Порівнює поточні коміти `lastMihonCommit` із останніми змінами в `keiyoushi/extensions-source` (та `Aidoku-Community/sources`).
  2. Якщо для якогось екстеншена в Mihon змінились файли:
     - Оновлює `sources/<id>/MIHON_SOURCE.md`.
     - Автоматично синхронізує нові домени/дзеркала (`baseUrl` / `mirrors`) у `res/source.json`, Rust-коді та `extensions.json`.
     - За наявності секрету `GEMINI_API_KEY` у **GitHub Repository Settings → Secrets and variables → Actions** передає Kotlin `git diff` у Gemini (`gemini-2.5-pro`), автоматично переписує відповідний Rust-код, перевіряє компіляцію (`cargo check --target wasm32-unknown-unknown`) і відкочує патч, якщо він не компілюється.
  3. Піднімає `version` (+1), компілює нові `.aix` через `aidoku package`, оновлює `index.min.json` у `gh-pages` і фіксує зміни в `main`.

### Як увімкнути ШІ-переписування складних Kotlin-змін у GitHub Actions (опційно)
1. Отримайте API-ключ у Google AI Studio (`https://aistudio.google.com/apikey`).
2. У репозиторії GitHub перейдіть у **Settings → Secrets and variables → Actions → New repository secret**.
3. Створіть секрет з назвою **`GEMINI_API_KEY`** і вставте ваш ключ.

## Як швидко додати новий екстеншен з Mihon вручну

```bash
# 1. Переглянути поточний список
node scripts/mihon-to-aidoku.mjs --list

# 2. Додати нове джерело за назвою папки в keiyoushi/extensions-source
node scripts/mihon-to-aidoku.mjs <extension-name>

# 3. За потреби перевірити локально або пересинхронізувати документацію
node scripts/mihon-to-aidoku.mjs --sync-docs
node scripts/build-repo.mjs --verify
```

## Структура репозиторію

- `sources/` — вихідний код кожного з 17 екстеншенів на Rust (`aidoku-rs`) + `MIHON_SOURCE.md`.
- `templates/` — спільні Rust-шаблони (`mangabox`, `madara`).
- `public/` — зібрані `.aix` пакети, PNG-іконки, `index.min.json`, `index.json` та веб-інтерфейс.
- `scripts/mihon-to-aidoku.mjs` — CLI-конвертер для швидкого додавання нових Mihon-екстеншенів.
- `scripts/sync-upstream.mjs` — автоматичний трекер і синхронізатор змін із `keiyoushi/extensions-source` та `Aidoku-Community/sources`.
- `scripts/build-repo.mjs` — збирач та валідатор `public/index.min.json` і `.aix` архівів (`--verify`).
- `extensions.json` — центральний реєстр активних екстеншенів і зафіксованих апстрім-комітів.
- `.github/workflows/build.yaml` — CI/CD для компіляції при push у `main`.
- `.github/workflows/sync-upstream.yaml` — автоматичний cron-воркфлоу синхронізації змін із Mihon.
