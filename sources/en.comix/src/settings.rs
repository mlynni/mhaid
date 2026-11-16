use crate::VERIFY_COOKIE_KEY;
use aidoku::{
	alloc::{string::String, vec::Vec},
	imports::defaults::{DefaultValue, defaults_get, defaults_get_map, defaults_set},
};

const HIDE_NSFW_KEY: &str = "hideNSFW";
const THUMBNAIL_QUALITY_KEY: &str = "thumbnailQuality";
const DEDUPED_CHAPTER_KEY: &str = "dedupedChapter";
const SCANLATOR_BLACKLIST_KEY: &str = "scanlatorBlacklist";
const SHOW_EXTRA_INFO_KEY: &str = "showExtraInfo";
const ALT_NAMES_IN_DESC_KEY: &str = "altNamesInDesc";

const HIDDEN_TYPES_KEY: &str = "hiddenTypes";
const HIDDEN_GENRES_KEY: &str = "hiddenGenres";
const HIDDEN_THEMES_KEY: &str = "hiddenThemes";
const WAF_PASS_DIRECT_KEY: &str = "wafPass";
const TRANSPORT_MODE_KEY: &str = "transportMode";

pub const VERIFY_KEY: &str = "verify";

pub fn hide_nsfw() -> bool {
	defaults_get::<bool>(HIDE_NSFW_KEY).unwrap_or(true)
}

pub fn image_quality() -> String {
	defaults_get::<String>(THUMBNAIL_QUALITY_KEY).unwrap_or_default()
}

pub fn dedupchapter() -> bool {
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

pub fn show_extra_info() -> bool {
	defaults_get::<bool>(SHOW_EXTRA_INFO_KEY).unwrap_or(true)
}

pub fn alt_names_in_desc() -> bool {
	defaults_get::<bool>(ALT_NAMES_IN_DESC_KEY).unwrap_or(false)
}

pub fn hidden_types() -> Vec<String> {
	defaults_get::<Vec<String>>(HIDDEN_TYPES_KEY).unwrap_or_default()
}

pub fn hidden_terms() -> Vec<i32> {
	hidden_genres().into_iter().chain(hidden_themes()).collect()
}

fn hidden_genres() -> Vec<i32> {
	defaults_get::<Vec<String>>(HIDDEN_GENRES_KEY)
		.unwrap_or_default()
		.into_iter()
		.filter_map(|s| s.parse().ok())
		.collect()
}

fn hidden_themes() -> Vec<i32> {
	defaults_get::<Vec<String>>(HIDDEN_THEMES_KEY)
		.unwrap_or_default()
		.into_iter()
		.filter_map(|s| s.parse().ok())
		.collect()
}

pub fn reset_filters() {
	defaults_set(HIDDEN_TYPES_KEY, DefaultValue::Null);
	defaults_set(HIDDEN_GENRES_KEY, DefaultValue::Null);
	defaults_set(HIDDEN_THEMES_KEY, DefaultValue::Null);
}

pub fn set_verify_cookie(token: &str) {
	let trimmed = token.trim();
	if !trimmed.is_empty() {
		defaults_set(WAF_PASS_DIRECT_KEY, DefaultValue::String(trimmed.into()));
	}
}

pub fn get_verify_cookie() -> Option<String> {
	if let Some(direct) = defaults_get::<String>(WAF_PASS_DIRECT_KEY) {
		let trimmed = direct.trim();
		if !trimmed.is_empty() {
			return Some(trimmed.into());
		}
	}
	defaults_get_map(VERIFY_KEY)?
		.get(VERIFY_COOKIE_KEY)
		.map(|s| s.trim().into())
		.filter(|s: &String| !s.is_empty())
}

pub fn transport_mode() -> String {
	defaults_get::<String>(TRANSPORT_MODE_KEY).unwrap_or_else(|| "auto".into())
}
