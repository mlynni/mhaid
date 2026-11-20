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
	#[serde(default, deserialize_with = "null_default")]
	id: String,
	name: Option<String>,
	image: Option<Image>,
	#[serde(default, deserialize_with = "string_or_vec")]
	description: Vec<String>,
	#[serde(default, rename = "originalLanguage", deserialize_with = "null_default")]
	pub original_language: String,
	status: Option<String>,
	#[serde(
		default,
		rename = "availableTranslatedLanguages",
		deserialize_with = "string_or_vec"
	)]
	pub translated_languages: Vec<String>,
	#[serde(default, deserialize_with = "null_default")]
	pub tags: Vec<Tag>,
	#[serde(default, deserialize_with = "null_default")]
	author: Vec<Author>,
	#[serde(default, deserialize_with = "null_default")]
	is18plus: bool,
	#[serde(default, deserialize_with = "null_default")]
	pub recent_chapters: Vec<ChapterItem>,
}

fn null_default<'de, D, T>(deserializer: D) -> core::result::Result<T, D::Error>
where
	D: Deserializer<'de>,
	T: Default + Deserialize<'de>,
{
	Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
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
	Ok(match Option::<StringOrVec>::deserialize(deserializer)? {
		Some(StringOrVec::Single(value)) => vec![value],
		Some(StringOrVec::Multiple(values)) => values,
		None => Vec::new(),
	})
}

#[derive(Deserialize)]
struct Image {
	file: Option<String>,
	cdn_mangadex: Option<String>,
	cdn_mangaupdate: Option<String>,
	cdn_mangaupdates: Option<String>,
	cdn_bato: Option<String>,
	cover: Option<Cover>,
}

#[derive(Deserialize)]
struct Cover {
	path: Option<String>,
	size: Option<u64>,
}

#[derive(Deserialize)]
pub struct Tag {
	#[serde(default, deserialize_with = "null_default")]
	pub id: String,
	#[serde(default, deserialize_with = "null_default")]
	name: String,
}

#[derive(Deserialize)]
struct Author {
	#[serde(default, deserialize_with = "null_default")]
	name: String,
}

#[derive(Deserialize)]
pub struct ChapterItem {
	#[serde(default, deserialize_with = "null_default")]
	id: String,
	name: Option<String>,
	number: Option<f32>,
	#[serde(default, deserialize_with = "null_default")]
	volume: f32,
	#[serde(default, deserialize_with = "null_default")]
	lang: String,
	group: Option<Group>,
	created_at: Option<String>,
	likes: Option<u64>,
	views: Option<u64>,
}

#[derive(Deserialize)]
struct Group {
	id: Option<String>,
	#[serde(default, deserialize_with = "null_default")]
	name: String,
}

#[derive(Deserialize)]
pub struct ChapterDetail {
	pub chapter: ChapterPages,
}

#[derive(Deserialize)]
pub struct ChapterPages {
	pub title_id: Option<String>,
	#[serde(default, deserialize_with = "null_default")]
	pub pages: Vec<String>,
}

fn is_dead_cover_host(url: &str) -> bool {
	let lower = url.to_lowercase();
	lower.contains("hentaicdn.org") || lower.contains("hencover")
}

fn normalize_external_url(raw: &str) -> Option<String> {
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
	if cleaned.starts_with('/') {
		return Some(format!("{BASE_URL}{cleaned}"));
	}
	None
}

