use crate::{api::BASE_URL, helpers::title_type};
use aidoku::{
	AidokuError, Chapter, ContentRating, Manga, MangaStatus, Result, Viewer,
	alloc::{String, Vec, format, vec},
	imports::std::parse_date,
	prelude::*,
};
use core::cmp::Ordering;
use serde::{Deserialize, Deserializer};

const CDN_URL: &str = "https://bulbasaur.poke-black-and-white.net/covers";

#[derive(Deserialize)]
pub struct ApiResponse<T> {
	pub data: T,
}

#[derive(Deserialize)]
pub struct Paginated<T> {
	pub data: Vec<T>,
	pub pagination: Pagination,
}

#[derive(Deserialize)]
pub struct Pagination {
	pub page: i32,
	pub total_pages: i32,
}

#[derive(Deserialize)]
pub struct Title {
	#[serde(alias = "_id")]
	id: String,
	name: Option<String>,
	#[serde(default)]
	image: Option<ImageValue>,
	#[serde(default)]
	background: Option<String>,
	#[serde(default)]
	description: Vec<String>,
	#[serde(default, rename = "originalLanguage")]
	pub original_language: String,
	status: Option<String>,
	#[serde(
		default,
		rename = "availableTranslatedLanguages",
		deserialize_with = "string_or_vec"
	)]
	pub translated_languages: Vec<String>,
	#[serde(default)]
	pub tags: Vec<Tag>,
	#[serde(default)]
	author: Vec<Author>,
	#[serde(default)]
	is18plus: bool,
	#[serde(default)]
	pub recent_chapters: Vec<ChapterItem>,
}

