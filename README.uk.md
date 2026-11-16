<div align="center">

<sub><a href="README.md">English</a> &nbsp;·&nbsp; <strong>Українська</strong></sub>

<br><br>

<img src="assets/header-banner.svg" alt="Mlyn Aidoku Sources" width="100%">

<br><br>

<img alt="Platform: iOS / iPadOS, Aidoku 0.8.4+" src="https://img.shields.io/badge/iOS%20%2F%20iPadOS-Aidoku%200.8.4%2B-0d9488?style=for-the-badge&labelColor=0f172a">
<img alt="Curated Sources: 17" src="https://img.shields.io/badge/Curated%20Sources-17-0284c7?style=for-the-badge&labelColor=0f172a">
<img alt="Auto-Sync: Every 12h" src="https://img.shields.io/badge/Auto--Sync-Every%2012h-4f46e5?style=for-the-badge&labelColor=0f172a">

<img src="assets/spacer.png" width="100%" height="20" alt="">

<a href="https://aidoku.app/add-source-list/?url=https://mlynni.github.io/mhaid/index.min.json">
  <img alt="Add Sources to Aidoku" src="assets/add-to-aidoku.svg" width="310">
</a>

<img src="assets/spacer.png" width="100%" height="18" alt="">

Натисніть кнопку на iPhone або iPad, щоб додати список одразу в **Aidoku**, або відкрийте **Settings → Source Lists → +** та вставте:

<code>https://mlynni.github.io/mhaid/index.min.json</code>

<sub>Веб-каталог: <a href="https://mlynni.github.io/mhaid/">mlynni.github.io/mhaid</a> &nbsp;·&nbsp; Raw-дзеркало: <code>https://raw.githubusercontent.com/mlynni/mhaid/gh-pages/index.min.json</code></sub>

</div>

---

## Про цей репозиторій

