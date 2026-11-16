# MHAID — Aidoku-екстеншени з Mihon

Персональний репозиторій розширень (джерел) для **[Aidoku](https://aidoku.app)** (iOS / iPadOS, версія 0.7+, для Comix — 0.8.4+), портованих із розширень Mihon (`keiyoushi/extensions-source`) у формат Rust/WebAssembly (`aidoku-rs`).

## Підтримувані екстеншени

| Джерело | ID в Aidoku | Мови | Версія | Мін. версія Aidoku | Базовий URL |
|---|---|---|---:|---:|---|
| **Weeb Central** | `en.weebcentral` | `en` | v9 | `0.7.1` | https://weebcentral.com |
| **Comix** | `en.comix` | `en` | v28 | `0.8.4` | https://comix.to |
| **MangaDex** | `multi.mangadex` | 54 мови (включно з `uk`, `en`) | v16 | `0.7.1` | https://mangadex.org |
| **MangaFire** | `multi.mangafire` | `en`, `es`, `es-419`, `fr`, `ja`, `pt`, `pt-BR` | v9 | `0.7.1` | https://mangafire.to |

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

## Як швидко додати новий екстеншен з Mihon

У репозиторії є готовий конвертер/генератор [`scripts/mihon-to-aidoku.mjs`](scripts/mihon-to-aidoku.mjs) (працює на Node.js без сторонніх npm-залежностей):

```bash
# 1. Переглянути поточний список
node scripts/mihon-to-aidoku.mjs --list

# 2. Додати нове джерело за назвою папки в keiyoushi/extensions-source
node scripts/mihon-to-aidoku.mjs <extension-name>
# Наприклад:
node scripts/mihon-to-aidoku.mjs bato --lang all
node scripts/mihon-to-aidoku.mjs asurascans --lang en

# 3. За потреби пересобрати index.min.json і перевірити цілісність .aix/.png
node scripts/build-repo.mjs
```

### Як працює `mihon-to-aidoku.mjs`

1. Знаходить та викачує код потрібного екстеншена з `keiyoushi/extensions-source` (парсить як сучасний `build.gradle.kts` / `KeiSource`, так і класичний `build.gradle` + усі `.kt` файли).
2. Витягує `baseUrl`, заголовки, API-ендпоінти, CSS-селектори, фільтри, іконку та шаблон (`madara`, `mangathemesia`, `mangabox`, `iken`, `keyoapp`, `gigaviewer` тощо).
3. Якщо сумісна Rust-реалізація вже є в `Aidoku-Community/sources`, автоматично імпортує її крейт у `sources/<id>/`, потрібні `templates/*` та готовий `.aix` у `public/sources/`.
4. Якщо готової реалізації ще немає, генерує готовий Rust-крейт `sources/<lang>.<name>/` із `Cargo.toml`, `res/source.json`, `res/icon.png`, стартовим `src/lib.rs` та `MIHON_SOURCE.md` із повною картою селекторів і ендпоінтів Mihon.
5. Після `git push` у `main` GitHub Actions (`.github/workflows/build.yaml`) автоматично компілює Rust у WebAssembly (`wasm32-unknown-unknown`) через `aidoku-cli`, упаковує `.aix` та публікує оновлений `index.min.json` у гілку `gh-pages`.

## Структура репозиторію

- `sources/` — вихідний код кожного екстеншена на Rust (`aidoku-rs`) + `MIHON_SOURCE.md` з деталями вихідного Mihon-джерела.
- `public/` — зібрані `.aix` пакети, PNG-іконки, `index.min.json`, `index.json` та веб-інтерфейс.
- `scripts/mihon-to-aidoku.mjs` — CLI-інструмент для швидкого перенесення нових Mihon-екстеншенів.
- `scripts/build-repo.mjs` — збирач та валідатор `public/index.min.json` і `.aix` архівів (`--verify`).
- `extensions.json` — центральний реєстр активних екстеншенів.
- `.github/workflows/build.yaml` — CI/CD для компіляції та автодеплою в `gh-pages`.