fn string_or_vec<'de, D: Deserializer<'de>>(
	deserializer: D,
) -> core::result::Result<Vec<String>, D::Error> {
	#[derive(Deserialize)]
	#[serde(untagged)]
	enum StringOrVec {
		Single(String),
		Multiple(Vec<String>),
	}
	Ok(match StringOrVec::deserialize(deserializer)? {
		StringOrVec::Single(value) => vec![value],
		StringOrVec::Multiple(values) => values,
	})
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ImageValue {
	Url(String),
	Object(Image),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ImageFileValue {
	Url(String),
	Object(Cover),
}

#[derive(Deserialize)]
struct Image {
	file: Option<ImageFileValue>,
	cdn_mangadex: Option<String>,
	cdn_mangaupdate: Option<String>,
	cdn_mangaupdates: Option<String>,
	cover: Option<ImageFileValue>,
}

#[derive(Deserialize)]
struct Cover {
	path: Option<String>,
	url: Option<String>,
}

#[derive(Deserialize)]
pub struct Tag {
	pub id: String,
	name: String,
}

#[derive(Deserialize)]
struct Author {
	name: String,
}

#[derive(Deserialize)]
pub struct ChapterItem {
	id: String,
	name: Option<String>,
	number: Option<f32>,
	#[serde(default)]
	volume: f32,
	lang: String,
	group: Option<Group>,
	created_at: Option<String>,
	likes: Option<u64>,
	views: Option<u64>,
}

#[derive(Deserialize)]
struct Group {
	id: Option<String>,
	name: String,
}

#[derive(Deserialize)]
pub struct ChapterDetail {
	pub chapter: ChapterPages,
}

#[derive(Deserialize)]
pub struct ChapterPages {
	pub title_id: Option<String>,
	#[serde(default)]
	pub pages: Vec<String>,
}

fn is_dead_cover_host(url: &str) -> bool {
	let lower = url.to_lowercase();
	lower.contains("hentaicdn.org") || lower.contains("hencover")
}

fn normalize_cover_url(raw: String, is_cover_cdn: bool, is_mangadex: bool) -> Option<String> {
	let cleaned = raw.trim().replace('\\', "/");
	if cleaned.is_empty() || is_dead_cover_host(&cleaned) {
		return None;
	}
	if cleaned.starts_with("http://") || cleaned.starts_with("https://") {
		return Some(cleaned);
	}
	if cleaned.starts_with("//") {
		return Some(format!("https:{cleaned}"));
	}
	if is_mangadex {
		return Some(format!(
			"https://uploads.mangadex.org/covers/{}",
			cleaned.trim_start_matches('/')
		));
	}
	if cleaned.starts_with("/media/")
		|| cleaned.starts_with("media/")
		|| cleaned.starts_with("/uploads/")
		|| cleaned.starts_with("uploads/")
		|| cleaned.starts_with("/storage/")
		|| cleaned.starts_with("storage/")
	{
		return Some(format!("{BASE_URL}/{}", cleaned.trim_start_matches('/')));
	}
	if cleaned.starts_with("/covers/") || cleaned.starts_with("covers/") {
		return Some(format!(
			"https://bulbasaur.poke-black-and-white.net/{}",
			cleaned.trim_start_matches('/')
		));
	}
	if is_cover_cdn {
		return Some(format!("{CDN_URL}/{}", cleaned.trim_start_matches('/')));
	}
	if cleaned.starts_with('/') {
		return Some(format!("{BASE_URL}{cleaned}"));
	}
	Some(format!("{CDN_URL}/{cleaned}"))
}

impl ImageFileValue {
	fn into_raw(self) -> Option<String> {
		match self {
			Self::Url(url) => Some(url),
			Self::Object(cover) => cover
				.path
				.filter(|path| !path.trim().is_empty())
				.or(cover.url),
		}
	}
}

impl ImageValue {
	// Prefer Mangaball's hosted cover copy and fall back across mirrors, skipping dead hosts.
	fn into_url(self) -> Option<String> {
		match self {
			Self::Url(url) => normalize_cover_url(url, false, false),
			Self::Object(image) => {
				let hosted = image
					.cover
					.and_then(ImageFileValue::into_raw)
					.and_then(|raw| normalize_cover_url(raw, true, false));
				let mangadex = image
					.cdn_mangadex
					.and_then(|raw| normalize_cover_url(raw, false, true));
				let file = image
					.file
					.and_then(ImageFileValue::into_raw)
					.and_then(|raw| normalize_cover_url(raw, false, false));
				let mangaupdates = image
					.cdn_mangaupdates
					.and_then(|raw| normalize_cover_url(raw, false, false));
				let mangaupdate = image
					.cdn_mangaupdate
					.and_then(|raw| normalize_cover_url(raw, false, false));
				[hosted, mangadex, file, mangaupdates, mangaupdate]
					.into_iter()
					.flatten()
					.next()
			}
		}
	}
}

impl Title {
	pub fn content_rating(&self) -> ContentRating {
		let has_tag = |names: &[&str]| {
			self.tags
				.iter()
				.any(|tag| names.contains(&tag.name.as_str()))
		};
		if self.is18plus || has_tag(&["Adult", "Hentai", "Manhwa 18+", "Smut"]) {
			ContentRating::NSFW
		} else if has_tag(&["Ecchi", "Mature"]) {
			ContentRating::Suggestive
		} else {
			ContentRating::Safe
		}
	}

	fn into_manga(self, detailed: bool) -> Result<Manga> {
		let content_rating = self.content_rating();
		let name = self.name.ok_or_else(|| error!("Series has no title"))?;
		if self.id.trim().is_empty() || name.trim().is_empty() {
			bail!("Series has no key or title");
		}
		let viewer = match title_type(&self.original_language) {
			Some("manhwa" | "manhua") => Viewer::Webtoon,
			_ if self.tags.iter().any(|tag| tag.name == "Long Strip") => Viewer::Webtoon,
			Some("manga") => Viewer::RightToLeft,
			_ => Viewer::Unknown,
		};
		let status = match self.status.as_deref().map(str::to_lowercase).as_deref() {
			Some("ongoing") => MangaStatus::Ongoing,
			Some("completed") => MangaStatus::Completed,
			Some("hiatus" | "on_hold") => MangaStatus::Hiatus,
			Some("cancelled") => MangaStatus::Cancelled,
			_ => MangaStatus::Unknown,
		};
		let cover = self
			.image
			.and_then(ImageValue::into_url)
			.or_else(|| self.background.and_then(|bg| normalize_cover_url(bg, false, false)));
		let mut manga = Manga {
			cover,
			status,
			content_rating,
			..Default::default()
		};
		if detailed {
			let mut tags: Vec<_> = self.tags.into_iter().map(|tag| tag.name).collect();
			tags.sort_unstable();
			tags.dedup();
			manga.url = Some(format!("{BASE_URL}/title-detail/{}", self.id));
			manga.description = Some(self.description.join("\n\n")).filter(|text| !text.is_empty());
			let mut authors = Vec::new();
			for author in self.author {
				let name = author.name.trim();
				if !name.is_empty() && !authors.iter().any(|existing| existing == name) {
					authors.push(name.into());
				}
			}
			manga.authors = (!authors.is_empty()).then_some(authors);
			manga.tags = Some(tags);
			manga.viewer = viewer;
		}
		manga.key = self.id;
		manga.title = name;
		Ok(manga)
	}

	pub fn into_details(self) -> Result<Manga> {
		self.into_manga(true)
	}
}

impl TryFrom<Title> for Manga {
	type Error = AidokuError;

	fn try_from(title: Title) -> Result<Self> {
		title.into_manga(false)
	}
}

impl ChapterItem {
	pub fn into_chapter(
		self,
		languages: &[(String, String)],
		excluded_scanlators: &[String],
	) -> Option<Chapter> {
		if self.id.trim().is_empty()
			|| self.group.as_ref().is_some_and(|group| {
				excluded_scanlators.contains(&group.name.trim().to_lowercase())
			}) {
			return None;
		}
		let lang = self.lang.to_lowercase();
		let language = languages.iter().find(|(code, _)| *code == lang)?.1.clone();
		let title = self.name.filter(|name| {
			let label = name.trim();
			let label = label
				.strip_prefix(&format!("Vol.{} ", self.volume))
				.unwrap_or(label);
			let label = label
				.strip_prefix("Chapter ")
				.or_else(|| label.strip_prefix("Ch."))
				.unwrap_or(label)
				.trim();
			!name.trim().is_empty()
				&& !self
					.number
					.is_some_and(|number| label.parse::<f32>() == Ok(number))
		});
		// Avoid quoted literals, which the runner does not parse.
		let date_uploaded = self
			.created_at
			.as_deref()
			.and_then(|date| date.get(..19))
			.and_then(|date| parse_date(date.replace('T', " "), "yyyy-MM-dd HH:mm:ss"));
		Some(Chapter {
			url: Some(format!("{BASE_URL}/chapter-detail/{}", self.id)),
			key: self.id,
			title,
			chapter_number: self.number,
			volume_number: (self.volume > 0.0).then_some(self.volume),
			date_uploaded,
			scanlators: self.group.map(|group| vec![group.name]),
			language: Some(language),
			..Default::default()
		})
	}
}

pub fn chapter_list(
	entries: Vec<ChapterItem>,
	languages: &[(String, String)],
	excluded: &[String],
	deduplicate: bool,
) -> Vec<Chapter> {
	let mut chapters: Vec<_> = entries
		.into_iter()
		.filter_map(|mut entry| {
			let quality = (
				entry.likes.unwrap_or_default(),
				entry.views.unwrap_or_default(),
				entry.created_at.clone(),
			);
			let group = entry.group.as_mut().map(|group| {
				group
					.id
					.take()
					.filter(|id| !id.is_empty())
					.unwrap_or_else(|| group.name.trim().to_lowercase())
			});
			entry
				.into_chapter(languages, excluded)
				.map(|chapter| (quality, group, chapter))
		})
		.collect();
	if deduplicate {
		chapters.sort_by(|(a_quality, _, a), (b_quality, _, b)| {
			b.volume_number
				.partial_cmp(&a.volume_number)
				.unwrap_or(Ordering::Equal)
				.then_with(|| {
					b.chapter_number
						.partial_cmp(&a.chapter_number)
						.unwrap_or(Ordering::Equal)
				})
				.then_with(|| a.language.cmp(&b.language))
				.then_with(|| b_quality.cmp(a_quality))
				.then_with(|| a.key.cmp(&b.key))
		});
		// Keep every part from the winning group.
		let mut selected = None;
		let mut selected_group = None;
		chapters.retain(|(_, group, chapter)| {
			let Some(number) = chapter.chapter_number else {
				return true;
			};
			let same = selected
				.as_ref()
				.is_some_and(|(volume, previous, language)| {
					*volume == chapter.volume_number
						&& *previous == number
						&& *language == chapter.language
				});
			if same {
				*group == selected_group
			} else {
				selected = Some((chapter.volume_number, number, chapter.language.clone()));
				selected_group = group.clone();
				true
			}
		});
	}
	chapters
		.into_iter()
		.map(|(_, _, chapter)| chapter)
		.collect()
}
