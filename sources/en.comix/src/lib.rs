#![no_std]
use aidoku::{
	BaseUrlProvider, Chapter, CoverImageProcessor, DeepLinkHandler, DeepLinkResult, FilterValue,
	HashMap, Home, HomeComponent, HomeComponentValue, HomeLayout, HomePartialResult,
	ImageRequestProvider, ImageResponse, Link, LinkValue, Listing, ListingProvider, Manga,
	MangaPageResult, MangaWithChapter, NotificationHandler, Page, PageContent, PageContext,
	PageImageProcessor, Result, Source, WebLoginHandler,
	alloc::{String, Vec, string::ToString, vec},
	helpers::uri::QueryParameters,
	imports::{
		canvas::ImageRef,
		net::Request,
		std::send_partial_result,
	},
	prelude::*,
};
use core::cell::RefCell;

mod descrambler;
mod helpers;
mod models;
mod settings;
mod web;

use models::*;
use settings::{VERIFY_KEY, VERIFY_WS_KEY};

struct Comix;

const SORT_VALUES: &[&str] = &[
	"relevance",
	"chapter_updated_at",
	"created_at",
	"title",
	"year",
	"score",
	"views_7d",
	"views_30d",
	"views_90d",
	"views_total",
	"follows_total",
];

fn parse_chapter_list(
	chapters_res: ChaptersHarvestResult,
	base_url: &str,
) -> Vec<Chapter> {
	let deduplicate = settings::dedup_chapters();
	let blacklist = settings::scanlator_blacklist();

	let filtered = chapters_res.chapters.into_iter().filter(|ch| {
		if blacklist.is_empty() {
			return true;
		}
		!ch.group.as_ref().is_some_and(|g| {
			let name_lower = g.name.trim().to_ascii_lowercase();
			let id_str = g.id.map(|id| id.to_string()).unwrap_or_default();
			blacklist
				.iter()
				.any(|b| b == &name_lower || (!id_str.is_empty() && b == &id_str))
		})
	});

	let selected: Vec<HarvestedChapter> = if deduplicate {
		let mut map: HashMap<String, HarvestedChapter> = HashMap::new();
		for item in filtered {
			helpers::dedup_insert(&mut map, item);
		}
		let mut list: Vec<HarvestedChapter> = map.into_values().collect();
		list.sort_by(|a, b| {
			b.number
				.partial_cmp(&a.number)
				.unwrap_or(core::cmp::Ordering::Equal)
				.then_with(|| b.id.cmp(&a.id))
		});
		list
	} else {
		filtered.collect()
	};

	selected
		.into_iter()
		.map(|ch| {
			let date_uploaded = helpers::extract_relative_date(&ch.text);
			let scanlators = ch.group.map(|g| vec![g.name]);
			let url = helpers::resolve_url(base_url, &ch.url);
			Chapter {
				key: ch.id.to_string(),
				title: ch.name.filter(|s| !s.trim().is_empty()),
				chapter_number: Some(ch.number),
				volume_number: ch.volume,
				date_uploaded,
				scanlators,
				url: Some(url),
				..Default::default()
			}
		})
		.collect()
}

fn find_detail_manga(data: &InitialData, hid: Option<&str>) -> Option<ComixManga> {
	if let Some(target_hid) = hid
		&& let Some((_, val)) = data
			.queries
			.iter()
			.find(|(k, _)| k.contains("\"detail\"") && k.contains(target_hid))
		&& let Ok(manga) = serde_json::from_value::<ComixManga>(val.clone())
	{
		return Some(manga);
	}
	data.queries
		.iter()
		.find(|(k, _)| k.contains("\"detail\""))
		.and_then(|(_, val)| serde_json::from_value::<ComixManga>(val.clone()).ok())
}

