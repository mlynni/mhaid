#![no_std]
use aidoku::{
	Chapter, ContentRating, DeepLinkHandler, DeepLinkResult, FilterValue, Home, HomeComponent,
	HomeComponentValue, HomeLayout, HomePartialResult, ImageRequestProvider, Link, LinkValue,
	Listing, ListingProvider, Manga, MangaPageResult, MangaStatus, Page, PageContent, PageContext,
	Result, Source, Viewer,
	alloc::{String, Vec, string::ToString, vec},
	helpers::uri::QueryParameters,
	imports::{
		html::{Document, Element},
		net::{Request, RequestError, Response},
		std::send_partial_result,
	},
	prelude::*,
};

const BASE_URL: &str = "https://mangapill.com";

struct MangaPill;

fn resolve_url(raw: &str) -> String {
	let trimmed = raw.trim();
	if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
		String::from(trimmed)
	} else if trimmed.starts_with("//") {
		format!("https:{trimmed}")
	} else if trimmed.starts_with('/') {
		format!("{BASE_URL}{trimmed}")
	} else {
		format!("{BASE_URL}/{trimmed}")
	}
}

fn path_key(raw: &str) -> String {
	let stripped = raw.trim().strip_prefix(BASE_URL).unwrap_or(raw.trim());
	if stripped.starts_with('/') {
		String::from(stripped)
	} else {
		format!("/{stripped}")
	}
}

fn parse_card(el: &Element) -> Option<Manga> {
	let link = el.select_first("a[href^='/manga/']")?;
	let href = link.attr("href").or_else(|| link.attr("abs:href"))?;
	let key = path_key(&href);
	let title = el.select_first("div.line-clamp-2")?.text()?;
	let cover = el
		.select_first("img")
		.and_then(|img| img.attr("data-src").or_else(|| img.attr("src")))
		.filter(|s| !s.trim().is_empty())
		.map(|s| resolve_url(&s));
	Some(Manga {
		key,
		title,
		cover,
		url: Some(resolve_url(&href)),
		..Default::default()
	})
}

fn parse_cards(html: &Document, selector: &str) -> Vec<Manga> {
	html.select(selector)
		.map(|els| els.filter_map(|el| parse_card(&el)).collect())
		.unwrap_or_default()
}

fn parse_status(text: &str) -> MangaStatus {
	let lower = text.to_ascii_lowercase();
	if lower.contains("publishing") {
		MangaStatus::Ongoing
	} else if lower.contains("finished") {
		MangaStatus::Completed
	} else if lower.contains("hiatus") {
		MangaStatus::Hiatus
	} else if lower.contains("discontinued") {
		MangaStatus::Cancelled
	} else {
		MangaStatus::Unknown
	}
}

fn parse_chapter_number(name: &str) -> Option<f32> {
	let trimmed = name.trim();
	trimmed
		.split_whitespace()
		.next_back()
		.and_then(|token| token.parse::<f32>().ok())
}

impl Source for MangaPill {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		let mut qs = QueryParameters::new();
		qs.push("page", Some(&page.to_string()));
		if let Some(q) = query.as_deref()
			&& !q.is_empty()
		{
			qs.push("q", Some(q));
		}

		for filter in filters {
			match filter {
				FilterValue::Select { id, value } if !value.is_empty() => {
					qs.push(&id, Some(&value));
				}
				FilterValue::MultiSelect { id, included, .. } if id == "genre" => {
					for genre in included {
						qs.push("genre", Some(&genre));
					}
				}
				_ => {}
			}
		}

		let url = format!("{BASE_URL}/search?{qs}");
		let html = Request::get(&url)?.html()?;
		let entries = parse_cards(&html, ".grid > div:not([class])");
		let has_next_page = html.select_first("a.btn.btn-sm").is_some();

		Ok(MangaPageResult {
			entries,
			has_next_page,
		})
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		let url = resolve_url(&manga.key);
		let html = Request::get(&url)?.html()?;

		if needs_details {
			if let Some(title) = html
				.select_first("h1")
				.and_then(|e| e.text())
				.filter(|t| !t.trim().is_empty())
			{
				manga.title = title;
			}

			manga.url = Some(url);
			manga.cover = html
				.select_first("div.container > div:first-child > div:first-child > img")
				.and_then(|e| e.attr("data-src").or_else(|| e.attr("src")))
				.filter(|s| !s.trim().is_empty())
				.map(|s| resolve_url(&s))
				.or(manga.cover);

			manga.description = html
				.select_first("div.container > div:first-child > div:last-child > div:nth-child(2) > p")
				.and_then(|e| e.text())
				.filter(|s| !s.trim().is_empty());

			let tags: Vec<String> = html
				.select("a[href*=genre]")
				.map(|els| els.filter_map(|e| e.text()).collect())
				.unwrap_or_default();
			if !tags.is_empty() {
				manga.content_rating = if tags
					.iter()
					.any(|t| matches!(t.as_str(), "Hentai" | "Yaoi" | "Yuri"))
				{
					ContentRating::NSFW
				} else if tags.iter().any(|t| t == "Ecchi") {
					ContentRating::Suggestive
				} else {
					ContentRating::Safe
				};
				manga.tags = Some(tags);
			}

			manga.status = html
				.select_first(
					"div.container > div:first-child > div:last-child > div:nth-child(3) > div:nth-child(2) > div",
				)
				.and_then(|e| e.text())
				.map(|s| parse_status(&s))
				.unwrap_or(MangaStatus::Unknown);

			manga.viewer = Viewer::RightToLeft;

			if needs_chapters {
				send_partial_result(&manga);
			}
		}

