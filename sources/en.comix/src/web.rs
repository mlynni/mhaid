use crate::{
	BASE_URL,
	helpers::create_request_get,
	models::ErrorResponse,
	settings::{get_verify_cookie, set_verify_cookie, transport_mode},
};
use aidoku::{
	HashMap, Manga, Result,
	alloc::{string::String, string::ToString, vec::Vec},
	helpers::uri::QueryParameters,
	imports::{
		js::WebView,
		net::{Request, Response},
		std::sleep,
	},
	prelude::*,
};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::Value;

const INSTALLER_REQUEST_TOKEN: &str = "__AIDOKU_INSTALLER_REQUEST_TOKEN__";
const INSTALLER_RESPONSE_TOKEN: &str = "__AIDOKU_INSTALLER_RESPONSE_TOKEN__";

const CF_CHALLENGE_HTML_ERROR_MESSAGE: &str = "Response returned CF challenge page. If problem persist, please clear the source cache and restart the application to resolve this issue.";
const CF_CHALLENGE_ERROR_MESSAGE: &str = "Response returned CF challenge page instead of JSON data. If problem persist, please clear the source cache and restart the application to resolve this issue.";

const WAF_CHALLENGE_KEY: &str = "captcha_required";
const THIRD_PARTY_APP_KEY: &str = "third_party_app";
const WAF_CHALLENGE_HTML_ERROR_MESSAGE: &str = "Response returned WAF challenge page. Please open Comix Settings and Verify Captcha to resolve this issue.";
const WAF_CHALLENGE_ERROR_MESSAGE: &str = "Response returned WAF challenge page instead of JSON data. Please open Comix Settings and Verify Captcha to resolve this issue.";

#[derive(Deserialize)]
struct AxiosRequest {
	url: String,
	params: Option<HashMap<String, Value>>,
}

#[derive(Deserialize)]
struct WebViewFetchResult {
	done: bool,
	status: Option<i32>,
	enc: Option<String>,
	body: Option<String>,
	error: Option<String>,
}

pub struct ComixWebView {
	web_view: WebView,
	is_initialized: bool,
	cover_host_challenged: Option<bool>,
	use_webview_transport: bool,
}

impl ComixWebView {
	pub fn new() -> Self {
		Self {
			web_view: WebView::new(),
			is_initialized: false,
			cover_host_challenged: None,
			use_webview_transport: false,
		}
	}

	pub fn reset(&mut self) {
		self.is_initialized = false;
		self.cover_host_challenged = None;
		self.use_webview_transport = false;
	}

	fn extract_waf_pass(raw: &str) -> Option<String> {
		for segment in raw.split([',', ';']) {
			let trimmed = segment.trim();
			if let Some(value) = trimmed.strip_prefix("waf_pass=") {
				let token = value.split(';').next().unwrap_or(value).trim();
				if !token.is_empty() {
					return Some(token.into());
				}
			}
		}
		None
	}

	fn capture_waf_pass_from_response(response: &Response) {
		let header = response
			.get_header("Set-Cookie")
			.or_else(|| response.get_header("set-cookie"));
		if let Some(raw) = header
			&& let Some(token) = Self::extract_waf_pass(&raw)
		{
			set_verify_cookie(&token);
		}
	}

	fn sync_waf_pass_from_webview_cookies(&mut self) -> Option<String> {
		let token = self
			.web_view
			.eval("(() => { const m = document.cookie.match(/(?:^|;\\s*)waf_pass=([^;]+)/); return m ? decodeURIComponent(m[1]) : ''; })()")
			.ok()?;
		let trimmed = token.trim();
		if !trimmed.is_empty() {
			set_verify_cookie(trimmed);
			Some(trimmed.into())
		} else {
			None
		}
	}

	fn load_webview(&mut self) -> Result<()> {
		let request = create_request_get(BASE_URL)?;
		let response = request.send()?;

		let status_code = response.status_code();

		if status_code == 403
			&& response
				.get_header("cf-mitigated")
				.is_some_and(|value| value == "challenge")
		{
			bail!("{CF_CHALLENGE_HTML_ERROR_MESSAGE}")
		} else if status_code >= 400 {
			bail!("Response Error: {}", response.status_code())
		} else if response
			.get_html()?
			.select_first("head > title")
			.is_some_and(|e| e.text().is_some_and(|t| t == "Security check"))
		{
			bail!("{}", WAF_CHALLENGE_HTML_ERROR_MESSAGE)
		}

		self.web_view
			.load_html_blocking(response.get_string()?.as_str(), Some(BASE_URL))?;
		self.find_functions(&response)?;
		self.is_initialized = true;
		Ok(())
	}