fn extract_home_list(data: &InitialData, needle: &str) -> Vec<ComixManga> {
	let Some((_, val)) = data.queries.iter().find(|(k, _)| k.contains(needle)) else {
		return Vec::new();
	};
	if val.is_array() {
		serde_json::from_value::<Vec<ComixManga>>(val.clone()).unwrap_or_default()
	} else {
		serde_json::from_value::<InitialListPayload>(val.clone())
			.map(|p| p.items)
			.unwrap_or_default()
	}
}

fn browse_with_defaults(
	sort: &str,
	extra_types: Option<&str>,
	page: i32,
) -> Result<MangaPageResult> {
	let base_url = settings::base_url();
	let mut qs = QueryParameters::new();

	qs.push("sort", Some(sort));
	qs.push("page", Some(&page.to_string()));

	let rating = settings::content_rating();
	let ratings = settings::content_ratings_up_to(&rating);
	qs.push("content_rating", Some(&ratings.join(",")));

	if let Some(types) = extra_types {
		qs.push("types", Some(types));
	} else {
		let types = settings::default_types();
		if !types.is_empty() && types.len() < 4 {
			qs.push("types", Some(&types.join(",")));
		}
	}

	let demos = settings::default_demographics();
	let mut excluded_genres: Vec<String> = settings::blocked_genres();
	for d in ["3", "4", "1", "2"] {
		if !demos.iter().any(|s| s == d) && !excluded_genres.iter().any(|s| s == d) {
			excluded_genres.push(d.into());
		}
	}
	if !excluded_genres.is_empty() {
		qs.push("genres_ex", Some(&excluded_genres.join(",")));
	}

	let url = format!("{base_url}/browse?{qs}");
	harvest_and_map_browse(&base_url, &url)
}

fn harvest_and_map_browse(base_url: &str, browse_url: &str) -> Result<MangaPageResult> {
	let res = web::harvest_browse(browse_url)?;
	let quality = settings::thumbnail_quality();
	let per_page = res.per_page.unwrap_or(28);
	let count = res.items.len();

	let entries = res
		.items
		.into_iter()
		.map(|item| {
			let cover = item
				.cover
				.as_deref()
				.filter(|s| !s.trim().is_empty())
				.map(|raw| helpers::apply_cover_quality(&helpers::resolve_url(base_url, raw), &quality));
			let url = helpers::resolve_url(base_url, &item.url);
			Manga {
				key: item.hid,
				title: item.title,
				cover,
				url: Some(url),
				..Default::default()
			}
		})
		.collect();

	Ok(MangaPageResult {
		entries,
		has_next_page: (count as i32) >= per_page,
	})
}

impl Source for Comix {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		let base_url = settings::base_url();
		let query_text = query
			.as_deref()
			.map(str::trim)
			.filter(|s| !s.is_empty())
			.map(String::from);

		let mut qs = QueryParameters::new();
		if let Some(ref q) = query_text {
			qs.push("q", Some(q));
		}
		qs.push("page", Some(&page.to_string()));

		let mut sort_param: Option<String> = None;
		let mut content_rating_override: Option<String> = None;
		let mut selected_types: Option<Vec<String>> = None;
		let mut genres_in: Vec<String> = Vec::new();
		let mut genres_ex: Vec<String> = settings::blocked_genres();
		let mut demos_in: Vec<String> = Vec::new();
		let mut excluded_demos: Vec<String> = Vec::new();
		let mut statuses: Vec<String> = Vec::new();
		let mut genres_mode: Option<String> = None;
		let mut min_chap: Option<String> = None;
		let mut max_chap: Option<String> = None;
		let mut year_from: Option<String> = None;
		let mut year_to: Option<String> = None;
		let mut author_ids: Vec<String> = Vec::new();
		let mut artist_ids: Vec<String> = Vec::new();

		let default_demos = settings::default_demographics();
		let mut demo_filter_seen = false;

