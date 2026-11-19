# Mihon Source Reference: Comix

- **Mihon Path:** `src/en/comix`
- **Mihon Version Code:** `65`
- **Aidoku Suggested ID:** `en.comix`
- **Base URL:** https://comix.to
- **Languages (1):** en
- **Content Rating:** Contains NSFW / Mixed (1)
- **Architecture:** Custom (Filter.CheckBox, Filter.TriState, KeiSource)

## Extracted Kotlin Files
- `src/eu/kanade/tachiyomi/extension/en/comix/BackgroundSurface.kt`
- `src/eu/kanade/tachiyomi/extension/en/comix/Comix.kt`
- `src/eu/kanade/tachiyomi/extension/en/comix/Descrambler.kt`
- `src/eu/kanade/tachiyomi/extension/en/comix/Dto.kt`
- `src/eu/kanade/tachiyomi/extension/en/comix/Filters.kt`
- `src/eu/kanade/tachiyomi/extension/en/comix/Swipe.kt`

## Extracted Assets
- `assets/chapter_pages.js`
- `assets/harvest_browse.js`
- `assets/harvest_chapters.js`

## Endpoints & URLs
- `$baseUrl/${chapter.url}`
- `$baseUrl/title${manga.url}`
- `/api/v1/client-errors`
- `/api/v1/threads/`

## Filter Classes
- `ArtistFilter`
- `AuthorFilter`
- `ChapterLengthFilter`
- `ContentRatingFilter`
- `DemographicFilter`
- `Filters`
- `FormatFilter`
- `GenreFilter`
- `MatchModeFilter`
- `SortFilter`
- `StatusFilter`
- `TagsFilter`
- `TermGroupFilter`
- `TypeFilter`
- `UriMultiSelectFilter`
- `UriPartFilter`
- `YearFromFilter`
- `YearToFilter`
