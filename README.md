<div align="center">

<sub><strong>English</strong> &nbsp;·&nbsp; <a href="README.uk.md">Українська</a></sub>

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

Tap the button on iPhone or iPad to import this list into **Aidoku**, or open **Settings → Source Lists → +** and paste:

<code>https://mlynni.github.io/mhaid/index.min.json</code>

<sub>Web Catalog: <a href="https://mlynni.github.io/mhaid/">mlynni.github.io/mhaid</a> &nbsp;·&nbsp; Raw Mirror: <code>https://raw.githubusercontent.com/mlynni/mhaid/gh-pages/index.min.json</code></sub>

</div>

---

## About This Repository

**[Aidoku](https://aidoku.app)** is a free, open-source manga, manhwa, and manhua reader for iOS and iPadOS.

**Mlyn Aidoku Sources** is a **focused personal source list** curated around the **17 extensions** I actively read with. Rather than trying to maintain hundreds of unused sources, this repository stays tightly scoped to a personal reading list and evolves over time as those sources or reading preferences change.

### How It Works Under the Hood

1. **Mihon Ports & Standalone Rust/WASM Crates (`.aix`):**
   - Each source under [`sources/`](sources/) is an independent Rust crate compiled to WebAssembly (`aidoku-rs`) and packaged into an `.aix` archive.
   - Alongside every crate, a [`MIHON_SOURCE.md`](sources/) reference document tracks extracted selectors, API endpoints, filters, languages, and mirrors from Mihon (`keiyoushi/extensions-source`).
   - Targeted fixes are maintained directly where needed — including post-captcha WebView cover resolution for **Comix**, multi-CDN cover normalization and hotlink referrer fixes for **Mangaball**, and a full Mihon-to-Aidoku Rust port for **MangaPill**.
2. **Automated Upstream Sync & CI/CD Pipeline:**
   - Every **12 hours**, [`.github/workflows/sync-upstream.yaml`](.github/workflows/sync-upstream.yaml) + [`scripts/sync-upstream.mjs`](scripts/sync-upstream.mjs) check upstream commits across all tracked sources.
   - Whenever upstream mirrors, selectors, or source code change, the workflow updates docs and Rust code, bumps source versions, compiles `.aix` binaries via `aidoku package`, verifies integrity via [`scripts/build-repo.mjs`](scripts/build-repo.mjs), and deploys `index.min.json` to `gh-pages`.

---

## Acknowledgements & Upstream Credits

This repository is a personal curation and automation layer built on top of open-source work by the manga reader community. Full credit and appreciation belong to the original authors and maintainers:

- **[Aidoku](https://github.com/Aidoku/Aidoku)** & **[aidoku-rs](https://github.com/Aidoku/aidoku-rs)** — the Aidoku app, source runtime, and Rust WebAssembly SDK.
- **[Aidoku Community Sources](https://github.com/Aidoku-Community/sources)** — open-source community Rust source implementations and shared templates (`mangabox`, `madara`).
- **[Keiyoushi / Mihon Extensions](https://github.com/keiyoushi/extensions-source)** — upstream Kotlin extension implementations, CSS selectors, filters, and mirror domains tracked by our porting and auto-sync scripts.

All site names, logos, and upstream extension code remain the property of their respective owners.

---

## Curated Sources (17)

| Source | ID | Version | Languages | Min. Aidoku | Notes |
|---|---|---:|---|---:|---|
| <img src="assets/icons/en.atsumaru.png" width="22" align="top"> **Atsumaru** | `en.atsumaru` | `v12` | English | `0.7.1` | [atsu.moe](https://atsu.moe) · Fast JSON API without heavy warmup |
| <img src="assets/icons/en.comix.png" width="22" align="top"> **Comix** | `en.comix` | `v29` | English | `0.8.4` | [comix.to](https://comix.to) · Large library + WebView cover resolution after Verify Captcha |
| <img src="assets/icons/multi.kagane.png" width="22" align="top"> **Kagane** | `multi.kagane` | `v26` | 15 languages | `0.7.1` | [kagane.moe](https://kagane.moe) · Mostly manhwa with scanlation source badges |
| <img src="assets/icons/multi.mangaplus.png" width="22" align="top"> **MANGA Plus** | `multi.mangaplus` | `v6` | 9 languages | `0.7.2` | [mangaplus.shueisha.co.jp](https://mangaplus.shueisha.co.jp) · Official Shueisha simulpubs |
| <img src="assets/icons/multi.mangaball.png" width="22" align="top"> **Mangaball** | `multi.mangaball` | `v13` | 34 languages (incl. `uk`) | `0.7.1` | [mangaball.com](https://mangaball.com) · Broad language support + multi-CDN cover fallback |
| <img src="assets/icons/en.mangabat.png" width="22" align="top"> **MangaBat** | `en.mangabat` | `v9` | English | `0.7.1` | [mangabats.com](https://www.mangabats.com) · Shared `mangabox` template |
| <img src="assets/icons/multi.mangadex.png" width="22" align="top"> **MangaDex** | `multi.mangadex` | `v16` | 54 languages (incl. `uk`, `en`) | `0.7.1` | [mangadex.org](https://mangadex.org) · Public API with broad language & group filters |
| <img src="assets/icons/en.mangadot.png" width="22" align="top"> **MangaDotNet** | `en.mangadot` | `v20` | 43 languages (incl. `uk`) | `0.8.3` | [mangadot.net](https://mangadot.net) · Consistent single-group chapter deduplication |
| <img src="assets/icons/multi.mangadotnet.png" width="22" align="top"> **Mangadotnet** | `multi.mangadotnet` | `v10` | 43 languages (incl. `uk`) | `0.9.0` | [mangadot.net](https://mangadot.net) · Aidoku 0.9.0+ variant |
| <img src="assets/icons/multi.mangafire.png" width="22" align="top"> **MangaFire** | `multi.mangafire` | `v9` | 7 languages (`en`, `es`, `fr`, `ja`, `pt`…) | `0.7.1` | [mangafire.to](https://mangafire.to) · Multi-language catalog & rich filtering |
| <img src="assets/icons/en.mangago.png" width="22" align="top"> **Mangago** <img alt="18+" src="https://img.shields.io/badge/18+-e11d48?style=flat-square" align="top"> | `en.mangago` | `v4` | English | `0.7.1` | [mangago.me](https://www.mangago.me) · Full site catalog (NSFW) |
| <img src="assets/icons/en.mangakakalot.png" width="22" align="top"> **MangaKakalot** | `en.mangakakalot` | `v6` | English | `0.7.1` | [mangakakalot.gg](https://www.mangakakalot.gg) · Shared `mangabox` template |
| <img src="assets/icons/en.manganato.png" width="22" align="top"> **MangaNato** | `en.manganato` | `v8` | English | `0.7.1` | [manganato.gg](https://www.manganato.gg) · Shared `mangabox` template |
| <img src="assets/icons/en.manganelo.png" width="22" align="top"> **MangaNelo** | `en.manganelo` | `v4` | English | `0.7.1` | [nelomanga.net](https://www.nelomanga.net) · Shared `mangabox` template |
| <img src="assets/icons/en.mangapill.png" width="22" align="top"> **MangaPill** | `en.mangapill` | `v4` | English | `0.7.1` | [mangapill.com](https://mangapill.com) · Native Rust port from Mihon Kotlin source |
| <img src="assets/icons/en.toonily.png" width="22" align="top"> **Toonily** <img alt="18+" src="https://img.shields.io/badge/18+-e11d48?style=flat-square" align="top"> | `en.toonily` | `v10` | English | `0.7.1` | [toonily.com](https://toonily.com) · Shared `madara` template (NSFW) |
| <img src="assets/icons/en.weebcentral.png" width="22" align="top"> **Weeb Central** | `en.weebcentral` | `v9` | English | `0.7.1` | [weebcentral.com](https://weebcentral.com) · Clean search & tag filters |

<sub>For multi-language sources, pick your preferred reading languages inside each source's **Settings** inside Aidoku.</sub>

---

## Source Highlights & Custom Fixes

<h3><img src="assets/icons/en.comix.png" width="24" align="top">&nbsp; Comix</h3>

<sub><code>en.comix</code> &nbsp;·&nbsp; <a href="https://comix.to">comix.to</a> &nbsp;·&nbsp; <strong>requires Aidoku 0.8.4+</strong></sub>

- **Post-Captcha Cover Loading.** After passing the check via **Comix Settings → Verify Captcha**, `ComixWebView` (`resolve_covers_in_webview` + `CoverImageProcessor`) automatically recovers covers even when direct CDN requests are restricted.
- **Descrambling & Deduplication.** Supports Comix tile descrambling and smart group/official release deduplication.

<details>
<summary><strong>Comix Verification Steps</strong></summary>

1. Open **Browse → Comix → Settings → Verify Captcha** inside Aidoku and complete the challenge.
2. Refresh the library or manga view — covers will automatically load through the verified WebView session if needed.
3. If a session expires later, run **Verify Captcha** again or **Clear Cache**.

</details>

<h3><img src="assets/icons/multi.mangaball.png" width="24" align="top">&nbsp; Mangaball</h3>

<sub><code>multi.mangaball</code> &nbsp;·&nbsp; <a href="https://mangaball.com">mangaball.com</a> &nbsp;·&nbsp; <strong>updated in v13</strong></sub>

- **Reliable Poster Loading (`v13`).**
  - Filters out defunct external hosts (`hentaicdn.org` / `hencover`) so titles fall back to valid mirrors.
  - Normalizes relative `/media/...`, `/covers/...`, Windows `\`, `background`, and relative MangaDex cover paths.
  - Adjusts `ImageRequestProvider` `Referer` headers per CDN (`mangadex.org`, `mangaupdates.com`, `mangaball.com`) and implements `CoverImageProcessor` fallback retries when an image host returns a 403/blank response.

<h3><img src="assets/icons/en.mangadot.png" width="24" align="top">&nbsp; MangaDotNet & Mangadotnet</h3>

<sub><code>en.mangadot</code> (v20, Aidoku 0.8.3+) &nbsp;·&nbsp; <code>multi.mangadotnet</code> (v10, Aidoku 0.9.0+) &nbsp;·&nbsp; <a href="https://mangadot.net">mangadot.net</a></sub>

- Both editions (`v20` and `v10`) are kept side-by-side to support both Aidoku 0.8.x and Aidoku 0.9.0+.

<h3><img src="assets/icons/en.mangapill.png" width="24" align="top">&nbsp; MangaPill</h3>

<sub><code>en.mangapill</code> &nbsp;·&nbsp; <a href="https://mangapill.com">mangapill.com</a></sub>

- Direct Rust (`aidoku-rs`) port from Mihon (`src/en/mangapill`) supporting search, type/status/genre filters, chapter lists, and page extraction.

---

## If a Source Gets Stuck Loading

> Keep **Aidoku** updated to the latest version — most `403` errors and endless spinners come from app-side Cloudflare handling rather than the source extension.

If a source still stalls after updating:

1. Open **Browse**, select the source, and tap **⋯ → Settings**.
2. Scroll down and tap **Clear Cache** (or **Verify Captcha** for Comix).
3. **Force close Aidoku** and reopen it to reset stale Cloudflare cookies.

---

## Managing the Source List & Automation CLI

All active extensions are registered in [`extensions.json`](extensions.json). As your personal list evolves, you can add, sync, or verify extensions locally:

```bash
# List tracked sources and pinned upstream commits
node scripts/mihon-to-aidoku.mjs --list

# Port or scaffold a new Mihon extension from keiyoushi/extensions-source
node scripts/mihon-to-aidoku.mjs <extension-name>

# Dry-run the upstream Mihon / Aidoku sync check
node scripts/sync-upstream.mjs --dry-run

# Verify index.min.json and every compiled .aix package
node scripts/build-repo.mjs --verify
```

### Optional AI-Assisted Kotlin → Rust Porting in CI
1. Generate an API key in [Google AI Studio](https://aistudio.google.com/apikey).
2. In your GitHub repository, open **Settings → Secrets and variables → Actions → New repository secret**.
3. Add **`GEMINI_API_KEY`** so [`scripts/sync-upstream.mjs`](scripts/sync-upstream.mjs) can automatically adapt complex upstream Kotlin diffs into Rust and validate them with `cargo check` before publishing.

---

<div align="center">
<img src="assets/footer-banner.svg" alt="" width="100%">
</div>