		for filter in filters {
			match filter {
				FilterValue::Sort {
					index,
					ascending,
					..
				} => {
					let selected = SORT_VALUES
						.get(index as usize)
						.copied()
						.unwrap_or("relevance");
					let field = if selected == "relevance" && query_text.is_none() {
						"chapter_updated_at"
					} else {
						selected
					};
					let dir = if ascending { "asc" } else { "desc" };
					sort_param = Some(format!("{field}:{dir}"));
				}
				FilterValue::Select { id, value } => match id.as_str() {
					"content_rating" => {
						if value != "default" {
							content_rating_override = Some(value);
						}
					}
					"genres_mode" => {
						if !value.is_empty() {
							genres_mode = Some(value);
						}
					}
					"year_from" => {
						if !value.is_empty() {
							year_from = Some(value);
						}
					}
					"year_to" => {
						if !value.is_empty() {
							year_to = Some(value);
						}
					}
					_ => {}
				},
				FilterValue::MultiSelect {
					id,
					included,
					excluded,
				} => match id.as_str() {
					"types" => {
						if !included.is_empty() {
							selected_types = Some(included);
						}
					}
					"genres" | "formats" => {
						for v in included {
							if let Some(pos) = genres_ex.iter().position(|x| x == &v) {
								genres_ex.swap_remove(pos);
							}
							if !genres_in.contains(&v) {
								genres_in.push(v);
							}
						}
						for v in excluded {
							if !genres_ex.contains(&v) {
								genres_ex.push(v);
							}
						}
					}
					"demos" => {
						demo_filter_seen = true;
						for v in included {
							if !demos_in.contains(&v) {
								demos_in.push(v);
							}
						}
						for v in excluded {
							if !excluded_demos.contains(&v) {
								excluded_demos.push(v);
							}
						}
					}
					"statuses" => {
						for v in included {
							if !statuses.contains(&v) {
								statuses.push(v);
							}
						}
					}
					_ => {}
				},
				FilterValue::Text { id, value } => {
					let trimmed = value.trim();
					if trimmed.is_empty() {
						continue;
					}
					match id.as_str() {
						"tags" => {
							for tid in web::fetch_tag_ids(&base_url, "tag", trimmed) {
								if !genres_in.contains(&tid) {
									genres_in.push(tid);
								}
							}
						}
						"author" => {
							for aid in web::fetch_tag_ids(&base_url, "author", trimmed) {
								if !author_ids.contains(&aid) {
									author_ids.push(aid);
								}
							}
						}
						"artist" => {
							for aid in web::fetch_tag_ids(&base_url, "artist", trimmed) {
								if !artist_ids.contains(&aid) {
									artist_ids.push(aid);
								}
							}
						}
						"min_chap" => {
							if let Ok(v) = trimmed.parse::<i32>()
								&& v > 0
							{
								min_chap = Some(v.to_string());
							}
						}
						"max_chap" => {
							if let Ok(v) = trimmed.parse::<i32>()
								&& v > 0
							{
								max_chap = Some(v.to_string());
							}
						}
						_ => {}
					}
				}
				_ => {}
			}
		}

		let sort_final = sort_param.unwrap_or_else(|| {
			if query_text.is_some() {
				"relevance:desc".into()
			} else {
				"chapter_updated_at:desc".into()
			}
		});
		qs.push("sort", Some(&sort_final));

		let rating_str = match content_rating_override.as_deref() {
			Some(val) => val.to_string(),
			None => settings::content_rating(),
		};
		let ratings = settings::content_ratings_up_to(&rating_str);
		qs.push("content_rating", Some(&ratings.join(",")));

		let types_to_apply = selected_types.unwrap_or_else(settings::default_types);
		if !types_to_apply.is_empty() && types_to_apply.len() < 4 {
			qs.push("types", Some(&types_to_apply.join(",")));
		}

		if !demo_filter_seen {
			for d in ["3", "4", "1", "2"] {
				if !default_demos.iter().any(|s| s == d) && !excluded_demos.iter().any(|s| s == d) {
					excluded_demos.push(d.into());
				}
			}
		}
		if !demos_in.is_empty() {
			qs.push("demos", Some(&demos_in.join(",")));
		}
		for ex_d in excluded_demos {
			if !genres_ex.contains(&ex_d) {
				genres_ex.push(ex_d);
			}
		}

