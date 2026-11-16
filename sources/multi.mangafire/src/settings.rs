use aidoku::{
	Result,
	alloc::{string::String, vec, vec::Vec},
	imports::defaults::defaults_get,
	prelude::*,
};

// settings keys
const LANGUAGES_KEY: &str = "languages";
const CONTENT_RATING_KEY: &str = "content_rating";
const SHOW_AS_VOLUMES_KEY: &str = "show_as_volumes";
const MERGE_CHAPTERS_KEY: &str = "merge_chapters";
const PREFER_OFFICIAL_KEY: &str = "prefer_official";

pub fn get_languages() -> Result<Vec<String>> {
	defaults_get::<Vec<String>>(LANGUAGES_KEY)
		.map(|langs| {
			langs
				.into_iter()
				.map(|lang| match lang.as_str() {
					"pt-BR" => "pt-br".into(),
					"es-419" => "es-la".into(),
					_ => lang,
				})
				.collect()
		})
		.ok_or(error!("Unable to fetch languages"))
}

pub fn get_content_ratings() -> Vec<String> {
	defaults_get::<Vec<String>>(CONTENT_RATING_KEY)
		.filter(|v| !v.is_empty())
		.unwrap_or_else(|| vec!["safe".into(), "suggestive".into()])
}

pub fn show_as_volumes() -> bool {
	defaults_get::<bool>(SHOW_AS_VOLUMES_KEY).unwrap_or(false)
}

pub fn merge_chapters() -> bool {
	defaults_get::<bool>(MERGE_CHAPTERS_KEY).unwrap_or(false)
}

pub fn prefer_official() -> bool {
	defaults_get::<bool>(PREFER_OFFICIAL_KEY).unwrap_or(true)
}
