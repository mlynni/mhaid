# Mihon Source Reference: MangaPill

- **Mihon Path:** `src/en/mangapill`
- **Mihon Version Code:** `1`
- **Aidoku Suggested ID:** `en.mangapill`
- **Base URL:** https://mangapill.com
- **Languages (1):** en
- **Content Rating:** Contains NSFW / Mixed (1)
- **Architecture:** Custom (Filter.Group<Genre>, Filter.TriState, KeiSource, UriPartFilter)

## Extracted Kotlin Files
- `src/eu/kanade/tachiyomi/extension/en/mangapill/Filters.kt`
- `src/eu/kanade/tachiyomi/extension/en/mangapill/MangaPill.kt`

## Endpoints & URLs
- `$baseUrl/`
- `$baseUrl/chapters`
- `$baseUrl/search`

## CSS Selectors
- `#chapters > div > a`
- `.grid > div:not([class])`
- `a.btn.btn-sm`
- `a[href*=genre]`
- `a[href^='/manga/']`
- `div.container > div:first-child > div:first-child > img`
- `div.container > div:first-child > div:last-child > div:nth-child(2) > p`
- `div.container > div:first-child > div:last-child > div:nth-child(3) > div:nth-child(2) > div`
- `div.line-clamp-2`
- `div:has(h4:contains(Trending)) > .grid > div:not([class])`
- `img`
- `picture img`

## Filter Classes
- `UriPartFilter`
