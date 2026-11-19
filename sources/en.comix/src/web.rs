use crate::{
	helpers,
	models::{
		BrowseHarvestResult, ChaptersHarvestResult, InitialData, PagesHarvestResult,
		TagsSearchResponse,
	},
};
use aidoku::{
	Result,
	alloc::{String, Vec, string::ToString},
	helpers::uri::encode_uri_component,
	imports::{
		js::WebView,
		net::Request,
		std::sleep,
	},
	prelude::*,
};
use serde::{Deserialize, de::DeserializeOwned};

const HARVEST_BROWSE_JS: &str = include_str!("../assets/harvest_browse.js");
const HARVEST_CHAPTERS_JS: &str = include_str!("../assets/harvest_chapters.js");
const CHAPTER_PAGES_JS: &str = include_str!("../assets/chapter_pages.js");

const WAF_ERROR_MSG: &str = "Comix requested a security check. Open Comix Settings → Site Check (Verify comix.to / Verify comix.ws) or switch Mirror.";

const READY_PROBE_JS: &str = "(() => JSON.stringify({ ready: document.readyState || 'loading', href: location.href || '', path: location.pathname || '', title: document.title || '', hasInitialData: !!document.querySelector('script#initial-data'), waf: (location.pathname || '').startsWith('/@waf/') || document.title === 'Security check' || document.title === 'Just a moment...' || !!document.querySelector('iframe[src*=\"/@waf/challenge\"]') }))()";

const POLL_RESULT_JS: &str = "(() => JSON.stringify({ waf: !!window.__comixWaf || (location.pathname || '').startsWith('/@waf/') || document.title === 'Security check' || !!document.querySelector('iframe[src*=\"/@waf/challenge\"]'), done: !!window.__comixResult, error: window.__comixError || null, result: window.__comixResult || null }))()";

const READ_INITIAL_DATA_JS: &str = "(() => { const s = document.querySelector('script#initial-data'); return s ? (s.textContent || '') : ''; })()";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadyProbe {
	ready: String,
	href: String,
	#[serde(default)]
	has_initial_data: bool,
	waf: bool,
}

#[derive(Deserialize)]
struct PollState {
	waf: bool,
	done: bool,
	error: Option<String>,
	result: Option<serde_json::Value>,
}

/// How long to let a security check resolve on its own before giving up.
/// The Turnstile widget on `/@waf/challenge` usually passes automatically in a real
/// WebView and then redirects, so we wait for that instead of failing immediately.
const WAF_AUTO_PASS_SECS: i32 = 25;

enum Ready {
	Page,
	Waf,
}

enum Polled<T> {
	Done(T),
	Waf,
}

fn wait_webview_ready(wv: &WebView, timeout_secs: i32, wait_for_initial_data: bool) -> Result<Ready> {
	for _ in 0..timeout_secs {
		if let Ok(raw) = wv.eval(READY_PROBE_JS)
			&& let Ok(probe) = serde_json::from_str::<ReadyProbe>(&raw)
		{
			if probe.waf {
				return Ok(Ready::Waf);
			}
			let loaded = !probe.href.is_empty()
				&& probe.href != "about:blank"
				&& probe.ready != "loading";
			if loaded && (!wait_for_initial_data || probe.has_initial_data || probe.ready == "complete") {
				return Ok(Ready::Page);
			}
		}
		sleep(1);
	}
	bail!("Timed out waiting for Comix WebView to load")
}

/// Waits for the security check to clear, then reloads `url` and waits for the real page.
/// Fails with `WAF_ERROR_MSG` if the check needs manual interaction or never clears.
fn recover_from_waf(wv: &WebView, url: &str, wait_for_initial_data: bool) -> Result<()> {
	let mut cleared = false;
	for _ in 0..WAF_AUTO_PASS_SECS {
		sleep(1);
		if let Ok(raw) = wv.eval(READY_PROBE_JS)
			&& let Ok(probe) = serde_json::from_str::<ReadyProbe>(&raw)
			&& !probe.waf
			&& probe.ready != "loading"
		{
			cleared = true;
			break;
		}
	}
	if !cleared {
		bail!("{WAF_ERROR_MSG}");
	}
	// The check may redirect to `/` instead of the page we wanted, so load it again.
	wv.load(Request::get(url)?)?;
	match wait_webview_ready(wv, 20, wait_for_initial_data)? {
		Ready::Page => Ok(()),
		Ready::Waf => bail!("{WAF_ERROR_MSG}"),
	}
}

/// Opens `url` in a fresh WebView, recovering from a security check when possible.
fn open_page(url: &str, wait_for_initial_data: bool) -> Result<WebView> {
	let wv = WebView::new();
	wv.load(Request::get(url)?)?;
	match wait_webview_ready(&wv, 20, wait_for_initial_data)? {
		Ready::Page => {}
		Ready::Waf => recover_from_waf(&wv, url, wait_for_initial_data)?,
	}
	Ok(wv)
}

