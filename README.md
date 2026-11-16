<div align="center">

<img src="https://capsule-render.vercel.app/api?type=waving&color=0:ff3b69,100:7c5cff&height=180&section=header&text=Mlyn%20Aidoku%20Sources&fontSize=54&fontColor=ffffff&fontAlignY=38&desc=Curated%20Mihon%20%26%20Aidoku%20Extensions%20for%20Personal%20Reading&descSize=16&descAlignY=62" alt="Mlyn Aidoku Sources" width="100%">

<img alt="Platform: iOS / iPadOS, Aidoku 0.8.4+" src="https://img.shields.io/badge/iOS%20%2F%20iPadOS-Aidoku%200.8.4%2B-ff3b69?style=for-the-badge">
<img alt="Sources: 17" src="https://img.shields.io/badge/sources-17-7c5cff?style=for-the-badge">
<img alt="Auto-Sync: Every 12h" src="https://img.shields.io/badge/auto--sync-every%2012h-1f2937?style=for-the-badge">

<img src="assets/spacer.png" width="100%" height="24" alt="">

<a href="https://aidoku.app/add-source-list/?url=https://mlynni.github.io/mhaid/index.min.json">
  <img alt="Add Mlyn Aidoku Sources to Aidoku" src="assets/add-to-aidoku.svg" width="300">
</a>

<img src="assets/spacer.png" width="100%" height="24" alt="">

Натисніть кнопку на iPhone або iPad, щоб додати список одразу в **Aidoku**, або відкрийте **Settings → Source Lists → +** та вставте:

<code>https://mlynni.github.io/mhaid/index.min.json</code>

<sub>Веб-сторінка списку: <a href="https://mlynni.github.io/mhaid/">mlynni.github.io/mhaid</a> &nbsp;·&nbsp; Резервний raw-URL: <code>https://raw.githubusercontent.com/mlynni/mhaid/gh-pages/index.min.json</code></sub>

</div>

---

## Про цей репозиторій

