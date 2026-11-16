# Mihon Source Reference: Manga Ball

- **Mihon Path:** `src/all/mangaball`
- **Mihon Version Code:** `7`
- **Aidoku Suggested ID:** `multi.mangaball`
- **Base URL:** https://mangaball.com
- **Languages (42):** ar, bg, bn, ca, cs, da, de, el, en, es, fa, fi, fr, he, hi, hu, id, is, it, ja, kn, ko, ml, ms, ne, nl, no, pl, pt-BR, ro, ru, sk, sl, sq, sr, sv, ta, th, tr, uk, vi, zh
- **Content Rating:** Contains NSFW / Mixed (1)
- **Architecture:** Custom (KeiSource)

## Extracted Kotlin Files
- `src/eu/kanade/tachiyomi/extension/all/mangaball/Dto.kt`
- `src/eu/kanade/tachiyomi/extension/all/mangaball/Filters.kt`
- `src/eu/kanade/tachiyomi/extension/all/mangaball/MangaBall.kt`
- `src/eu/kanade/tachiyomi/extension/all/mangaball/ScanlatorBlacklist.kt`

## Endpoints & URLs
- `$baseUrl/api/v1/chapter-detail?chapter_id=$chapterId`
- `$baseUrl/api/v1/chapter-detail?chapter_id=${chapter.url}`
- `$baseUrl/api/v1/chapter/chapter-listing-by-title-id`
- `$baseUrl/api/v1/title/detail/$idOrSlug`
- `$baseUrl/api/v1/title/search-advanced`
- `$baseUrl/api/v1/views/update`
- `$baseUrl/chapter-detail/${chapter.url}`
- `$baseUrl/title-detail/$id`
- `https://bulbasaur.poke-black-and-white.net/covers/`

## Filter Classes
- `ContentFilter`
- `DemographicFilter`
- `FormatFilter`
- `GenreFilter`
- `OriginalLanguageFilter`
- `SelectFilter`
- `SortFilter`
- `StatusFilter`
- `TagModeFilter`
- `ThemeFilter`
- `TranslatedLanguageFilter`
- `TriStateFilter`
- `TriStateGroupFilter`
- `TypeFilter`
