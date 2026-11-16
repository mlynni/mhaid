# Mihon Source Reference: Atsumaru

- **Mihon Path:** `src/en/atsumaru`
- **Mihon Version Code:** `26`
- **Aidoku Suggested ID:** `en.atsumaru`
- **Base URL:** https://atsu.moe
- **Languages (1):** en
- **Content Rating:** Contains NSFW / Mixed (1)
- **Architecture:** Custom (Filter.Group<TagFilter>, KeiSource)

## Extracted Kotlin Files
- `src/eu/kanade/tachiyomi/extension/en/atsumaru/Atsumaru.kt`
- `src/eu/kanade/tachiyomi/extension/en/atsumaru/Dto.kt`
- `src/eu/kanade/tachiyomi/extension/en/atsumaru/Filters.kt`

## Endpoints & URLs
- `$baseUrl/api/explore/availableFilters`
- `$baseUrl/api/home2/popular?offset=$offset&limit=$BROWSE_LIMIT`
- `$baseUrl/api/home2/recentlyUpdated?offset=$offset&limit=$BROWSE_LIMIT`
- `$baseUrl/api/manga/allChapters?mangaId=${manga.url}`
- `$baseUrl/api/manga/page?id=${manga.url}`
- `$baseUrl/api/read/chapter`
- `$baseUrl/collections/manga/documents/search`
- `$baseUrl/manga/${manga.url}`
- `$baseUrl/read/$slug/$name`
- `$baseUrl/static/$it`
- `$baseUrl/static/${page.image.removePrefix(`
- `https://cdn.`

## Filter Classes
- `AdultFilter`
- `Filter`
- `FilterData`
- `GenreFilter`
- `MinChaptersFilter`
- `OfficialFilter`
- `SortFilter`
- `StatusFilter`
- `TagFilter`
- `TagFilters`
- `TypeFilter`
- `YearFilter`

## Custom Headers
- `Content-Type: application/json`