**[Aidoku](https://aidoku.app)** — безкоштовний open-source рідер манґи, манхви та маньхуа для iOS і iPadOS.

**Mlyn Aidoku Sources** — це **персональний, вузькосфокусований список джерел**, зібраний виключно навколо тих **17 екстеншенів**, якими я користуюсь для читання. Замість того щоб розпилятися на сотні сторонніх сайтів, репозиторій тримає у фокусі лише обраний перелік і змінюється разом із моїм списком читання та оновленнями апстрімів.

### Що саме відбувається в цьому репозиторії

1. **Порти з Mihon та окремі Rust/WebAssembly крейти (`.aix`):**
   - Кожне джерело у [`sources/`](sources/) — це незалежний Rust-крейт (`aidoku-rs`), що компілюється у WebAssembly-пакет `.aix` для Aidoku.
   - Поруч із кожним джерелом зберігається файл `MIHON_SOURCE.md` з актуальними CSS-селекторами, API-ендпоінтами, фільтрами, мовами та дзеркалами з Mihon (`keiyoushi/extensions-source`).
   - Там, де це потрібно, підтримуються цільові фікси — зокрема WebView-завантаження постерів після капчі в **Comix**, нормалізація мульти-CDN та обхід hotlink-захисту постерів у **Mangaball**, а також нативний Rust-порт **MangaPill** з Mihon.
2. **Автоматична синхронізація та збірка через GitHub Actions:**
   - Кожні **12 годин** воркфлоу [`.github/workflows/sync-upstream.yaml`](.github/workflows/sync-upstream.yaml) + [`scripts/sync-upstream.mjs`](scripts/sync-upstream.mjs) перевіряють апстрім-коміти для всіх 17 відстежуваних джерел.
   - Коли в апстрімах змінюються домени чи логіка, пайплайн оновлює документацію та Rust-код, піднімає версію, компілює нові `.aix` через `aidoku package`, перевіряє цілісність ([`scripts/build-repo.mjs`](scripts/build-repo.mjs)) і публікує оновлений `index.min.json` у гілку `gh-pages`.

---

## Подяки та першоджерела

Цей репозиторій не претендує на статус офіційного списку спільноти і побудований завдяки праці відкритих проєктів:

- **[Aidoku](https://github.com/Aidoku/Aidoku)** & **[aidoku-rs](https://github.com/Aidoku/aidoku-rs)** — сам додаток Aidoku та Rust WebAssembly SDK для джерел.
- **[Aidoku Community Sources](https://github.com/Aidoku-Community/sources)** — відкриті Rust-імплементації базових джерел і спільні шаблони (`mangabox`, `madara`).
- **[Keiyoushi / Mihon Extensions](https://github.com/keiyoushi/extensions-source)** — апстрім Kotlin-розширень Mihon, з яких беруться селектори, дзеркала та логіка оновлень для нашого конвертера й автосинхронізації.

Усі назви сайтів, логотипи та вихідний код апстрімів належать їхнім оригінальним авторам і власникам.

---

## Сфокусований список (17 джерел)

| Джерело | ID | Версія | Мови | Мін. Aidoku | Примітки |
|---|---|---:|---|---:|---|
| <img src="assets/icons/en.atsumaru.png" width="22" align="top"> **Atsumaru** | `en.atsumaru` | `v12` | English | `0.7.1` | [atsu.moe](https://atsu.moe) · Швидке JSON API без зайвого прогріву |
| <img src="assets/icons/en.comix.png" width="22" align="top"> **Comix** | `en.comix` | `v29` | English | `0.8.4` | [comix.to](https://comix.to) · Велика бібліотека + WebView-резолв постерів після Verify Captcha |
| <img src="assets/icons/multi.kagane.png" width="22" align="top"> **Kagane** | `multi.kagane` | `v26` | 15 мов | `0.7.1` | [kagane.moe](https://kagane.moe) · Переважно манхва з бейджами сканлейт-груп |
| <img src="assets/icons/multi.mangaplus.png" width="22" align="top"> **MANGA Plus** | `multi.mangaplus` | `v6` | 9 мов | `0.7.2` | [mangaplus.shueisha.co.jp](https://mangaplus.shueisha.co.jp) · Офіційні релізи Shueisha |
| <img src="assets/icons/multi.mangaball.png" width="22" align="top"> **Mangaball** | `multi.mangaball` | `v13` | 34 мови (включно з `uk`) | `0.7.1` | [mangaball.com](https://mangaball.com) · Багатомовне покриття + мульти-CDN фолбек постерів |
| <img src="assets/icons/en.mangabat.png" width="22" align="top"> **MangaBat** | `en.mangabat` | `v9` | English | `0.7.1` | [mangabats.com](https://www.mangabats.com) · Шаблон `mangabox` |
| <img src="assets/icons/multi.mangadex.png" width="22" align="top"> **MangaDex** | `multi.mangadex` | `v16` | 54 мови (включно з `uk`, `en`) | `0.7.1` | [mangadex.org](https://mangadex.org) · Публічне API, підтримка української |
| <img src="assets/icons/en.mangadot.png" width="22" align="top"> **MangaDotNet** | `en.mangadot` | `v20` | 43 мови (включно з `uk`) | `0.8.3` | [mangadot.net](https://mangadot.net) · Групування перекладу під одну команду |
| <img src="assets/icons/multi.mangadotnet.png" width="22" align="top"> **Mangadotnet** | `multi.mangadotnet` | `v10` | 43 мови (включно з `uk`) | `0.9.0` | [mangadot.net](https://mangadot.net) · Варіант під Aidoku 0.9.0+ |
| <img src="assets/icons/multi.mangafire.png" width="22" align="top"> **MangaFire** | `multi.mangafire` | `v9` | 7 мов (`en`, `es`, `fr`, `ja`, `pt`…) | `0.7.1` | [mangafire.to](https://mangafire.to) · Мультимовний каталог із фільтрами |
| <img src="assets/icons/en.mangago.png" width="22" align="top"> **Mangago** <img alt="18+" src="https://img.shields.io/badge/18+-e11d48?style=flat-square" align="top"> | `en.mangago` | `v4` | English | `0.7.1` | [mangago.me](https://www.mangago.me) · Повний каталог сайту (NSFW) |
| <img src="assets/icons/en.mangakakalot.png" width="22" align="top"> **MangaKakalot** | `en.mangakakalot` | `v6` | English | `0.7.1` | [mangakakalot.gg](https://www.mangakakalot.gg) · Шаблон `mangabox` |
| <img src="assets/icons/en.manganato.png" width="22" align="top"> **MangaNato** | `en.manganato` | `v8` | English | `0.7.1` | [manganato.gg](https://www.manganato.gg) · Шаблон `mangabox` |
| <img src="assets/icons/en.manganelo.png" width="22" align="top"> **MangaNelo** | `en.manganelo` | `v4` | English | `0.7.1` | [nelomanga.net](https://www.nelomanga.net) · Шаблон `mangabox` |
| <img src="assets/icons/en.mangapill.png" width="22" align="top"> **MangaPill** | `en.mangapill` | `v4` | English | `0.7.1` | [mangapill.com](https://mangapill.com) · Кастомний Rust-порт із Mihon Kotlin |
| <img src="assets/icons/en.toonily.png" width="22" align="top"> **Toonily** <img alt="18+" src="https://img.shields.io/badge/18+-e11d48?style=flat-square" align="top"> | `en.toonily` | `v10` | English | `0.7.1` | [toonily.com](https://toonily.com) · Шаблон `madara` (NSFW) |
| <img src="assets/icons/en.weebcentral.png" width="22" align="top"> **Weeb Central** | `en.weebcentral` | `v9` | English | `0.7.1` | [weebcentral.com](https://weebcentral.com) · Чистий пошук і фільтри |

<sub>Для мультимовних джерел оберіть потрібні мови (наприклад, `uk` і `en`) у налаштуваннях відповідного екстеншена всередині Aidoku.</sub>

---

## Особливості ключових джерел

<h3><img src="assets/icons/en.comix.png" width="24" align="top">&nbsp; Comix</h3>

<sub><code>en.comix</code> &nbsp;·&nbsp; <a href="https://comix.to">comix.to</a> &nbsp;·&nbsp; <strong>потрібен Aidoku 0.8.4+</strong></sub>

- **Веб-перевірка та постери.** Після проходження перевірки (**Settings екстеншена Comix → Verify Captcha**) WebView автоматично розпізнає та підвантажує навіть ті постери й сторінки, де CDN вимагає браузерні куки.
- **Дедуплікація та розшифрування.** Підтримка дескремблінгу тайлів та пріоритет офіційним/найкращим сканлейт-групам.

<details>
<summary><strong>Кроки перевірки для Comix</strong></summary>

1. Відкрийте в Aidoku **Browse → Comix → Settings → Verify Captcha** і пройдіть перевірку у вікні.
2. Оновіть картку тайлу — обкладинки автоматично підтягнуться через підтверджену WebView-сесію.
3. Якщо сесія застаріла — повторіть **Verify Captcha** або натисніть **Clear Cache**.

</details>

<h3><img src="assets/icons/multi.mangaball.png" width="24" align="top">&nbsp; Mangaball</h3>

<sub><code>multi.mangaball</code> &nbsp;·&nbsp; <a href="https://mangaball.com">mangaball.com</a> &nbsp;·&nbsp; <strong>оновлено у v13</strong></sub>

- **Надійне завантаження постерів (`v13`).**
  - Відфільтровані мертві зовнішні дзеркала (`hentaicdn.org` / `hencover`).
  - Нормалізовані відносні шляхи `/media/...`, `/covers/...`, `background`, Windows-слеші та відносні MangaDex-обкладинки.
  - Виправлено `Referer`-заголовки для зовнішніх CDN (`mangadex.org`, `mangaupdates.com`) і додано резервне перезавантаження в `CoverImageProcessor`.

<h3><img src="assets/icons/en.mangadot.png" width="24" align="top">&nbsp; MangaDotNet та Mangadotnet</h3>

<sub><code>en.mangadot</code> (v20, Aidoku 0.8.3+) &nbsp;·&nbsp; <code>multi.mangadotnet</code> (v10, Aidoku 0.9.0+) &nbsp;·&nbsp; <a href="https://mangadot.net">mangadot.net</a></sub>

- Залишено обидва варіанти для сумісності з Aidoku 0.8.x та 0.9.0+.

<h3><img src="assets/icons/en.mangapill.png" width="24" align="top">&nbsp; MangaPill</h3>

<sub><code>en.mangapill</code> &nbsp;·&nbsp; <a href="https://mangapill.com">mangapill.com</a></sub>

- Нативний Rust-порт із Mihon (`src/en/mangapill`) із пошуком, фільтрами за типом/статусом/жанрами, главами та сторінками.

---

## Якщо джерело зависає при завантаженні

> Тримайте **Aidoku** оновленим до останньої версії — більшість помилок `403` або нескінченних спінерів пов'язані з Cloudflare-обробкою на рівні самого додатка.

Якщо після оновлення джерело все одно підвисає:

1. Відкрийте **Browse**, перейдіть у джерело та натисніть **⋯ → Settings**.
2. Прокрутіть униз і натисніть **Clear Cache** (для Comix також за потреби **Verify Captcha**).
3. Повністю **перезапустіть Aidoku**, щоб скинути застарілі Cloudflare-куки.

---

## Керування списком та CLI-інструменти

Усі активні екстеншени перелічені у [`extensions.json`](extensions.json):

```bash
# Переглянути активні джерела та зафіксовані апстрім-коміти
node scripts/mihon-to-aidoku.mjs --list

# Додати нове джерело за назвою папки з keiyoushi/extensions-source
node scripts/mihon-to-aidoku.mjs <extension-name>

# Перевірити апстрім-оновлення Mihon / Aidoku в режимі dry-run
node scripts/sync-upstream.mjs --dry-run

# Перевірити цілісність index.min.json та всіх .aix архівів
node scripts/build-repo.mjs --verify
```

### Як увімкнути ШІ-переписування складних Kotlin-змін у CI (опційно)
1. Отримайте API-ключ у [Google AI Studio](https://aistudio.google.com/apikey).
2. У репозиторії GitHub відкрийте **Settings → Secrets and variables → Actions → New repository secret**.
3. Створіть секрет **`GEMINI_API_KEY`**, щоб [`scripts/sync-upstream.mjs`](scripts/sync-upstream.mjs) автоматично переносив складні Kotlin-зміни у Rust із перевіркою `cargo check`.

---

<div align="center">
<img src="assets/footer-banner.svg" alt="" width="100%">
</div>