**[Aidoku](https://aidoku.app)** — безкоштовний open-source рідер манґи, манхви та маньхуа для iOS і iPadOS.

**Mlyn Aidoku Sources** — це **персональний, вузькосфокусований список джерел**, створений виключно під ті **17 екстеншенів**, якими я реально користуюсь для читання. Замість того щоб розпилятися на сотні сторонніх сайтів, репозиторій тримає у фокусі лише обраний перелік і оновлюється тоді, коли змінюється мій список читання або виходять апдейти на апстрімах.

### Що саме відбувається в цьому репозиторії

1. **Порти з Mihon та адаптації під `aidoku-rs` (Rust → WebAssembly `.aix`):**
   - Кожен екстеншен у [`sources/`](sources/) — це окремий незалежний Rust-крейт (`Cargo.toml`), який компілюється в WebAssembly-пакет `.aix` для Aidoku.
   - Для кожного джерела поруч зберігається файл `MIHON_SOURCE.md` з актуальними CSS-селекторами, API-ендпоінтами, фільтрами, мовами та дзеркалами з Mihon (`keiyoushi/extensions-source`).
   - Окремі екстеншени мають власні виправлення (наприклад, кастомний порт `MangaPill` із Mihon, завантаження обкладинок через WebView після капчі в `Comix`, мульти-CDN резолв та Hotlink-обхід для постерів `Mangaball`).
2. **Автоматична синхронізація та збірка через GitHub Actions:**
   - Воркфлоу [`.github/workflows/sync-upstream.yaml`](.github/workflows/sync-upstream.yaml) запускається кожні **12 годин** і перевіряє нові коміти в апстрімах для всіх 17 відстежуваних джерел.
   - При змінах [`scripts/sync-upstream.mjs`](scripts/sync-upstream.mjs) оновлює `MIHON_SOURCE.md`, дзеркала (`baseURL` / `mirrors`) та Rust-код, піднімає версію, компілює нові `.aix` через `aidoku package`, перевіряє цілісність ([`scripts/build-repo.mjs`](scripts/build-repo.mjs)) і публікує оновлений `index.min.json` у гілку `gh-pages`.

---

## Подяки та першоджерела

Цей репозиторій не є «офіційним списком спільноти» і не привласнює роботу інших авторів. Він побудований завдяки праці кількох чудових open-source проєктів:

- **[Aidoku](https://github.com/Aidoku/Aidoku)** & **[aidoku-rs](https://github.com/Aidoku/aidoku-rs)** — сам додаток Aidoku та Rust WebAssembly SDK для джерел.
- **[Aidoku Community Sources](https://github.com/Aidoku-Community/sources)** — відкриті Rust-імплементації більшості базових джерел і спільні шаблони (`mangabox`, `madara`).
- **[Keiyoushi / Mihon Extensions](https://github.com/keiyoushi/extensions-source)** — Kotlin-розширення Mihon, з яких ми беремо селектори, домени, налаштування та логіку оновлень для портування й автосинхронізації.
- **[Yomu Aidoku Sources (Smexhy)](https://github.com/Smexhy/yomu-aidoku-sources)** — натхнення для оформлення та структури цього `README`, а також орієнтир для покращень у джерелах (зокрема обхід перевірок `Comix`, робота з `Mangaball`, `MangaDotNet`, `Kagane` та `Atsumaru`).

Уся повага й авторство базових проєктів і платформ належить їхнім оригінальним мейнтейнерам і власникам сайтів.

---

## Сфокусований список (17 джерел)

| Джерело | ID | Версія | Мови | Мін. Aidoku | Сайт і примітки |
|---|---|---:|---|---:|---|
| <img src="assets/icons/en.atsumaru.png" width="22" align="top"> **Atsumaru** | `en.atsumaru` | `v12` | English | `0.7.1` | [atsu.moe](https://atsu.moe) · Швидке JSON API без зайвого прогріву |
| <img src="assets/icons/en.comix.png" width="22" align="top"> **Comix** | `en.comix` | `v29` | English | `0.8.4` | [comix.to](https://comix.to) · Величезна бібліотека + WebView-резолв постерів після Verify Captcha |
| <img src="assets/icons/multi.kagane.png" width="22" align="top"> **Kagane** | `multi.kagane` | `v26` | 15 мов | `0.7.1` | [kagane.moe](https://kagane.moe) · Переважно манхва, бейджі груп і джерел |
| <img src="assets/icons/multi.mangaplus.png" width="22" align="top"> **MANGA Plus** | `multi.mangaplus` | `v6` | 9 мов | `0.7.2` | [mangaplus.shueisha.co.jp](https://mangaplus.shueisha.co.jp) · Офіційні релізи Shueisha |
| <img src="assets/icons/multi.mangaball.png" width="22" align="top"> **Mangaball** | `multi.mangaball` | `v13` | 34 мови (включно з `uk`) | `0.7.1` | [mangaball.com](https://mangaball.com) · Багатомовне покриття + fallback-завантаження постерів із різних CDN |
| <img src="assets/icons/en.mangabat.png" width="22" align="top"> **MangaBat** | `en.mangabat` | `v9` | English | `0.7.1` | [mangabats.com](https://www.mangabats.com) · Шаблон `mangabox` |
| <img src="assets/icons/multi.mangadex.png" width="22" align="top"> **MangaDex** | `multi.mangadex` | `v16` | 54 мови (включно з `uk`, `en`) | `0.7.1` | [mangadex.org](https://mangadex.org) · Офіційне публічне API, підтримка української |
| <img src="assets/icons/en.mangadot.png" width="22" align="top"> **MangaDotNet** | `en.mangadot` | `v20` | 43 мови (включно з `uk`) | `0.8.3` | [mangadot.net](https://mangadot.net) · Групування перекладу під одну команду |
| <img src="assets/icons/multi.mangadotnet.png" width="22" align="top"> **Mangadotnet** | `multi.mangadotnet` | `v10` | 43 мови (включно з `uk`) | `0.9.0` | [mangadot.net](https://mangadot.net) · Варіант під Aidoku 0.9.0+ |
| <img src="assets/icons/multi.mangafire.png" width="22" align="top"> **MangaFire** | `multi.mangafire` | `v9` | 7 мов (`en`, `es`, `fr`, `ja`, `pt`…) | `0.7.1` | [mangafire.to](https://mangafire.to) · Мультимовний каталог із фільтрами |
| <img src="assets/icons/en.mangago.png" width="22" align="top"> **Mangago** <img alt="18+" src="https://img.shields.io/badge/18+-ff3b69?style=flat-square" align="top"> | `en.mangago` | `v4` | English | `0.7.1` | [mangago.me](https://www.mangago.me) · Повний каталог сайту (NSFW) |
| <img src="assets/icons/en.mangakakalot.png" width="22" align="top"> **MangaKakalot** | `en.mangakakalot` | `v6` | English | `0.7.1` | [mangakakalot.gg](https://www.mangakakalot.gg) · Шаблон `mangabox` |
| <img src="assets/icons/en.manganato.png" width="22" align="top"> **MangaNato** | `en.manganato` | `v8` | English | `0.7.1` | [manganato.gg](https://www.manganato.gg) · Шаблон `mangabox` |
| <img src="assets/icons/en.manganelo.png" width="22" align="top"> **MangaNelo** | `en.manganelo` | `v4` | English | `0.7.1` | [nelomanga.net](https://www.nelomanga.net) · Шаблон `mangabox` |
| <img src="assets/icons/en.mangapill.png" width="22" align="top"> **MangaPill** | `en.mangapill` | `v4` | English | `0.7.1` | [mangapill.com](https://mangapill.com) · Кастомний порт із Mihon Kotlin під Aidoku WASM |
| <img src="assets/icons/en.toonily.png" width="22" align="top"> **Toonily** <img alt="18+" src="https://img.shields.io/badge/18+-ff3b69?style=flat-square" align="top"> | `en.toonily` | `v10` | English | `0.7.1` | [toonily.com](https://toonily.com) · Шаблон `madara` (NSFW) |
| <img src="assets/icons/en.weebcentral.png" width="22" align="top"> **Weeb Central** | `en.weebcentral` | `v9` | English | `0.7.1` | [weebcentral.com](https://weebcentral.com) · Чистий пошук і фільтри |

<sub>Для мультимовних джерел оберіть потрібні мови (наприклад, `uk` і `en`) у налаштуваннях відповідного екстеншена всередині Aidoku.</sub>

---

## Особливості ключових джерел

<h3><img src="assets/icons/en.comix.png" width="24" align="top">&nbsp; Comix</h3>

<sub><code>en.comix</code> &nbsp;·&nbsp; <a href="https://comix.to">comix.to</a> &nbsp;·&nbsp; <strong>потрібен Aidoku 0.8.4+</strong></sub>

- **Веб-перевірка та постери.** Після проходження перевірки (**Settings екстеншена Comix → Verify Captcha**) WebView автоматично розпізнає та підвантажує навіть ті постери й зображення сторінок, де CDN вимагає браузерні куки/перевірку.
- **Дедуплікація та розшифрування.** Пріоритет офіційним релізам і кращим групам, підтримка дескремблінгу захищених сторінок.

<details>
<summary><strong>Деталі про Comix і роботу з Verify Captcha</strong></summary>

1. Відкрийте в Aidoku **Browse → Comix → Settings → Verify Captcha** і пройдіть перевірку у вікні.
2. Завантажені обкладинки автоматично підтягуються через WebView (`resolve_covers_in_webview` + `CoverImageProcessor`), навіть якщо прямі HTTP-запити до CDN тимчасово обмежені WAF/Cloudflare.
3. Якщо сесія застаріла — повторіть **Verify Captcha** або оберіть **Clear Cache**.

</details>

<h3><img src="assets/icons/multi.mangaball.png" width="24" align="top">&nbsp; Mangaball</h3>

<sub><code>multi.mangaball</code> &nbsp;·&nbsp; <a href="https://mangaball.com">mangaball.com</a> &nbsp;·&nbsp; <strong>оновлено у v13</strong></sub>

- **Фікс завантаження постерів (v13).**
  - Відфільтровані мертві зовнішні дзеркала (`hentaicdn.org` / `hencover`), через які обкладинки лишались порожніми.
  - Нормалізовані відносні шляхи `/media/mbim/...`, `/covers/...`, Windows-слеші та відносні MangaDex-обкладинки, а також додано резервний `background`.
  - Виправлено `Referer`-заголовки в `ImageRequestProvider` для зовнішніх CDN (зокрема `uploads.mangadex.org` і `mangaupdates.com`) та додано `CoverImageProcessor` fallback на випадок відмови окремого дзеркала.

<h3><img src="assets/icons/en.mangadot.png" width="24" align="top">&nbsp; MangaDotNet та Mangadotnet</h3>

<sub><code>en.mangadot</code> (v20, Aidoku 0.8.3+) &nbsp;·&nbsp; <code>multi.mangadotnet</code> (v10, Aidoku 0.9.0+) &nbsp;·&nbsp; <a href="https://mangadot.net">mangadot.net</a></sub>

- У списку залишено обидві версії під різні версії додатка Aidoku (0.8.x і 0.9.0+), як у вашому наборі встановлених джерел.

<h3><img src="assets/icons/en.mangapill.png" width="24" align="top">&nbsp; MangaPill</h3>

<sub><code>en.mangapill</code> &nbsp;·&nbsp; <a href="https://mangapill.com">mangapill.com</a></sub>

- Прямий порт із Mihon (`src/en/mangapill`) на чистий Rust (`aidoku-rs`), включно з пошуком, фільтрами за типом/статусом/жанрами, парсингом глав і сторінок.

---

## Якщо джерело зависає при завантаженні

> 💡 **Тримайте Aidoku оновленим до останньої версії.** Більшість помилок `403` або нескінченних спінерів пов'язані з Cloudflare-обробкою на рівні самого додатка, а не джерела.

Якщо після оновлення джерело все одно підвисає:

1. Відкрийте **Browse**, перейдіть у джерело та натисніть **три крапки → Settings**.
2. Прокрутіть униз і натисніть **Clear Cache** (для Comix також за потреби **Verify Captcha**).
3. Повністю **перезапустіть Aidoku** (змахніть із панелі задач і відкрийте знову).

---

## Як оновлювати та змінювати цей список

Усі 17 екстеншенів перелічені у [`extensions.json`](extensions.json). Щоб з часом додати або оновити джерело з Mihon:

```bash
# Переглянути активні джерела та зафіксовані апстрім-коміти
node scripts/mihon-to-aidoku.mjs --list

# Додати нове джерело за назвою папки з keiyoushi/extensions-source
node scripts/mihon-to-aidoku.mjs <extension-name>

# Вручну перевірити апстрім-оновлення Mihon / Aidoku
node scripts/sync-upstream.mjs --dry-run

# Перевірити цілісність index.min.json та всіх .aix архівів
node scripts/build-repo.mjs --verify
```

### Як увімкнути ШІ-переписування складних Kotlin-змін при автосинхронізації (опційно)
1. Отримайте API-ключ у [Google AI Studio](https://aistudio.google.com/apikey).
2. У репозиторії GitHub відкрийте **Settings → Secrets and variables → Actions → New repository secret**.
3. Створіть секрет **`GEMINI_API_KEY`**, і воркфлоу автоматично адаптуватиме навіть нестандартні зміни селекторів/API з Kotlin у Rust із перевіркою `cargo check` перед публікацією.

---

<div align="center">
<img src="https://capsule-render.vercel.app/api?type=waving&color=0:7c5cff,100:ff3b69&height=90&section=footer" alt="" width="100%">
</div>
