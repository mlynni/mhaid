# Mihon Source Reference: Weeb Central

- **Mihon Path:** `src/en/weebcentral`
- **Mihon Version Code:** `27`
- **Aidoku Suggested ID:** `en.weebcentral`
- **Base URL:** https://weebcentral.com
- **Languages (1):** en
- **Content Rating:** Contains NSFW / Mixed (1)
- **Architecture:** Custom (Filter.CheckBox, Filter.TriState, KeiSource)

## Extracted Kotlin Files
- `src/eu/kanade/tachiyomi/extension/en/weebcentral/Filters.kt`
- `src/eu/kanade/tachiyomi/extension/en/weebcentral/WeebCentral.kt`

## Endpoints & URLs
- `$baseUrl/search/data`
- `$baseUrl/series/$seriesId/full-chapter-list`

## CSS Selectors
- `a`
- `article > section > a`
- `button`
- `div.truncate`
- `div:not([class]):last-child`
- `div[x-data] > a`
- `img`
- `li:has(strong:contains(Related Series)) li > a`
- `li:has(strong:contains(Track)) span[data-tip] > a`
- `section:has(> h2 strong:contains(Recommendations)) li.glide__slide > a`
- `section[x-data] > section`
- `section[x-data~=scroll] > img`
- `span`
- `span.flex > span`
- `time[datetime]`

## Filter Classes
- `AdultContentFilter`
- `AnimeAdaptationFilter`
- `AuthorFilter`
- `OfficialTranslationFilter`
- `SortFilter`
- `SortOrderFilter`
- `StatusFilter`
- `TagFilter`
- `TypeFilter`
- `UriMultiSelectFilter`
- `UriMultiTriSelectFilter`
- `UriPartFilter`