impl Image {
	// Order hosted and external mirrors, encoding the top backup after `#` for CoverImageProcessor.
	fn into_url(self) -> Option<String> {
		let oversized = self
			.cover
			.as_ref()
			.and_then(|cover| cover.size)
			.is_some_and(|size| size > 1_500_000);
		let hosted = self
			.cover
			.and_then(|cover| cover.path)
			.map(|path| path.trim().replace('\\', "/"))
			.filter(|path| !path.is_empty() && !is_dead_cover_host(path))
			.map(|path| {
				if path.starts_with("http://") || path.starts_with("https://") {
					path
				} else if path.starts_with("/covers/") || path.starts_with("covers/") {
					format!(
						"https://bulbasaur.poke-black-and-white.net/{}",
						path.trim_start_matches('/')
					)
				} else {
					format!("{CDN_URL}/{}", path.trim_start_matches('/'))
				}
			});
		let mangadex = self
			.cdn_mangadex
			.as_deref()
			.map(|raw| raw.trim().replace('\\', "/"))
			.filter(|raw| !raw.is_empty() && !is_dead_cover_host(raw))
			.map(|raw| {
				let url = if raw.starts_with("http://") || raw.starts_with("https://") {
					raw
				} else if raw.starts_with("//") {
					format!("https:{raw}")
				} else {
					format!(
						"https://uploads.mangadex.org/covers/{}",
						raw.trim_start_matches('/')
					)
				};
				if url.starts_with("https://uploads.mangadex.org/covers/")
					&& !url.ends_with(".256.jpg")
					&& !url.ends_with(".512.jpg")
				{
					format!("{url}.512.jpg")
				} else {
					url
				}
			});
		let file = self.file.as_deref().and_then(normalize_external_url);
		let mangaupdates = self
			.cdn_mangaupdates
			.as_deref()
			.and_then(normalize_external_url);
		let mangaupdate = self
			.cdn_mangaupdate
			.as_deref()
			.and_then(normalize_external_url);
		let bato = self.cdn_bato.as_deref().and_then(normalize_external_url);

		let raw_candidates = if oversized && mangadex.is_some() {
			[mangadex, hosted, file, mangaupdates, mangaupdate, bato]
		} else {
			[hosted, mangadex, file, mangaupdates, mangaupdate, bato]
		};
		let mut candidates = Vec::new();
		for candidate in raw_candidates.into_iter().flatten() {
			if !candidates.iter().any(|existing| existing == &candidate) {
				candidates.push(candidate);
			}
		}
		let mut iter = candidates.into_iter();
		let primary = iter.next()?;
		match iter.next() {
			Some(fallback) => Some(format!("{primary}#{fallback}")),
			None => Some(primary),
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
		let mut manga = Manga {
			cover: self.image.and_then(Image::into_url),
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

fn parse_iso_timestamp(raw: &str) -> Option<i64> {
	let s = raw.get(..19)?;
	let b = s.as_bytes();
	if b.get(4) != Some(&b'-')
		|| b.get(7) != Some(&b'-')
		|| !matches!(b.get(10), Some(b'T' | b' '))
		|| b.get(13) != Some(&b':')
		|| b.get(16) != Some(&b':')
	{
		return None;
	}
	let year: i64 = s.get(0..4)?.parse().ok()?;
	let month: i64 = s.get(5..7)?.parse().ok()?;
	let day: i64 = s.get(8..10)?.parse().ok()?;
	let hour: i64 = s.get(11..13)?.parse().ok()?;
	let min: i64 = s.get(14..16)?.parse().ok()?;
	let sec: i64 = s.get(17..19)?.parse().ok()?;
	if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
		return None;
	}
	// Howard Hinnant's civil_from_days / days_from_civil algorithm
	let y = year - i64::from(month <= 2);
	let era = if y >= 0 { y } else { y - 399 } / 400;
	let yoe = y - era * 400;
	let m = month + if month > 2 { -3 } else { 9 };
	let doy = (153 * m + 2) / 5 + day - 1;
	let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
	let days = era * 146_097 + doe - 719_468;
	Some(days * 86_400 + hour * 3_600 + min * 60 + sec)
}

impl ChapterItem {
	pub fn into_chapter(
		self,
		languages: &[(String, String)],
		excluded_scanlators: &[String],
	) -> Option<Chapter> {
		if self.id.trim().is_empty() {
			return None;
		}
		let lang = self.lang.to_lowercase();
		let language = languages.iter().find(|(code, _)| *code == lang)?.1.clone();
		if self.group.as_ref().is_some_and(|group| {
			excluded_scanlators.contains(&group.name.trim().to_lowercase())
		}) {
			return None;
		}
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
		let date_uploaded = self.created_at.as_deref().and_then(|date| {
			parse_iso_timestamp(date).or_else(|| {
				date.get(..19)
					.and_then(|d| parse_date(d.replace('T', " "), "yyyy-MM-dd HH:mm:ss"))
			})
		});
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