	fn find_functions(&mut self, response: &Response) -> Result<()> {
		let main_module_src = response
			.get_html()?
			.select_first("head > script[type=\"module\"][src*=\"main\"]")
			.and_then(|e| e.attr("src"))
			.ok_or_else(|| error!("Main module not found"))?;
		let main_module_url = Self::module_url(BASE_URL, &main_module_src)?;
		let main_module = create_request_get(&main_module_url)?.string()?;
		let api_module = main_module
			.split(['\'', '"'])
			.find(|path| {
				path.strip_prefix("./")
					.unwrap_or(path)
					.starts_with("apicore-")
					&& path.ends_with(".js")
			})
			.ok_or_else(|| error!("API module not found"))?;
		let api_module_url = Self::module_url(&main_module_url, api_module)?;
		let source = create_request_get(&api_module_url)?.string()?;
		let module_body = Self::load_module_body(&api_module_url, &source, 0)?;

		let names: Vec<&str> = {
			let words: Vec<&str> = source
				.split(|c: char| !c.is_ascii() || !Self::is_identifier_byte(c as u8))
				.filter(|s| !s.is_empty())
				.collect();
			let mut names: Vec<&str> = words
				.windows(2)
				.filter(|pair| {
					matches!(pair[0], "var" | "let" | "const") && Self::is_identifier(pair[1])
				})
				.map(|pair| pair[1])
				.collect();
			names.sort_unstable();
			names.dedup();
			names
		};
		let names = serde_json::to_string(&names)?;
		let result = self.web_view.eval(&format!(
			"(() => {{
				try {{
					{module_body}
					for (const name of {names}) {{
						try {{
							const client = eval(name);
							const requests = client?.interceptors?.request?.handlers;
							const responses = client?.interceptors?.response?.handlers;
							if (!requests?.length || !responses?.length) continue;
							const request = requests[requests.length - 1]?.fulfilled;
							const response = responses[responses.length - 1]?.fulfilled;
							if (typeof request !== 'function' || typeof response !== 'function') continue;
							window['{INSTALLER_REQUEST_TOKEN}'] = request;
							window['{INSTALLER_RESPONSE_TOKEN}'] = response;
							return 'ok';
						}} catch (e) {{}}
					}}
					return 'API interceptors not found';
				}} catch (e) {{
					return 'JS error: ' + e;
				}}
			}})()"
		))?;
		if result != "ok" {
			bail!("Failed to load API client: {result}");
		}
		Ok(())
	}

	fn module_url(base: &str, path: &str) -> Result<String> {
		if path.starts_with("https://") {
			return Ok(path.into());
		}
		if path.starts_with('/') {
			return Ok(format!("{BASE_URL}{path}"));
		}
		let path = path.strip_prefix("./").unwrap_or(path);
		let directory = base
			.rsplit_once('/')
			.filter(|(dir, _)| dir.len() > "https://".len());
		Ok(format!(
			"{}/{}",
			directory.map_or(base, |(dir, _)| dir),
			path
		))
	}

	fn load_module_body(url: &str, source: &str, depth: usize) -> Result<String> {
		if depth > 4 {
			bail!("API module dependency limit exceeded");
		}
		let (body, _) = Self::split_module(source)?;

		let mut result = String::new();
		let mut offset = 0;

		let keyword = "import";
		let keyword_positions = body.match_indices(keyword).filter_map(move |(start, _)| {
			let end = start + keyword.len();
			let before = start.checked_sub(1).map(|i| body.as_bytes()[i]);
			let after = body.as_bytes().get(end).copied();
			(!before.is_some_and(Self::is_identifier_byte)
				&& !after.is_some_and(Self::is_identifier_byte))
			.then_some(start)
		});

		for start in keyword_positions {
			let Some(rest) = body[start + keyword.len()..].trim_start().strip_prefix('{') else {
				continue;
			};
			let (bindings, rest) = rest
				.split_once('}')
				.ok_or_else(|| error!("Invalid API module import"))?;
			let rest = rest
				.trim_start()
				.strip_prefix("from")
				.ok_or_else(|| error!("Invalid API module import"))?
				.trim_start();
			let quote = rest
				.chars()
				.next()
				.filter(|c| matches!(c, '\'' | '"'))
				.ok_or_else(|| error!("Invalid API module import path"))?;
			let (path, rest) = rest[1..]
				.split_once(quote)
				.ok_or_else(|| error!("Invalid API module import path"))?;
			result.push_str(&body[offset..start]);
			offset = body.len() - rest.len();
			let dependency_url = Self::module_url(url, path)?;
			let dependency = create_request_get(&dependency_url)?.string()?;
			let (_, exports) = Self::split_module(&dependency)?;
			let dependency_body = Self::load_module_body(&dependency_url, &dependency, depth + 1)?;
			let bindings = Self::module_bindings(bindings, true)?;
			let exports = Self::module_bindings(exports, false)?;
			result.push_str(&format!(
				"const {{{bindings}}} = (() => {{{dependency_body}; return {{{exports}}};}})();"
			));
		}
		result.push_str(&body[offset..]);
		Ok(result)
	}

	fn split_module(source: &str) -> Result<(&str, &str)> {
		let (body, exports) = source
			.rsplit_once("export")
			.ok_or_else(|| error!("API module exports not found"))?;
		let exports = exports.trim().trim_end_matches(';').trim_end();
		let exports = exports
			.strip_prefix('{')
			.and_then(|s| s.strip_suffix('}'))
			.filter(|s| !s.contains(['{', '}']))
			.ok_or_else(|| error!("API module exports not found"))?;
		Ok((body, exports))
	}

	fn module_bindings(bindings: &str, import: bool) -> Result<String> {
		let mut result = Vec::new();
		for binding in bindings.split(',').map(str::trim).filter(|s| !s.is_empty()) {
			let parts: Vec<&str> = binding.split_whitespace().collect();
			let (local, exported) = match parts.as_slice() {
				[name] => (*name, *name),
				[name, "as", alias] => (*name, *alias),
				_ => bail!("Unsupported API module binding"),
			};
			if !Self::is_identifier(local) || !Self::is_identifier(exported) {
				bail!("Invalid API module binding");
			}
			result.push(if import {
				format!("{local}:{exported}")
			} else {
				format!("{exported}:{local}")
			});
		}
		Ok(result.join(","))
	}

	fn is_identifier_byte(byte: u8) -> bool {
		byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$')
	}

	fn is_identifier(name: &str) -> bool {
		!name.is_empty()
			&& !name.as_bytes()[0].is_ascii_digit()
			&& name.bytes().all(Self::is_identifier_byte)
	}
}