		if !genres_in.is_empty() {
			qs.push("genres_in", Some(&genres_in.join(",")));
			if let Some(mode) = genres_mode {
				qs.push("genres_mode", Some(&mode));
			}
		}
		if !genres_ex.is_empty() {
			qs.push("genres_ex", Some(&genres_ex.join(",")));
		}
		if !statuses.is_empty() {
			qs.push("statuses", Some(&statuses.join(",")));
		}
		if let Some(v) = min_chap {
			qs.push("min_chap", Some(&v));
		}
		if let Some(v) = max_chap {
			qs.push("max_chap", Some(&v));
		}
		if let Some(v) = year_from {
			qs.push("year_from", Some(&v));
		}
		if let Some(v) = year_to {
			qs.push("year_to", Some(&v));
		}
		if !author_ids.is_empty() {
			qs.push("authors", Some(&author_ids.join(",")));
		}
		if !artist_ids.is_empty() {
			qs.push("artists", Some(&artist_ids.join(",")));
		}

		let url = format!("{base_url}/browse?{qs}");
		harvest_and_map_browse(&base_url, &url)
	}

	fn get_manga_update(
		&self,
		manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		if !needs_details && !needs_chapters {
			return Ok(manga);
		}

		let base_url = settings::base_url();
		let title_path = manga
			.url
			.as_deref()
			.map(helpers::extract_path)
			.filter(|p| p.starts_with("/title/") && p.len() > 7)
			.unwrap_or_else(|| format!("/title/{}", manga.key));
		let title_url = helpers::resolve_url(&base_url, &title_path);

		let manga_cell = RefCell::new(manga);
		let (_, chapters_opt) =
			web::harvest_title_page(&title_url, needs_details, needs_chapters, |initial_data| {
				let hid = manga_cell.borrow().key.clone();
				if let Some(comix_manga) = find_detail_manga(initial_data, Some(&hid)) {
					let detailed = comix_manga.into_manga(&base_url, true);
					let mut m = manga_cell.borrow_mut();
					m.copy_from(detailed);
					if needs_chapters {
						send_partial_result(&*m);
					}
				}
			})?;

		let mut manga = manga_cell.into_inner();
		if needs_chapters && let Some(chapters_res) = chapters_opt {
			manga.chapters = Some(parse_chapter_list(chapters_res, &base_url));
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let base_url = settings::base_url();

		let valid_chapter_path = chapter
			.url
			.as_deref()
			.map(helpers::extract_path)
			.filter(|path| {
				if let Some(rest) = path.strip_prefix("/title/")
					&& let Some((manga_seg, ch_seg)) = rest.split_once('/')
				{
					manga_seg.contains('-') && !ch_seg.is_empty() && path.contains("-chapter-")
				} else {
					false
				}
			});

		let chapter_url = if let Some(path) = valid_chapter_path {
			helpers::resolve_url(&base_url, &path)
		} else {
			let mut slug_path = manga
				.url
				.as_deref()
				.map(helpers::extract_path)
				.filter(|p| p.starts_with("/title/") && p.contains('-'))
				.unwrap_or_default();

			if slug_path.is_empty() {
				let lookup_url = format!("{base_url}/title/{}", manga.key);
				if let Ok(initial) = web::fetch_initial_data(&lookup_url)
					&& let Some(detail) = find_detail_manga(&initial, Some(&manga.key))
					&& let Some(u) = detail.url
				{
					slug_path = helpers::extract_path(&u);
				}
			}
			if slug_path.is_empty() {
				slug_path = format!("/title/{}", manga.key);
			}

			let num_str = helpers::format_chapter_num(chapter.chapter_number.unwrap_or(0.0));
			helpers::resolve_url(
				&base_url,
				&format!("{}/{}-chapter-{}", slug_path.trim_end_matches('/'), chapter.key, num_str),
			)
		};

		let harvested = web::harvest_chapter_pages(&chapter_url)?;
		if harvested.pages.is_empty() {
			bail!("No pages found for this chapter");
		}

		Ok(harvested
			.pages
			.into_iter()
			.map(|p| {
				let mut context = PageContext::new();
				context.insert("url".into(), p.url.clone());
				Page {
					content: PageContent::url_context(p.url, context),
					..Default::default()
				}
			})
			.collect())
	}
}

