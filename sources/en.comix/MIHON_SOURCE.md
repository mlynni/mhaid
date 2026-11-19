# Mihon Source Reference: Comix

- **Mihon Path:** `src/en/comix`
- **Mihon Upstream Commit:** `9c66c1f7b6774c03ee04ec386e4c9dd88949c660` (v64)
- **Aidoku ID:** `en.comix`
- **Aidoku Version:** `32` (`minAppVersion: "0.8.4"`)
- **Mirrors:** `https://comix.to`, `https://comix.ws` (configurable via Settings → Mirror)
- **Languages (1):** `en`
- **Content Rating:** Contains NSFW / Mixed (`1`)
- **Architecture:** Rewritten from scratch based on Mihon's WebView SPA harvesters (`harvest_browse.js`, `harvest_chapters.js`, `chapter_pages.js`) + SSR `script#initial-data` + `5x5`/XOR image descrambler (`descrambler.rs`) without reverse-engineered `/api/v1` signing.

## Embedded JS Harvesters (`assets/`)
- `assets/harvest_browse.js` — extracts `/title/{hid}-{slug}` manga cards and pagination `perPage` from `/browse?...`.
- `assets/harvest_chapters.js` — harvests paginated chapter rows (`id`, `number`, `volume`, `name`, `votes`, `group`, relative timestamp) directly from `/title/{hid}-{slug}`.
- `assets/chapter_pages.js` — switches reader preferences temporarily to vertical strip (`ttb` / `preload: all`), observes reader `<img>` slots, and captures external CDN image URLs.

## Endpoints & Paths
- Home SSR: `GET {baseUrl}/home` (`script#initial-data`)
- Browse / Search: `{baseUrl}/browse?q=...&page=...&sort=...&content_rating=...&types=...&genres_in=...&genres_ex=...&demos=...&genres_mode=...&statuses=...&min_chap=...&max_chap=...&year_from=...&year_to=...&authors=...&artists=...`
- Term Lookup: `GET {baseUrl}/api/v1/tags/search?type={author|artist|tag}&q=...&limit=20`
- Details & Chapters: `{baseUrl}/title/{hid}-{slug}` (`script#initial-data` + `harvest_chapters.js`)
- Reader Pages: `{baseUrl}/title/{hid}-{slug}/{chapterId}-chapter-{number}` (`chapter_pages.js`)
- Image Requests: No `Referer` / `Origin` headers; retries `?r=1..2` and fallback CDN paths `["/hi/", "/fcf/", "/i5/", "/si/", "/i/", "/sii/", "/ii/"]` before `x-enc-*` XOR decoding and `5x5` tile descrambling.