impl ComixWebView {
	pub fn is_webview_transport(&self) -> bool {
		self.use_webview_transport
	}

	pub fn sign_url(&mut self, url: &str) -> Result<String> {
		if !self.is_initialized {
			self.load_webview()?
		}

		let url_json = serde_json::to_string(url)?;
		let result = self.web_view.eval(&format!(
			r#"(() => {{
			const url = new URL({url_json});
			const result = {{}};

			for (const [key, rawValue] of url.searchParams) {{
				const value = rawValue.length > 0 && [...rawValue].every(c => c >= '0' && c <= '9')
					? Number(rawValue)
					: rawValue;

				const parts = key.split(']').join('').split('[');

				let current = result;

				for (let i = 0; i < parts.length; i++) {{
					const part = parts[i];
					const last = i === parts.length - 1;

					if (last) {{
						if (part === '') {{
							current.push(value);
						}} else if (current[part] === undefined) {{
							current[part] = value;
						}} else if (Array.isArray(current[part])) {{
							current[part].push(value);
						}} else {{
							current[part] = [current[part], value];
						}}
					}} else {{
						const nextPart = parts[i + 1];

						current[part] ??= nextPart === '' ? [] : {{}};
						current = current[part];
					}}
				}}
			}}

			// Since 2026-10-16 Comix checks the signed `v` flag word on chapter endpoints.
			const apiPath = url.pathname.replace(/^\/api\/v1/, '');
			if (
				result.v === undefined
				&& (/\/(?:manga\/[^/?]+|groups\/\d+)\/chapters$/.test(apiPath) || /^\/chapters\/\d+$/.test(apiPath))
			) {{
				result.v = '0';
			}}

			const request = window['{INSTALLER_REQUEST_TOKEN}']({{
				url: `${{url.origin}}${{url.pathname}}`,
				method: 'GET',
				params: result,
			}});

			return JSON.stringify(request);
		}})()"#
		))?;

		let axios_request: AxiosRequest = serde_json::from_str(result.as_str())?;

		fn build_query(params_map: &HashMap<String, Value>) -> QueryParameters {
			let mut params = QueryParameters::new();

			for (key, value) in params_map {
				push_value(&mut params, key, value);
			}

			params
		}

		fn push_value(params: &mut QueryParameters, key: &str, value: &Value) {
			match value {
				Value::Null => {
					params.push_key(key);
				}

				Value::Bool(_) | Value::Number(_) | Value::String(_) => {
					let value_str = value.to_string();

					// Remove JSON string quotes
					let value_str = match value {
						Value::String(s) => s.as_str(),
						_ => value_str.as_str(),
					};

					params.push(key, Some(value_str));
				}

				Value::Array(arr) => {
					let array_key = format!("{key}[]");

					for item in arr {
						match item {
							Value::String(s) => {
								params.push(&array_key, Some(s));
							}
							_ => {
								let value_str = item.to_string();
								params.push(&array_key, Some(&value_str));
							}
						}
					}
				}

				Value::Object(obj) => {
					for (child_key, child_value) in obj {
						let nested_key = format!("{key}[{child_key}]");
						push_value(params, &nested_key, child_value);
					}
				}
			}
		}

		if let Some(params) = axios_request.params {
			let query = build_query(&params);
			Ok(format!("{}?{query}", axios_request.url))
		} else {
			Ok(axios_request.url)
		}
	}

	pub fn build_request(&mut self, url: &str) -> Result<Request> {
		let signed = self.sign_url(url)?;
		create_request_get(&signed)
	}

	fn is_challenged_status(status: i32, body: &str) -> bool {
		(status == 403 || status == 429)
			&& (body.contains(WAF_CHALLENGE_KEY)
				|| body.contains(THIRD_PARTY_APP_KEY)
				|| body.contains("Just a moment"))
	}

	pub fn try_background_renew(&mut self) -> bool {
		if !self.is_initialized && self.load_webview().is_err() {
			return false;
		}
		let before = get_verify_cookie();
		let start_js = r#"(() => {
			window.__cxRenew = 'waiting';
			document.querySelectorAll('iframe[data-cx-renew]').forEach(f => f.remove());
			const frame = document.createElement('iframe');
			frame.setAttribute('data-cx-renew', '1');
			let timer;
			const finish = state => {
				if (window.__cxRenew !== 'waiting') return;
				window.__cxRenew = state;
				clearTimeout(timer);
				window.removeEventListener('message', receive);
				frame.remove();
			};
			const receive = event => {
				if (event.origin === location.origin && event.source === frame.contentWindow
					&& event.data && event.data.type === 'waf-captcha-solved') finish('solved');
			};
			window.addEventListener('message', receive);
			frame.src = '/@waf/challenge?return=%2F';
			frame.style.cssText = 'position:fixed;left:-9999px;width:360px;height:480px;border:0;opacity:0';
			frame.addEventListener('error', () => finish('failed'));
			timer = setTimeout(() => finish('timeout'), 6000);
			document.body.appendChild(frame);
			return 'started';
		})()"#;
		if self.web_view.eval(start_js).is_err() {
			return false;
		}
		for attempt in 0..6 {
			if attempt > 0 {
				sleep(1);
			}
			let state = self
				.web_view
				.eval("String(window.__cxRenew || '')")
				.unwrap_or_default();
			let now = self.sync_waf_pass_from_webview_cookies();
			if state == "solved" || (now.is_some() && now != before) {
				return true;
			}
			if state == "failed" || state == "timeout" {
				break;
			}
		}
		let _ = self
			.web_view
			.eval("document.querySelectorAll('iframe[data-cx-renew]').forEach(f => f.remove());");
		false
	}

	pub fn fetch_via_webview(&mut self, signed_url: &str) -> Result<(i32, Option<String>, String)> {
		if !self.is_initialized {
			self.load_webview()?
		}
		let url_json = serde_json::to_string(signed_url)?;
		let kick_js = format!(
			r#"((u) => {{
				window.__cxApiFetch = JSON.stringify({{ done: false }});
				const c = new AbortController();
				setTimeout(() => {{ try {{ c.abort(); }} catch (_) {{}} }}, 14000);
				fetch(u, {{ credentials: 'include', cache: 'no-store', headers: {{ 'Accept': 'application/json' }}, signal: c.signal }})
					.then(async r => {{
						const body = await r.text();
						const enc = r.headers.get('x-enc');
						window.__cxApiFetch = JSON.stringify({{ done: true, status: r.status, enc: enc || null, body }});
					}})
					.catch(e => {{
						window.__cxApiFetch = JSON.stringify({{ done: true, error: String((e && e.message) || e || 'fetch failed') }});
					}});
				return 'kicked';
			}})({url_json})"#
		);
		self.web_view.eval(&kick_js)?;

		for attempt in 0..15 {
			if attempt > 0 {
				sleep(1);
			}
			let raw = self.web_view.eval("String(window.__cxApiFetch || '')")?;
			if let Ok(res) = serde_json::from_str::<WebViewFetchResult>(&raw)
				&& res.done
			{
				let _ = self.sync_waf_pass_from_webview_cookies();
				if let Some(err) = res.error {
					bail!("WebView fetch failed: {err}");
				}
				return Ok((res.status.unwrap_or(500), res.enc, res.body.unwrap_or_default()));
			}
		}
		bail!("WebView API fetch timed out")
	}

	pub fn send_and_decode<T: DeserializeOwned>(&mut self, url: &str) -> Result<T> {
		let mode = transport_mode();
		if mode == "webview" || (mode != "native" && self.use_webview_transport) {
			let signed_url = self.sign_url(url)?;
			let (status, enc, body) = self.fetch_via_webview(&signed_url)?;
			if !Self::is_challenged_status(status, &body) {
				return self.decode_payload(status, None, enc.as_deref(), &body);
			}
			if self.try_background_renew() {
				let retry_url = self.sign_url(url)?;
				let (status2, enc2, body2) = self.fetch_via_webview(&retry_url)?;
				return self.decode_payload(status2, None, enc2.as_deref(), &body2);
			}
			return self.decode_payload(status, None, enc.as_deref(), &body);
		}

		let mut response = self.build_request(url)?.send()?;
		Self::capture_waf_pass_from_response(&response);
		let mut status = response.status_code();
		let mut cf = response.get_header("cf-mitigated");
		let mut enc = response.get_header("x-enc");
		let mut body = response.get_string().unwrap_or_default();

		let challenged = (status == 403 || status == 429)
			&& (cf.as_deref() == Some("challenge") || Self::is_challenged_status(status, &body));

		if challenged {
			if self.try_background_renew() {
				response = self.build_request(url)?.send()?;
				Self::capture_waf_pass_from_response(&response);
				status = response.status_code();
				cf = response.get_header("cf-mitigated");
				enc = response.get_header("x-enc");
				body = response.get_string().unwrap_or_default();
				if status < 400 {
					return self.decode_payload(status, cf.as_deref(), enc.as_deref(), &body);
				}
			}
			if mode != "native" {
				let signed_url = self.sign_url(url)?;
				if let Ok((wv_status, wv_enc, wv_body)) = self.fetch_via_webview(&signed_url)
					&& wv_status < 400
				{
					self.use_webview_transport = true;
					return self.decode_payload(wv_status, None, wv_enc.as_deref(), &wv_body);
				}
			}
		}

		self.decode_payload(status, cf.as_deref(), enc.as_deref(), &body)
	}

	pub fn decode_json_owned<T: DeserializeOwned>(&mut self, response: &Response) -> Result<T> {
		Self::capture_waf_pass_from_response(response);
		let status = response.status_code();
		let cf = response.get_header("cf-mitigated");
		let enc = response.get_header("x-enc");
		let body = response.get_string().unwrap_or_default();
		self.decode_payload(status, cf.as_deref(), enc.as_deref(), &body)
	}

	fn decode_payload<T: DeserializeOwned>(
		&mut self,
		status_code: i32,
		cf_mitigated: Option<&str>,
		enc_header: Option<&str>,
		body: &str,
	) -> Result<T> {
		if !self.is_initialized {
			self.load_webview()?;
		}

		if status_code == 403 && cf_mitigated == Some("challenge") {
			bail!("{CF_CHALLENGE_ERROR_MESSAGE}")
		} else if status_code >= 400 {
			if serde_json::from_str::<ErrorResponse>(body)
				.is_ok_and(|e| e.error == WAF_CHALLENGE_KEY || e.error == THIRD_PARTY_APP_KEY)
			{
				bail!("{}", WAF_CHALLENGE_ERROR_MESSAGE)
			} else {
				bail!("Response Error: {status_code}")
			}
		} else if let Some(enc) = enc_header {
			let encoded_response = serde_json::to_string(body)?;
			let enc = serde_json::to_string(enc)?;

			let result = self.web_view.eval(&format!(
				r#"(() => {{
					try {{
						let decoded = window['{INSTALLER_RESPONSE_TOKEN}']({{
							data: JSON.parse({encoded_response}),
							status: 200,
							headers: {{
								'x-enc': {enc},
							}},
						}});
						return JSON.stringify({{ result: decoded && decoded.data }});
					}} catch(e) {{
						return 'error: ' + e;
					}}
				}})()"#,
			))?;

			if result.starts_with("error:") {
				bail!("{result}");
			} else if result.is_empty() {
				bail!("Failed to fetch result")
			}

			serde_json::from_str(&result).map_err(|e| error!("Invalid json: {}", e))
		} else {
			serde_json::from_str(body).map_err(|e| error!("Invalid json: {}", e))
		}
	}

	pub fn is_cover_host_challenged(&mut self, sample_url: &str) -> bool {
		if let Some(cached) = self.cover_host_challenged {
			return cached;
		}
		let challenged = match create_request_get(sample_url) {
			Ok(req) => match req
				.header("Referer", &format!("{BASE_URL}/"))
				.header("Range", "bytes=0-0")
				.send()
			{
				Ok(resp) => resp.status_code() >= 400,
				Err(_) => true,
			},
			Err(_) => true,
		};
		self.cover_host_challenged = Some(challenged);
		challenged
	}

	pub fn resolve_covers_in_webview(&mut self, urls: &[String]) -> HashMap<String, String> {
		let mut resolved = HashMap::new();
		let mut unique_urls: Vec<String> = Vec::new();
		for url in urls {
			if url.starts_with("https://") && !unique_urls.contains(url) {
				unique_urls.push(url.clone());
			}
		}
		if unique_urls.is_empty() {
			return resolved;
		}
		if !self.is_initialized && self.load_webview().is_err() {
			return resolved;
		}
		let Ok(urls_json) = serde_json::to_string(&unique_urls) else {
			return resolved;
		};
		let kick_js = format!(
			"(function(urls){{\
				window.__cxCovers = window.__cxCovers || {{}};\
				var known = Object.keys(window.__cxCovers);\
				if (known.length > 240) {{\
					known.slice(0, known.length - 240).forEach(function(k){{ delete window.__cxCovers[k]; }});\
				}}\
				urls.forEach(function(u){{\
					var prev = window.__cxCovers[u];\
					if (prev && !prev.error) return;\
					window.__cxCovers[u] = {{ done: false }};\
					var c = new AbortController(), ctype = '';\
					setTimeout(function(){{ try {{ c.abort(); }} catch (e) {{}} }}, 15000);\
					fetch(u, {{ credentials: 'include', cache: 'no-store', signal: c.signal }})\
						.then(function(r){{ return r.ok ? r : fetch(u, {{ credentials: 'omit', cache: 'no-store', signal: c.signal }}); }})\
						.catch(function(){{ return fetch(u, {{ credentials: 'omit', cache: 'no-store', signal: c.signal }}); }})\
						.then(function(r){{ if (!r.ok) throw new Error('status ' + r.status); ctype = r.headers.get('content-type') || ''; return r.arrayBuffer(); }})\
						.then(function(ab){{\
							var b = new Uint8Array(ab), s = '', CH = 0x8000;\
							for (var i = 0; i < b.length; i += CH) {{ s += String.fromCharCode.apply(null, b.subarray(i, i + CH)); }}\
							window.__cxCovers[u] = {{ done: true, id: u, b64: btoa(s), ct: ctype }};\
						}})\
						.catch(function(e){{ window.__cxCovers[u] = {{ done: true, id: u, error: String(e) }}; }});\
				}});\
				return 'kicked';\
			}})({urls_json})"
		);
		if self.web_view.eval(&kick_js).is_err() {
			return resolved;
		}

		let read_js = format!(
			"JSON.stringify(({urls_json}).map(function(u){{\
				var e = (window.__cxCovers || {{}})[u];\
				if (!e || !e.done) return null;\
				if (e.error) return ['', String(e.error)];\
				if (e.id !== u) return null;\
				var r = [e.b64 || '', e.ct || 'image/jpeg'];\
				delete window.__cxCovers[u];\
				return r;\
			}}))"
		);

		for attempt in 0..8 {
			if attempt > 0 {
				sleep(1);
			}
			let Ok(raw) = self.web_view.eval(&read_js) else {
				break;
			};
			let Ok(items) = serde_json::from_str::<Vec<Option<(String, String)>>>(&raw) else {
				break;
			};
			let mut pending = false;
			for (url, item) in unique_urls.iter().zip(items.into_iter()) {
				if let Some((b64, ct)) = item {
					if !b64.is_empty() {
						let mime = ct.split(';').next().unwrap_or("image/jpeg").trim();
						let mime = if mime.starts_with("image/") {
							mime
						} else {
							"image/jpeg"
						};
						resolved.insert(url.clone(), format!("data:{mime};base64,{b64}"));
					}
				} else if !resolved.contains_key(url) {
					pending = true;
				}
			}
			if !pending {
				break;
			}
		}

		resolved
	}

	pub fn apply_covers_to_mangas(&mut self, mangas: &mut [Manga]) {
		let Some(first_url) = mangas
			.iter()
			.filter_map(|m| m.cover.as_deref())
			.find(|u| u.starts_with("https://"))
			.map(ToString::to_string)
		else {
			return;
		};

		if !self.is_cover_host_challenged(&first_url) {
			return;
		}

		let urls: Vec<String> = mangas
			.iter()
			.filter_map(|m| m.cover.as_ref())
			.filter(|u| u.starts_with("https://"))
			.cloned()
			.collect();
		let resolved = self.resolve_covers_in_webview(&urls);
		for manga in mangas.iter_mut() {
			if let Some(cover) = manga.cover.as_deref()
				&& let Some(data_uri) = resolved.get(cover)
			{
				manga.cover = Some(data_uri.clone());
			}
		}
	}

	pub fn fetch_cover_bytes(&mut self, url: &str) -> Option<Vec<u8>> {
		if !url.starts_with("http://") && !url.starts_with("https://") {
			return None;
		}
		if let Ok(req) = create_request_get(url)
			&& let Ok(resp) = req.header("Referer", &format!("{BASE_URL}/")).send()
			&& resp.status_code() < 400
			&& let Ok(bytes) = resp.get_data()
			&& !bytes.is_empty()
		{
			return Some(bytes);
		}

		self.cover_host_challenged = Some(true);
		let resolved = self.resolve_covers_in_webview(&[url.to_string()]);
		let data_uri = resolved.get(url)?;
		let (_, b64) = data_uri.split_once(";base64,")?;
		decode_base64(b64)
	}
}

pub fn decode_base64(input: &str) -> Option<Vec<u8>> {
	let mut out = Vec::with_capacity(input.len() * 3 / 4);
	let mut buf: u32 = 0;
	let mut bits: u8 = 0;
	for b in input.bytes() {
		if b.is_ascii_whitespace() {
			continue;
		}
		if b == b'=' {
			break;
		}
		let val = match b {
			b'A'..=b'Z' => b - b'A',
			b'a'..=b'z' => b - b'a' + 26,
			b'0'..=b'9' => b - b'0' + 52,
			b'+' | b'-' => 62,
			b'/' | b'_' => 63,
			_ => return None,
		};
		buf = (buf << 6) | (val as u32);
		bits += 6;
		if bits >= 8 {
			bits -= 8;
			out.push(((buf >> bits) & 0xFF) as u8);
		}
	}
	Some(out)
}