impl ImageRequestProvider for Comix {
	fn get_image_request(&self, url: String, context: Option<PageContext>) -> Result<Request> {
		let _ = context;
		// Comix image domains reject requests that send Referer / Origin headers.
		Ok(Request::get(url)?)
	}
}

impl CoverImageProcessor for Comix {
	fn process_cover_image(&self, response: ImageResponse) -> Result<ImageRef> {
		if response.code < 400 && response.image.width() > 0.0 {
			return Ok(response.image);
		}
		if let Some(orig_url) = response.request.url.as_deref() {
			let mut candidates: Vec<String> = Vec::new();
			if orig_url.contains("@280.") {
				candidates.push(orig_url.replace("@280.", "."));
			} else if orig_url.ends_with(".jpg") {
				candidates.push(orig_url.replacen(".jpg", "@280.jpg", 1));
			}
			if let Some(alt) = settings::alternate_mirror(orig_url) {
				candidates.push(alt.clone());
				if alt.contains("@280.") {
					candidates.push(alt.replace("@280.", "."));
				}
			}
			for candidate in candidates {
				if let Ok(req) = Request::get(&candidate)
					&& let Ok(resp) = req.send()
					&& resp.status_code() < 400
					&& let Ok(bytes) = resp.get_data()
					&& !bytes.is_empty()
				{
					return Ok(ImageRef::new(&bytes));
				}
			}
		}
		Ok(response.image)
	}
}

impl PageImageProcessor for Comix {
	fn process_page_image(
		&self,
		response: ImageResponse,
		context: Option<PageContext>,
	) -> Result<ImageRef> {
		descrambler::process_page_image(response, context)
	}
}

impl Home for Comix {
	fn get_home(&self) -> Result<HomeLayout> {
		send_partial_result(&HomePartialResult::Layout(HomeLayout {
			components: vec![
				HomeComponent {
					title: Some("Trending".into()),
					subtitle: None,
					value: HomeComponentValue::empty_scroller(),
				},
				HomeComponent {
					title: Some("Most Followed".into()),
					subtitle: None,
					value: HomeComponentValue::empty_scroller(),
				},
				HomeComponent {
					title: Some("Latest Updates".into()),
					subtitle: None,
					value: HomeComponentValue::empty_scroller(),
				},
				HomeComponent {
					title: Some("Recently Added".into()),
					subtitle: None,
					value: HomeComponentValue::empty_manga_chapter_list(),
				},
			],
		}));

		let base_url = settings::base_url();
		let rating = settings::content_rating();
		let allowed_ratings = settings::content_ratings_up_to(&rating);
		let allowed_types = settings::default_types();
		let blocked_genres = settings::blocked_genres();

		let data = web::fetch_initial_data(&format!("{base_url}/home"))?;

		let sections = [
			("\"trending\"", "Trending"),
			("\"follows\"", "Most Followed"),
			("\"scope\":\"hot\"", "Latest Updates"),
		];

		for (needle, title) in sections {
			let entries: Vec<Link> = extract_home_list(&data, needle)
				.into_iter()
				.filter(|m| m.matches_settings(&allowed_ratings, &allowed_types, &blocked_genres))
				.take(20)
				.map(|m| {
					let manga = m.into_manga(&base_url, false);
					Link {
						title: manga.title.clone(),
						subtitle: None,
						image_url: manga.cover.clone(),
						value: Some(LinkValue::Manga(manga)),
					}
				})
				.collect();

			send_partial_result(&HomePartialResult::Component(HomeComponent {
				title: Some(title.into()),
				subtitle: None,
				value: HomeComponentValue::Scroller {
					entries,
					listing: Some(Listing {
						id: title.into(),
						name: title.into(),
						..Default::default()
					}),
				},
			}));
		}

		{
			let recent_items = extract_home_list(&data, "\"created_at\"");
			let entries: Vec<MangaWithChapter> = recent_items
				.into_iter()
				.filter(|m| m.matches_settings(&allowed_ratings, &allowed_types, &blocked_genres))
				.take(15)
				.map(|m| {
					let latest_ch = m.latest_chapter;
					let manga = m.into_manga(&base_url, false);
					MangaWithChapter {
						manga,
						chapter: Chapter {
							chapter_number: latest_ch,
							..Default::default()
						},
					}
				})
				.collect();

			let title = "Recently Added";
			send_partial_result(&HomePartialResult::Component(HomeComponent {
				title: Some(title.into()),
				subtitle: None,
				value: HomeComponentValue::MangaChapterList {
					page_size: None,
					entries,
					listing: Some(Listing {
						id: title.into(),
						name: title.into(),
						..Default::default()
					}),
				},
			}));
		}

		Ok(HomeLayout::default())
	}
}

