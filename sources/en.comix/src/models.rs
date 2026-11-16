use crate::{BASE_URL, helpers, settings};
use aidoku::{
	Chapter, ContentRating, HashMap, Manga, MangaPageResult, MangaStatus, Viewer,
	alloc::{String, Vec, string::ToString, vec},
	prelude::*,
};
use serde::{Deserialize, Deserializer, de};

#[derive(Deserialize)]
pub struct ErrorResponse {
	pub error: String,
}

#[derive(Deserialize)]
pub struct SearchResponse {
	#[serde(deserialize_with = "manga_items_or_vec")]
	pub result: MangaItems,
}

fn manga_items_or_vec<'de, D>(deserializer: D) -> Result<MangaItems, D::Error>
where
	D: Deserializer<'de>,
{
	let value = serde_json::Value::deserialize(deserializer)?;

	if let Some(items) = value.get("items") {
		let items: Vec<ComixManga> =
			serde_json::from_value(items.clone()).map_err(serde::de::Error::custom)?;
		let meta = value
			.get("meta")
			.map(|m| serde_json::from_value(m.clone()).map_err(serde::de::Error::custom))
			.transpose()?;
		Ok(MangaItems { items, meta })
	} else if value.is_array() {
		let items: Vec<ComixManga> =
			serde_json::from_value(value).map_err(serde::de::Error::custom)?;
		Ok(MangaItems { items, meta: None })
	} else {
		Err(serde::de::Error::custom(
			"Invalid MangaItems or Vec<ComixManga>",
		))
	}
}

impl From<SearchResponse> for MangaPageResult {
	fn from(value: SearchResponse) -> Self {
		value.result.into()
	}
}

#[derive(Deserialize)]
pub struct SingleMangaResponse {
	pub result: ComixManga,
}

#[derive(Deserialize)]
pub struct ChapterDetailsResponse {
	pub result: ChapterItems,
}

#[derive(Deserialize)]
pub struct ChapterResponse {
	pub result: Option<ComixChapterWithPages>,
}

#[derive(Deserialize)]
pub struct TermResponse {
	pub result: TermItems,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
	pub page: i32,
	pub last_page: i32,
}

#[derive(Deserialize)]
pub struct MangaItems {
	pub items: Vec<ComixManga>,
	pub meta: Option<Pagination>,
}

impl MangaItems {
	pub fn into_filtered(self, content_types: &[String], hidden_terms: &[i32]) -> MangaPageResult {
		MangaPageResult {
			entries: self
				.items
				.into_iter()
				.filter(|m| !m.is_hidden(content_types, hidden_terms))
				.map(Into::into)
				.collect(),
			has_next_page: self.meta.map(|p| p.page < p.last_page).unwrap_or_default(),
		}
	}
}

impl From<MangaItems> for MangaPageResult {
	fn from(value: MangaItems) -> Self {
		MangaPageResult {
			entries: value.items.into_iter().map(Into::into).collect(),
			has_next_page: value.meta.map(|p| p.page < p.last_page).unwrap_or_default(),
		}
	}
}

#[derive(Deserialize)]
pub struct ChapterItems {
	pub items: Vec<ComixChapter>,
	pub meta: Pagination,
}

