#![no_std]
mod api;
mod helpers;
mod home;
mod models;
mod settings;

use aidoku::{
	Chapter, CoverImageProcessor, DeepLinkHandler, DeepLinkResult, FilterValue, ImageRequestProvider,
	ImageResponse, Listing, ListingProvider, Manga, MangaPageResult, Page, PageContent, PageContext,
	Result, Source,
	alloc::{String, Vec, vec},
	imports::{canvas::ImageRef, net::Request, std::send_partial_result},
	prelude::*,
};
use api::{API_URL, BASE_URL, SORTS, SearchParams, get, parse, site_headers};
use core::cell::RefCell;
use helpers::{Link, parse_link, site_languages};
use models::{ApiResponse, ChapterDetail, ChapterItem, Title, chapter_list};

struct MangaBall {
	cover_fallbacks: RefCell<Vec<(String, String)>>,
}

impl Source for MangaBall {
	fn new() -> Self {
		Self {
			cover_fallbacks: RefCell::new(Vec::new()),
		}
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		if let Some(Link::Title(id)) = query.as_deref().and_then(parse_link)
			&& let Ok(found) = get::<ApiResponse<Title>>(&format!("{API_URL}/title/detail/{id}"))
		{
			return Ok(MangaPageResult {
				entries: vec![Manga::try_from(found.data)?],
				has_next_page: false,
			});
		}

		let mut search = SearchParams::new(query);
		for filter in filters {
			match filter {
				FilterValue::Sort {
					index, ascending, ..
				} => {
					if let Some(sort) = SORTS.get(index as usize) {
						search.sort = sort;
					}
					search.ascending = ascending;
				}
				FilterValue::Select { id, value } if !value.is_empty() => match id.as_str() {
					"type" => search.kind = if value == "any" { String::new() } else { value },
					"status" => search.status = value,
					"demographic" => search.demographic = value,
					"rating" => search.adult_mode = value,
					"tag_mode" => search.tag_mode = value,
					_ => {}
				},
				FilterValue::MultiSelect {
					id,
					included,
					excluded,
				} => match id.as_str() {
					"genre" | "theme" | "format" | "content" => {
						search.included_tags.extend(included);
						search.excluded_tags.extend(excluded);
					}
					_ => {}
				},
				_ => {}
			}
		}
		// A tag included in a search overrides its exclusion in settings.
		let included = &search.included_tags;
		search.excluded_tags.retain(|id| !included.contains(id));

		search.page(page)
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		if needs_details {
			let url = format!("{API_URL}/title/detail/{}", manga.key);
			let details = get::<ApiResponse<Title>>(&url)?.data.into_details()?;
			manga.copy_from(details);
			if needs_chapters {
				send_partial_result(&manga);
			}
		}

		if needs_chapters {
			let languages = site_languages(settings::languages());
			let excluded = settings::excluded_scanlators();
			let url = format!("{API_URL}/chapter/chapter-listing-by-title-id");
			// The API rejects a content type that carries a charset.
			let request = site_headers(Request::post(url)?)
				.header("Content-Type", "application/json")
				.body(format!(r#"{{"title_id":"{}"}}"#, manga.key));
			let chapters = parse::<ApiResponse<Vec<ChapterItem>>>(request.send()?)?.data;
			manga.chapters = Some(chapter_list(
				chapters,
				&languages,
				&excluded,
				settings::deduplicate_chapters(),
			));
		}

		Ok(manga)
	}

	fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let url = format!("{API_URL}/chapter-detail?chapter_id={}", chapter.key);
		let pages = get::<ApiResponse<ChapterDetail>>(&url)?.data.chapter.pages;
		if pages.is_empty() || pages.iter().any(|url| url.trim().is_empty()) {
			bail!("This chapter has missing pages");
		}
		Ok(pages
			.into_iter()
			.map(|url| Page {
				content: PageContent::url(url),
				..Default::default()
			})
			.collect())
	}
}

impl ListingProvider for MangaBall {
	fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		let Some(sort) = SORTS.iter().find(|sort| **sort == listing.id) else {
			bail!("Unknown listing");
		};
		SearchParams::sorted(sort).page(page)
	}
}

fn build_image_request(url: &str, is_page: bool) -> Result<Request> {
	let req = Request::get(url)?.header(
		"Accept",
		"image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
	);
	if url.contains("mangaball.") || url.contains("poke-black-and-white.net") {
		Ok(req.header("Referer", &format!("{BASE_URL}/")))
	} else if url.contains("comix.to") || url.contains("comix.ws") {
		Ok(req.header("Referer", "https://comix.to/"))
	} else if is_page {
		Ok(req.header("Referer", &format!("{BASE_URL}/")))
	} else {
		Ok(req)
	}
}

impl ImageRequestProvider for MangaBall {
	// Avoid sending Referer headers to external cover CDNs (e.g. uploads.mangadex.org) that reject them.
	fn get_image_request(&self, url: String, context: Option<PageContext>) -> Result<Request> {
		if context.is_none()
			&& let Some((primary, fallback)) = url.split_once('#')
		{
			let mut fallbacks = self.cover_fallbacks.borrow_mut();
			if !fallbacks.iter().any(|(p, _)| p == primary) {
				if fallbacks.len() >= 64 {
					fallbacks.remove(0);
				}
				fallbacks.push((String::from(primary), String::from(fallback)));
			}
			return build_image_request(primary, false);
		}
		build_image_request(&url, context.is_some())
	}
}

impl CoverImageProcessor for MangaBall {
	fn process_cover_image(&self, response: ImageResponse) -> Result<ImageRef> {
		if response.code < 400 && response.image.width() > 0.0 {
			return Ok(response.image);
		}
		if let Some(raw_url) = response.request.url.as_deref() {
			let primary = raw_url.split('#').next().unwrap_or(raw_url);
			let fallback = raw_url.split_once('#').map(|(_, fb)| String::from(fb)).or_else(|| {
				self.cover_fallbacks
					.borrow()
					.iter()
					.find(|(p, _)| p == primary)
					.map(|(_, fb)| fb.clone())
			});
			if let Some(fallback_url) = fallback
				&& let Ok(req) = build_image_request(&fallback_url, false)
				&& let Ok(resp) = req.send()
				&& resp.status_code() < 400
				&& let Ok(bytes) = resp.get_data()
				&& !bytes.is_empty()
			{
				return Ok(ImageRef::new(&bytes));
			}
			if !primary.is_empty()
				&& let Ok(req) = Request::get(primary)
				&& let Ok(resp) = req.send()
				&& resp.status_code() < 400
				&& let Ok(bytes) = resp.get_data()
				&& !bytes.is_empty()
			{
				return Ok(ImageRef::new(&bytes));
			}
		}
		Ok(response.image)
	}
}

impl DeepLinkHandler for MangaBall {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		Ok(match parse_link(&url) {
			Some(Link::Title(key)) => Some(DeepLinkResult::Manga { key }),
			Some(Link::Chapter(key)) => {
				let url = format!("{API_URL}/chapter-detail?chapter_id={key}");
				get::<ApiResponse<ChapterDetail>>(&url)?
					.data
					.chapter
					.title_id
					.map(|manga_key| DeepLinkResult::Chapter { manga_key, key })
			}
			None => None,
		})
	}
}

register_source!(
	MangaBall,
	Home,
	ListingProvider,
	ImageRequestProvider,
	CoverImageProcessor,
	DeepLinkHandler
);
