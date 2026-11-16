# Mihon Source Reference: Kagane

- **Mihon Path:** `src/all/kagane`
- **Mihon Version Code:** `42`
- **Aidoku Suggested ID:** `multi.kagane`
- **Base URL:** https://kagane.moe
- **Languages (18):** ar, de, en, es, es-419, fr, hi, id, it, ja, ko, pl, pt, pt-BR, ru, th, vi, zh
- **Content Rating:** Contains NSFW / Mixed (1)
- **Architecture:** Custom (Filter.CheckBox, Filter.TriState, KeiSource)

## Extracted Kotlin Files
- `src/eu/kanade/tachiyomi/extension/all/kagane/Dto.kt`
- `src/eu/kanade/tachiyomi/extension/all/kagane/Filters.kt`
- `src/eu/kanade/tachiyomi/extension/all/kagane/Kagane.kt`

## Endpoints & URLs
- `$baseUrl/api/integrity`
- `$baseUrl/api/v2`
- `$baseUrl/series/$seriesSegment`
- `$baseUrl/series/$slug-${parts[2].takeLast(36)}/reader/${parts[4]}`
- `$baseUrl/sources/$sourceId`
- `https://kagane.org/`

## CSS Selectors
- `li:has(p[aria-live]) a[href^=https://]`

## Filter Classes
- `ChapterCountFilter`
- `ContentRatingFilter`
- `FilterData`
- `FormatFilter`
- `GenresFilter`
- `GroupsSearchFilter`
- `JsonMultiSelectFilter`
- `JsonMultiSelectTriFilter`
- `MatchAllGenresFilter`
- `MatchAllTagsFilter`
- `PublicationStatusFilter`
- `RangeTextFilter`
- `SelectFilterOption`
- `SortFilter`
- `SourcesFilter`
- `StaffSearchFilter`
- `TagsSearchFilter`
- `YearFilter`