#[derive(Deserialize)]
pub struct TermItems {
	pub items: Vec<Term>,
	// pub pagination: Pagination,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComixManga {
	pub hid: String,
	pub title: String,
	#[serde(default, alias = "alt_titles")]
	pub alt_titles: Vec<String>,
	pub synopsis: Option<String>,
	#[serde(default)]
	pub r#type: String,
	#[serde(alias = "cover", alias = "thumbnail", alias = "thumb")]
	pub poster: Option<Poster>,
	#[serde(default)]
	pub status: String,
	#[serde(default)]
	pub content_rating: ComixContentRating,
	#[serde(alias = "author")]
	pub authors: Option<Vec<Term>>,
	#[serde(alias = "artist")]
	pub artists: Option<Vec<Term>>,
	#[serde(alias = "genre")]
	pub genres: Option<Vec<Term>>,
	#[serde(alias = "theme")]
	pub tags: Option<Vec<Term>>,
	#[serde(alias = "demographic")]
	pub demographics: Option<Vec<Term>>,
	pub rated_avg: Option<f64>,
	pub rated_count: Option<i64>,
	pub follows_total: Option<i64>,
	pub rank: Option<i32>,
	pub year: Option<i32>,
	pub original_language: Option<String>,
	pub links: Option<HashMap<String, serde_json::Value>>,
	pub latest_chapter: Option<f32>,
	#[serde(default)]
	pub url: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ComixContentRating {
	Safe,
	Suggestive,
	Erotica,
	Pornographic,
	#[default]
	#[serde(other)]
	Unknown,
}

impl From<ComixContentRating> for ContentRating {
	fn from(value: ComixContentRating) -> Self {
		match value {
			ComixContentRating::Safe => ContentRating::Safe,
			ComixContentRating::Suggestive => ContentRating::Suggestive,
			ComixContentRating::Erotica => ContentRating::NSFW,
			ComixContentRating::Pornographic => ContentRating::NSFW,
			ComixContentRating::Unknown => ContentRating::Unknown,
		}
	}
}

impl ComixManga {
	pub fn into_detailed_manga(mut self) -> Manga {
		let mut parts: Vec<String> = Vec::new();
		if let Some(score) = self.rated_avg.filter(|&s| s > 0.0) {
			parts.push(format!("Score: {score:.2}"));
		}
		if let Some(synopsis) = self.synopsis.take().filter(|s| !s.trim().is_empty()) {
			parts.push(synopsis);
		}
		if settings::alt_names_in_desc() && !self.alt_titles.is_empty() {
			parts.push(format!("Alternative Titles:\n- {}", self.alt_titles.join("\n- ")));
		}
		if settings::show_extra_info() {
			let mut meta = Vec::new();
			if let Some(y) = self.year.filter(|&y| y > 0) {
				meta.push(format!("Year: {y}"));
			}
			if let Some(lang) = self.original_language.as_deref().filter(|l| !l.is_empty()) {
				meta.push(format!("Language: {}", lang.to_ascii_uppercase()));
			}
			if let Some(rank) = self.rank.filter(|&r| r > 0) {
				meta.push(format!("Rank: #{rank}"));
			}
			if let Some(rated_by) = self.rated_count.filter(|&c| c > 0) {
				meta.push(format!("Rated by: {rated_by}"));
			}
			if let Some(followed_by) = self.follows_total.filter(|&f| f > 0) {
				meta.push(format!("Followed by: {followed_by}"));
			}
			if let Some(links) = self.links.take() {
				let tracker_names = [
					("al", "AniList"),
					("mal", "MyAnimeList"),
					("mu", "MangaUpdates"),
					("md", "MangaDex"),
					("mb", "MangaBaka"),
				];
				let mut tracker_links = Vec::new();
				for (key, label) in tracker_names {
					if let Some(url) = links.get(key).and_then(|v| v.as_str()).filter(|u| u.starts_with("http")) {
						tracker_links.push(format!("[{label}]({url})"));
					}
				}
				if !tracker_links.is_empty() {
					meta.push(format!("Trackers: {}", tracker_links.join(" · ")));
				}
			}
			if !meta.is_empty() {
				parts.push(meta.join("\n"));
			}
		}

		let mut manga: Manga = self.into();
		manga.description = (!parts.is_empty()).then(|| parts.join("\n\n"));
		manga
	}

