# Mlyn Aidoku Sources

<sub>[English](README.md) · Українська</sub>

Персональний репозиторій джерел для [Aidoku](https://github.com/Aidoku/Aidoku), портованих із розширень Mihon (`keiyoushi/extensions-source`) та адаптованих у формат Rust/WebAssembly (`.aix`). Список сфокусований виключно на 17 екстеншенах для щоденного читання, без розпилення на сотні зайвих джерел.

## Використання

На пристрої з Aidoku (`0.7+`, для Comix та MangaDotNet v20 — `0.8.4+`, для Mangadotnet v10 — `0.9.0+`) відкрийте [https://mlynni.github.io/mlyn-aidoku-sources/](https://mlynni.github.io/mlyn-aidoku-sources/) і натисніть кнопку **Add Sources to Aidoku**, щоб додати список напряму в додаток.

Або перейдіть у **Settings → Source Lists** у додатку Aidoku та вставте:

```text
https://mlynni.github.io/mlyn-aidoku-sources/index.min.json
```

Резервне raw-дзеркало (`gh-pages`):

```text
https://raw.githubusercontent.com/mlynni/mlyn-aidoku-sources/gh-pages/index.min.json
```

## Джерела (17)

| Джерело | ID | Версія | Мови | Мін. Aidoku | Базовий URL | Примітки |
|---|---|---:|---|---:|---|---|
| Atsumaru | `en.atsumaru` | `v12` | `en` | `0.7.1` | https://atsu.moe | Пряме JSON API |
| Comix | `en.comix` | `v31` | `en` | `0.8.4` | https://comix.to | Переписано з Mihon v64: WebView SPA-харвестери, вибір дзеркал (`comix.to` / `comix.ws`), дескремблер |
| Kagane | `multi.kagane` | `v26` | 15 мов | `0.7.1` | https://kagane.moe | Манхва та манґа |
| MANGA Plus | `multi.mangaplus` | `v6` | 9 мов | `0.7.2` | https://mangaplus.shueisha.co.jp | Офіційні релізи Shueisha |
| Mangaball | `multi.mangaball` | `v14` | 34 мови (incl. `uk`) | `0.7.1` | https://mangaball.com | Мульти-CDN фолбек постерів і `.512.jpg` |
| MangaBat | `en.mangabat` | `v9` | `en` | `0.7.1` | https://www.mangabats.com | Шаблон `mangabox` |
| MangaDex | `multi.mangadex` | `v16` | 54 мови (incl. `uk`, `en`) | `0.7.1` | https://mangadex.org | Публічне API |
| MangaDotNet | `en.mangadot` | `v20` | 43 мови (incl. `uk`) | `0.8.3` | https://mangadot.net | Групування глав під одну команду |
| Mangadotnet | `multi.mangadotnet` | `v10` | 43 мови (incl. `uk`) | `0.9.0` | https://mangadot.net | Варіант під Aidoku 0.9.0+ |
| MangaFire | `multi.mangafire` | `v9` | 7 мов | `0.7.1` | https://mangafire.to | Мультимовний каталог |
| Mangago | `en.mangago` | `v4` | `en` | `0.7.1` | https://www.mangago.me | 18+ / NSFW |
| MangaKakalot | `en.mangakakalot` | `v6` | `en` | `0.7.1` | https://www.mangakakalot.gg | Шаблон `mangabox` |
| MangaNato | `en.manganato` | `v8` | `en` | `0.7.1` | https://www.manganato.gg | Шаблон `mangabox` |
| MangaNelo | `en.manganelo` | `v4` | `en` | `0.7.1` | https://www.nelomanga.net | Шаблон `mangabox` |
| MangaPill | `en.mangapill` | `v4` | `en` | `0.7.1` | https://mangapill.com | Порт на Rust із Mihon Kotlin |
| Toonily | `en.toonily` | `v10` | `en` | `0.7.1` | https://toonily.com | Шаблон `madara` (18+) |
| Weeb Central | `en.weebcentral` | `v9` | `en` | `0.7.1` | https://weebcentral.com | Пошук і фільтри |

> **Примітка для Comix:** джерело переписано з нуля за архітектурою Mihon без реверс-інжинірингу API-підписів, тому воно не скидає верифікацію кожні кілька хвилин. У **Comix → Settings → Mirror** можна перемикатись між `https://comix.to` та `https://comix.ws`; пункт **Open Site Check in WebView** потрібен лише якщо сайт сам запросив перевірку.

## Підтримка та автоматизація

Автоматичний воркфлоу ([`.github/workflows/sync-upstream.yaml`](.github/workflows/sync-upstream.yaml)) кожні 12 годин перевіряє апстрім-коміти через [`scripts/sync-upstream.mjs`](scripts/sync-upstream.mjs), оновлює метадані й Rust-код, компілює `.aix` через `aidoku package`, перевіряє цілісність ([`scripts/build-repo.mjs`](scripts/build-repo.mjs)) та деплоїть у `gh-pages`.

```bash
# Список активних джерел і зафіксованих комітів
node scripts/mihon-to-aidoku.mjs --list

# Додати нове джерело з keiyoushi/extensions-source
node scripts/mihon-to-aidoku.mjs <extension-name>

# Суха перевірка змін апстріму
node scripts/sync-upstream.mjs --dry-run

# Валідація індексу та .aix пакетів
node scripts/build-repo.mjs --verify
```

Щоб увімкнути ШІ-адаптацію складних змін Kotlin → Rust у GitHub Actions, додайте секрет **`GEMINI_API_KEY`** у **Settings → Secrets and variables → Actions**.

## Подяки

Репозиторій спирається на відкриті розробки:

- **[Aidoku](https://github.com/Aidoku/Aidoku)** & **[aidoku-rs](https://github.com/Aidoku/aidoku-rs)** — додаток Aidoku і Rust WebAssembly SDK.
- **[Aidoku Community Sources](https://github.com/Aidoku-Community/sources)** — відкриті Rust-імплементації джерел та спільні шаблони (`mangabox`, `madara`).
- **[Keiyoushi / Mihon Extensions](https://github.com/keiyoushi/extensions-source)** — апстрім Kotlin-розширень Mihon, селектори та дзеркала.

## Застереження

Цей проєкт не має афіліації з провайдерами контенту або додатком Aidoku. Усі назви й матеріали належать їхнім правовласникам. За запитом правовласника будь-яке джерело буде видалено зі списку.
