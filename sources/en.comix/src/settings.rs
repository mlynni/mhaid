use aidoku::{
	alloc::{String, Vec, string::ToString, vec},
	imports::defaults::{DefaultValue, defaults_get, defaults_set},
};

pub const DEFAULT_MIRROR: &str = "https://comix.to";
pub const ALT_MIRROR: &str = "https://comix.ws";
pub const VERIFY_KEY: &str = "verify";

const MIRROR_KEY: &str = "mirror";
const THUMBNAIL_QUALITY_KEY: &str = "thumbnailQuality";
const CONTENT_RATING_KEY: &str = "contentRating";
const DEDUPED_CHAPTER_KEY: &str = "dedupedChapter";
const SCANLATOR_BLACKLIST_KEY: &str = "scanlatorBlacklist";
const SCORE_POSITION_KEY: &str = "scorePosition";
const SHOW_EXTRA_INFO_KEY: &str = "showExtraInfo";
const ALT_NAMES_IN_DESC_KEY: &str = "altNamesInDesc";
const SHOW_TAGS_IN_GENRES_KEY: &str = "showTagsInGenres";

const DEFAULT_TYPES_KEY: &str = "defaultTypes";
const DEFAULT_DEMOGRAPHICS_KEY: &str = "defaultDemographics";
const BLOCKED_GENRES_KEY: &str = "blockedGenres";

pub fn base_url() -> String {
	defaults_get::<String>(MIRROR_KEY)
		.map(|s| s.trim().trim_end_matches('/').to_string())
		.filter(|s| s.starts_with("https://"))
		.unwrap_or_else(|| DEFAULT_MIRROR.into())
}

pub fn alternate_mirror(url: &str) -> Option<String> {
	if url.contains("comix.to") {
		Some(url.replacen("comix.to", "comix.ws", 1))
	} else if url.contains("comix.ws") {
		Some(url.replacen("comix.ws", "comix.to", 1))
	} else {
		None
	}
}

pub fn thumbnail_quality() -> String {
	defaults_get::<String>(THUMBNAIL_QUALITY_KEY).unwrap_or_else(|| "large".into())
}

pub fn content_rating() -> String {
	defaults_get::<String>(CONTENT_RATING_KEY).unwrap_or_else(|| "suggestive".into())
}

pub fn content_ratings_up_to(rating: &str) -> Vec<&'static str> {
	match rating {
		"safe" => vec!["safe"],
		"suggestive" => vec!["safe", "suggestive"],
		"erotica" => vec!["safe", "suggestive", "erotica"],
		"pornographic" | "all" | "" => vec!["safe", "suggestive", "erotica", "pornographic"],
		_ => vec!["safe", "suggestive"],
	}
}

pub fn default_types() -> Vec<String> {
	defaults_get::<Vec<String>>(DEFAULT_TYPES_KEY).unwrap_or_else(|| {
		vec![
			"manga".into(),
			"manhwa".into(),
			"manhua".into(),
			"other".into(),
		]
	})
}

pub fn default_demographics() -> Vec<String> {
	defaults_get::<Vec<String>>(DEFAULT_DEMOGRAPHICS_KEY)
		.unwrap_or_else(|| vec!["3".into(), "4".into(), "1".into(), "2".into()])
}

pub fn blocked_genres() -> Vec<String> {
	defaults_get::<Vec<String>>(BLOCKED_GENRES_KEY).unwrap_or_default()
}

pub fn dedup_chapters() -> bool {
	defaults_get::<bool>(DEDUPED_CHAPTER_KEY).unwrap_or(false)
}

pub fn scanlator_blacklist() -> Vec<String> {
	defaults_get::<String>(SCANLATOR_BLACKLIST_KEY)
		.unwrap_or_default()
		.split(',')
		.map(|s| s.trim().to_ascii_lowercase())
		.filter(|s| !s.is_empty())
		.collect()
}

pub fn score_position() -> String {
	defaults_get::<String>(SCORE_POSITION_KEY).unwrap_or_else(|| "top".into())
}

pub fn show_extra_info() -> bool {
	defaults_get::<bool>(SHOW_EXTRA_INFO_KEY).unwrap_or(true)
}

pub fn alt_names_in_desc() -> bool {
	defaults_get::<bool>(ALT_NAMES_IN_DESC_KEY).unwrap_or(false)
}

pub fn show_tags_in_genres() -> bool {
	defaults_get::<bool>(SHOW_TAGS_IN_GENRES_KEY).unwrap_or(false)
}

pub fn reset_filters() {
	defaults_set(CONTENT_RATING_KEY, DefaultValue::Null);
	defaults_set(DEFAULT_TYPES_KEY, DefaultValue::Null);
	defaults_set(DEFAULT_DEMOGRAPHICS_KEY, DefaultValue::Null);
	defaults_set(BLOCKED_GENRES_KEY, DefaultValue::Null);
}