impl ListingProvider for Comix {
	fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		match listing.id.as_str() {
			"Popular" => browse_with_defaults("score:desc", None, page),
			"Latest Updates" | "Latest Updates (Hot)" => {
				browse_with_defaults("chapter_updated_at:desc", None, page)
			}
			"Trending" | "Most Recent Popular" => browse_with_defaults("views_7d:desc", None, page),
			"Most Followed" | "Most Follows New Comics" => {
				browse_with_defaults("follows_total:desc", None, page)
			}
			"Recently Added" => browse_with_defaults("created_at:desc", None, page),
			"Trending Webtoon" => browse_with_defaults("views_30d:desc", Some("manhua,manhwa"), page),
			"Trending Manga" => browse_with_defaults("views_30d:desc", Some("manga"), page),
			_ => bail!("Unknown listing: {}", listing.id),
		}
	}
}

impl BaseUrlProvider for Comix {
	fn get_base_url(&self) -> Result<String> {
		Ok(settings::base_url())
	}
}

impl NotificationHandler for Comix {
	fn handle_notification(&self, notification: String) {
		if notification == "resetFilters" {
			settings::reset_filters();
		}
	}
}

impl DeepLinkHandler for Comix {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		let path = [
			"https://comix.to/",
			"https://www.comix.to/",
			"https://comix.ws/",
			"https://www.comix.ws/",
		]
		.iter()
		.find_map(|prefix| url.strip_prefix(prefix));

		let Some(path) = path else {
			return Ok(None);
		};

		let mut segments = path.split('/');
		if let (Some("title"), Some(manga_segment)) = (segments.next(), segments.next()) {
			let manga_key = manga_segment.split('-').next().unwrap_or(manga_segment);
			if manga_key.is_empty() {
				return Ok(None);
			}
			if let Some(chapter_segment) = segments.next() {
				let chapter_key = chapter_segment.split('-').next().unwrap_or("");
				if !chapter_key.is_empty() {
					return Ok(Some(DeepLinkResult::Chapter {
						manga_key: manga_key.to_string(),
						key: chapter_key.to_string(),
					}));
				}
			}
			return Ok(Some(DeepLinkResult::Manga {
				key: manga_key.to_string(),
			}));
		}

		Ok(None)
	}
}

impl WebLoginHandler for Comix {
	fn handle_web_login(&self, key: String, cookies: HashMap<String, String>) -> Result<bool> {
		if key == VERIFY_KEY || key == VERIFY_WS_KEY {
			if let Some(pass) = cookies.get("waf_pass")
				&& !pass.trim().is_empty()
			{
				return Ok(true);
			}
			return Ok(false);
		}
		Ok(false)
	}
}

register_source!(
	Comix,
	Home,
	ListingProvider,
	BaseUrlProvider,
	ImageRequestProvider,
	CoverImageProcessor,
	PageImageProcessor,
	NotificationHandler,
	DeepLinkHandler,
	WebLoginHandler
);