		if needs_chapters {
			let chapters = html
				.select("#chapters > div > a")
				.map(|els| {
					els.filter_map(|el| {
						let href = el.attr("href").or_else(|| el.attr("abs:href"))?;
						let name = el.text()?;
						Some(Chapter {
							key: path_key(&href),
							title: Some(name.clone()),
							chapter_number: parse_chapter_number(&name),
							url: Some(resolve_url(&href)),
							..Default::default()
						})
					})
					.collect()
				})
				.unwrap_or_default();
			manga.chapters = Some(chapters);
		}

		Ok(manga)
	}

	fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let url = resolve_url(&chapter.key);
		let html = Request::get(&url)?.html()?;
		Ok(html
			.select("picture img")
			.map(|els| {
				els.filter_map(|img| {
					let src = img
						.attr("data-src")
						.or_else(|| img.attr("src"))
						.filter(|s| !s.trim().is_empty())?;
					Some(Page {
						content: PageContent::url(resolve_url(&src)),
						..Default::default()
					})
				})
				.collect()
			})
			.unwrap_or_default())
	}
}

impl ListingProvider for MangaPill {
	fn get_manga_list(&self, listing: Listing, _page: i32) -> Result<MangaPageResult> {
		match listing.id.as_str() {
			"popular" => {
				let html = Request::get(format!("{BASE_URL}/"))?.html()?;
				Ok(MangaPageResult {
					entries: parse_cards(
						&html,
						"div:has(h4:contains(Trending)) > .grid > div:not([class])",
					),
					has_next_page: false,
				})
			}
			"latest" => {
				let html = Request::get(format!("{BASE_URL}/chapters"))?.html()?;
				Ok(MangaPageResult {
					entries: parse_cards(&html, ".grid > div:not([class])"),
					has_next_page: false,
				})
			}
			_ => bail!("Unknown listing"),
		}
	}
}

impl Home for MangaPill {
	fn get_home(&self) -> Result<HomeLayout> {
		send_partial_result(&HomePartialResult::Layout(HomeLayout {
			components: vec![
				HomeComponent {
					title: Some("Popular".into()),
					subtitle: None,
					value: HomeComponentValue::empty_scroller(),
				},
				HomeComponent {
					title: Some("Latest Updates".into()),
					subtitle: None,
					value: HomeComponentValue::empty_scroller(),
				},
			],
		}));

		let responses: [core::result::Result<Response, RequestError>; 2] =
			Request::send_all([Request::get(format!("{BASE_URL}/"))?, Request::get(format!("{BASE_URL}/chapters"))?])
				.try_into()
				.expect("requests vec length should be 2");

		let [home_res, chapters_res] = responses;

		let sections = [
			(
				home_res,
				"Popular",
				"popular",
				"div:has(h4:contains(Trending)) > .grid > div:not([class])",
			),
			(
				chapters_res,
				"Latest Updates",
				"latest",
				".grid > div:not([class])",
			),
		];

		for (res, title, listing_id, selector) in sections {
			let html = res?.get_html()?;
			let entries = parse_cards(&html, selector)
				.into_iter()
				.map(|manga| Link {
					title: manga.title.clone(),
					subtitle: None,
					image_url: manga.cover.clone(),
					value: Some(LinkValue::Manga(manga)),
				})
				.collect();
			send_partial_result(&HomePartialResult::Component(HomeComponent {
				title: Some(title.into()),
				subtitle: None,
				value: HomeComponentValue::Scroller {
					entries,
					listing: Some(Listing {
						id: listing_id.into(),
						name: title.into(),
						..Default::default()
					}),
				},
			}));
		}

		Ok(HomeLayout::default())
	}
}

impl ImageRequestProvider for MangaPill {
	fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
		Ok(Request::get(url)?.header("Referer", &format!("{BASE_URL}/")))
	}
}

impl DeepLinkHandler for MangaPill {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		let Some(path) = url.strip_prefix(BASE_URL) else {
			return Ok(None);
		};
		let clean = path.split(['?', '#']).next().unwrap_or(path);
		if clean.starts_with("/manga/") {
			return Ok(Some(DeepLinkResult::Manga {
				key: String::from(clean),
			}));
		}
		if clean.starts_with("/chapters/") {
			let html = Request::get(&url)?.html()?;
			if let Some(manga_href) = html
				.select_first("a[href^='/manga/']")
				.and_then(|e| e.attr("href"))
			{
				return Ok(Some(DeepLinkResult::Chapter {
					manga_key: path_key(&manga_href),
					key: String::from(clean),
				}));
			}
		}
		Ok(None)
	}
}

register_source!(
	MangaPill,
	ListingProvider,
	Home,
	ImageRequestProvider,
	DeepLinkHandler
);