fn poll_webview_result<T: DeserializeOwned>(wv: &WebView, max_secs: i32) -> Result<Polled<T>> {
	for _ in 0..max_secs {
		if let Ok(raw) = wv.eval(POLL_RESULT_JS)
			&& let Ok(state) = serde_json::from_str::<PollState>(&raw)
		{
			if state.waf {
				return Ok(Polled::Waf);
			}
			if let Some(err) = state.error {
				bail!("Comix error: {err}");
			}
			if state.done
				&& let Some(val) = state.result
			{
				return serde_json::from_value::<T>(val)
					.map(Polled::Done)
					.map_err(|e| error!("Failed to parse Comix WebView result: {e}"));
			}
		}
		sleep(1);
	}
	bail!("Timed out waiting for Comix page data")
}

/// Runs a harvest script and polls for its result. If a security check shows up mid-run,
/// waits for it to clear, reloads the page, and runs the script once more.
fn run_script<T: DeserializeOwned>(
	wv: &WebView,
	url: &str,
	script: &str,
	start_error: &str,
	max_secs: i32,
) -> Result<T> {
	for attempt in 0..2 {
		wv.eval(script).map_err(|_| error!("{start_error}"))?;
		match poll_webview_result::<T>(wv, max_secs)? {
			Polled::Done(value) => return Ok(value),
			Polled::Waf if attempt == 0 => recover_from_waf(wv, url, false)?,
			Polled::Waf => break,
		}
	}
	bail!("{WAF_ERROR_MSG}")
}

/// Fetches SSR `script#initial-data` via HTTP first, falling back to WebView if challenged.
pub fn fetch_initial_data(url: &str) -> Result<InitialData> {
	if let Ok(req) = Request::get(url)
		.map(|r| r.header("Accept", "text/html,application/xhtml+xml"))
		&& let Ok(resp) = req.send()
		&& resp.status_code() < 400
		&& let Ok(html) = resp.get_string()
		&& !helpers::is_waf_or_cf_html(&html)
		&& let Some(json_str) = helpers::extract_initial_data_json(&html)
		&& let Ok(parsed) = serde_json::from_str::<InitialData>(json_str)
	{
		return Ok(parsed);
	}

	let wv = open_page(url, true)?;
	let raw = wv
		.eval(READ_INITIAL_DATA_JS)
		.map_err(|_| error!("Failed to read initial-data from WebView"))?;
	if raw.trim().is_empty() {
		bail!("Comix page did not include initial-data");
	}
	serde_json::from_str::<InitialData>(&raw)
		.map_err(|e| error!("Failed to parse Comix initial-data: {e}"))
}

/// Resolves comma-separated author/artist/tag names via `/api/v1/tags/search`.
pub fn fetch_tag_ids(base_url: &str, term_type: &str, raw_csv: &str) -> Vec<String> {
	let base = base_url.trim_end_matches('/');
	let mut ids = Vec::new();
	for token in raw_csv.split(',') {
		let query = token.trim();
		if query.is_empty() {
			continue;
		}
		if query.chars().all(|c| c.is_ascii_digit()) {
			if !ids.iter().any(|x| x == query) {
				ids.push(query.to_string());
			}
			continue;
		}
		let url = format!(
			"{base}/api/v1/tags/search?type={term_type}&q={}&limit=20",
			encode_uri_component(query)
		);
		if let Ok(req) = Request::get(&url).map(|r| r.header("Accept", "application/json"))
			&& let Ok(resp) = req.send()
			&& resp.status_code() < 400
			&& let Ok(parsed) = resp.get_json_owned::<TagsSearchResponse>()
			&& let Some(items) = parsed.result
		{
			let picked = items
				.iter()
				.find(|item| item.label.eq_ignore_ascii_case(query))
				.or_else(|| items.first());
			if let Some(item) = picked {
				let id_str = item.id.to_string();
				if !ids.contains(&id_str) {
					ids.push(id_str);
				}
			}
		}
	}
	ids
}

pub fn harvest_browse(url: &str) -> Result<BrowseHarvestResult> {
	let wv = open_page(url, false)?;
	run_script(&wv, url, HARVEST_BROWSE_JS, "Failed to start Comix browse script", 35)
}

pub fn harvest_title_page<F>(
	url: &str,
	needs_details: bool,
	needs_chapters: bool,
	on_details: F,
) -> Result<(Option<InitialData>, Option<ChaptersHarvestResult>)>
where
	F: FnOnce(&InitialData),
{
	if !needs_chapters {
		if needs_details {
			let data = fetch_initial_data(url)?;
			on_details(&data);
			return Ok((Some(data), None));
		}
		return Ok((None, None));
	}

	let wv = open_page(url, needs_details)?;

	let details = if needs_details {
		let raw = wv.eval(READ_INITIAL_DATA_JS).unwrap_or_default();
		let parsed = serde_json::from_str::<InitialData>(&raw).ok();
		if let Some(ref data) = parsed {
			on_details(data);
		}
		parsed
	} else {
		None
	};

	let chapters: ChaptersHarvestResult = run_script(
		&wv,
		url,
		HARVEST_CHAPTERS_JS,
		"Failed to start Comix chapter script",
		90,
	)?;
	Ok((details, Some(chapters)))
}

pub fn harvest_chapter_pages(url: &str) -> Result<PagesHarvestResult> {
	let wv = open_page(url, false)?;
	run_script(&wv, url, CHAPTER_PAGES_JS, "Failed to start Comix reader script", 90)
}
