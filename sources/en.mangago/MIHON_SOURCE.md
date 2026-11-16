# Mihon Source Reference: Mangago

- **Mihon Path:** `src/en/mangago`
- **Mihon Version Code:** `45`
- **Aidoku Suggested ID:** `en.mangago`
- **Base URL:** https://www.mangago.me
- **Languages (1):** en
- **Content Rating:** Contains NSFW / Mixed (1)
- **Architecture:** Custom (Filter.Group<GenreFilter>, Filter.Group<StatusFilter>, KeiSource)

## Extracted Kotlin Files
- `src/eu/kanade/tachiyomi/extension/en/mangago/Filters.kt`
- `src/eu/kanade/tachiyomi/extension/en/mangago/Mangago.kt`
- `src/eu/kanade/tachiyomi/extension/en/mangago/SoJsonV4Deobfuscator.kt`

## Endpoints & URLs
- `$baseUrl/genre/`
- `$baseUrl/genre/all/`
- `$baseUrl/genre/all/$page/?f=1&o=1&sortby=update_date&e=`
- `$baseUrl/genre/all/$page/?f=1&o=1&sortby=view&e=`
- `$baseUrl/r/l_search`
- `$baseUrl/read-manga/${location.pathSegments[1]}`

## CSS Selectors
- `#genre_panel .genre_select_div[_id]`
- `.also-like li`
- `.current+li > a`
- `.manga_info li, .manga_right tr`
- `.manga_summary`
- `.thm-effect`
- `.updatesli, .pic_list > li`
- `.w-title h1`
- `:is(table#raws_table, table#chapter_table) > tbody > tr, table.uk-table > tbody > tr`
- `a`
- `a.chico`
- `b, label`
- `div.also_like:has(h4:contains(Other manga by the same author)) + .pic_list .updatesli`
- `img`
- `input#curl`
- `script:containsData(imgsrcs)`
- `script:containsData(total_pages)`
- `script[src*=chapter.js]`
- `span`
- `td.no a, td.uk-table-shrink a`
- `td:last-child`

## Filter Classes
- `GenreFilter`
- `GenreFilterGroup`
- `SortFilter`
- `StatusFilter`
- `StatusFilterGroup`
- `UriPartFilter`