	pub fn is_hidden(&self, hidden_types: &[String], hidden_terms: &[i32]) -> bool {
		if hidden_types.contains(&self.r#type) {
			return true;
		}

		if !hidden_terms.is_empty() {
			let tag_match = self
				.tags
				.as_ref()
				.map(|tags| tags.iter().any(|term| hidden_terms.contains(&term.id)))
				.unwrap_or(false);
			if tag_match {
				true
			} else {
				self.genres
					.as_ref()
					.map(|genres| genres.iter().any(|term| hidden_terms.contains(&term.id)))
					.unwrap_or(false)
			}
		} else {
			false
		}
	}
}

impl From<ComixManga> for Manga {
	fn from(value: ComixManga) -> Self {
		let url = format!("{BASE_URL}/{}", value.url.trim_start_matches('/'));
		Self {
			key: value.hid,
			title: value.title,
			cover: value
				.poster
				.and_then(|poster| match settings::image_quality().as_str() {
					"medium" => poster.medium.or(poster.large).or(poster.small),
					"small" => poster.small.or(poster.medium).or(poster.large),
					_ => poster.large.or(poster.medium).or(poster.small),
				})
				.and_then(normalize_cover_url),
			artists: value
				.artists
				.map(|v| v.into_iter().map(|t| t.title).collect()),
			authors: value
				.authors
				.map(|v| v.into_iter().map(|t| t.title).collect()),
			description: None,
			url: Some(url),
			tags: {
				let mut tags = Vec::new();
				if let Some(genres) = value.genres {
					tags.extend(genres.into_iter().map(|t| t.title));
				}
				if let Some(demographics) = value.demographics {
					tags.extend(demographics.into_iter().map(|t| t.title));
				}
				if let Some(tags_vec) = value.tags {
					tags.extend(tags_vec.into_iter().map(|t| t.title));
				}
				if tags.is_empty() { None } else { Some(tags) }
			},
			status: match value.status.as_str() {
				"releasing" => MangaStatus::Ongoing,
				"on_hiatus" => MangaStatus::Hiatus,
				"finished" => MangaStatus::Completed,
				"discontinued" => MangaStatus::Cancelled,
				_ => MangaStatus::Unknown,
			},
			content_rating: value.content_rating.into(),
			viewer: match value.r#type.as_str() {
				"manhwa" => Viewer::Webtoon,
				"manhua" => Viewer::Webtoon,
				"manga" => Viewer::RightToLeft,
				_ => Viewer::Unknown,
			},
			..Default::default()
		}
	}
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComixChapter {
	pub id: i32,
	pub number: f32,
	pub name: String,
	pub votes: i32,
	pub created_at_formatted: String,
	pub group: Option<ScanlationGroup>,
	#[serde(deserialize_with = "bool_from_any")]
	pub is_official: bool,
	pub url: String,
}

impl ComixChapter {
	pub fn created_at(&self) -> i64 {
		helpers::parse_relative_date_string(&self.created_at_formatted)
	}
}

impl From<ComixChapter> for Chapter {
	fn from(value: ComixChapter) -> Self {
		let created_at = value.created_at();
		Chapter {
			key: value.id.to_string(),
			title: (!value.name.is_empty()).then_some(value.name),
			chapter_number: Some(value.number),
			date_uploaded: Some(created_at),
			scanlators: if let Some(group) = value.group {
				Some(vec![group.name])
			} else if value.is_official {
				Some(vec!["Official".into()])
			} else {
				None
			},
			url: Some(format!("{BASE_URL}/{}", value.url.trim_start_matches('/'))),
			..Default::default()
		}
	}
}

#[derive(Deserialize)]
pub struct ComixChapterWithPages {
	pub pages: ComixPages,
}

pub fn normalize_cover_url(url: String) -> Option<String> {
	let trimmed = url.trim();
	if trimmed.is_empty() {
		return None;
	}
	if trimmed.starts_with("http://")
		|| trimmed.starts_with("https://")
		|| trimmed.starts_with("data:")
	{
		Some(trimmed.to_string())
	} else if trimmed.starts_with("//") {
		Some(format!("https:{trimmed}"))
	} else if trimmed.starts_with('/') {
		Some(format!("{BASE_URL}{trimmed}"))
	} else {
		Some(format!("{BASE_URL}/{trimmed}"))
	}
}

#[derive(Default)]
pub struct Poster {
	pub small: Option<String>,
	pub medium: Option<String>,
	pub large: Option<String>,
}

impl<'de> Deserialize<'de> for Poster {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct PosterVisitor;

		impl<'de> de::Visitor<'de> for PosterVisitor {
			type Value = Poster;

			fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
				formatter.write_str("a poster URL string or object")
			}

			fn visit_str<E: de::Error>(self, v: &str) -> Result<Poster, E> {
				let s = v.trim();
				Ok(Poster {
					large: (!s.is_empty()).then(|| s.to_string()),
					..Default::default()
				})
			}

			fn visit_string<E: de::Error>(self, v: String) -> Result<Poster, E> {
				let s = v.trim();
				Ok(Poster {
					large: (!s.is_empty()).then(|| s.to_string()),
					..Default::default()
				})
			}

			fn visit_map<A>(self, mut map: A) -> Result<Poster, A::Error>
			where
				A: de::MapAccess<'de>,
			{
				let mut small: Option<String> = None;
				let mut medium: Option<String> = None;
				let mut large: Option<String> = None;
				let mut fallback_url: Option<String> = None;

				while let Some(key) = map.next_key::<String>()? {
					match key.as_str() {
						"small" => {
							small = map
								.next_value::<Option<String>>()?
								.and_then(|s| (!s.trim().is_empty()).then_some(s));
						}
						"medium" => {
							medium = map
								.next_value::<Option<String>>()?
								.and_then(|s| (!s.trim().is_empty()).then_some(s));
						}
						"large" => {
							large = map
								.next_value::<Option<String>>()?
								.and_then(|s| (!s.trim().is_empty()).then_some(s));
						}
						"url" | "thumb" => {
							fallback_url = map
								.next_value::<Option<String>>()?
								.and_then(|s| (!s.trim().is_empty()).then_some(s));
						}
						_ => {
							let _ = map.next_value::<de::IgnoredAny>()?;
						}
					}
				}

				if large.is_none() {
					large = fallback_url;
				}

				Ok(Poster {
					small,
					medium,
					large,
				})
			}
		}

		deserializer.deserialize_any(PosterVisitor)
	}
}

#[derive(Deserialize)]
pub struct Term {
	pub id: i32,
	pub title: String,
}

#[derive(Deserialize)]
pub struct ScanlationGroup {
	pub id: i32,
	pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComixPages {
	#[serde(default)]
	pub base_url: String,
	pub items: Vec<ComixPage>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComixPage {
	pub url: String,
	#[serde(default, alias = "scramble_seed")]
	pub scramble_seed: Option<i64>,
	#[serde(default, alias = "scramble_grid")]
	pub scramble_grid: Option<String>,
	#[serde(default, alias = "scramble_algo")]
	pub scramble_algo: Option<String>,
	#[serde(default, alias = "scramble_hash")]
	pub scramble_hash: Option<String>,
	#[serde(default, alias = "enc_seed")]
	pub enc_seed: Option<i64>,
	#[serde(default, alias = "enc_len")]
	pub enc_len: Option<usize>,
	#[serde(default, alias = "enc_algo")]
	pub enc_algo: Option<String>,
}

// deserialize a bool from a json bool, number, or string
fn bool_from_any<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
	struct BoolVisitor;

	impl<'de> de::Visitor<'de> for BoolVisitor {
		type Value = bool;

		fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
			formatter.write_str("a boolean or 0/1")
		}

		fn visit_bool<E>(self, v: bool) -> Result<bool, E> {
			Ok(v)
		}

		fn visit_u64<E>(self, v: u64) -> Result<bool, E> {
			match v {
				0 => Ok(false),
				_ => Ok(true),
			}
		}

		fn visit_i64<E>(self, v: i64) -> Result<bool, E> {
			match v {
				0 => Ok(false),
				_ => Ok(true),
			}
		}

		fn visit_str<E: de::Error>(self, v: &str) -> Result<bool, E> {
			match v.to_ascii_lowercase().as_str() {
				"true" => Ok(true),
				"false" => Ok(false),
				"1" => Ok(true),
				"0" => Ok(false),
				_ => Err(E::custom(format!("invalid string for bool: {v}"))),
			}
		}

		fn visit_none<E>(self) -> Result<bool, E> {
			Ok(false)
		}
	}

	deserializer.deserialize_any(BoolVisitor)
}
