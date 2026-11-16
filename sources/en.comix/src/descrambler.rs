// Ported from keiyoushi/extensions-source src/en/comix/Descrambler.kt + Comix.kt fallback interceptor
use aidoku::{
	HashMap, ImageResponse, PageContext, Result,
	alloc::{String, Vec, vec},
	canvas::Rect,
	imports::{
		canvas::{Canvas, ImageRef},
		net::Request,
	},
	prelude::*,
};

const GRID_COLS: i32 = 5;
const GRID_ROWS: i32 = 5;
const NUM_TILES: usize = (GRID_COLS * GRID_ROWS) as usize;

const ENC_MULTIPLIER: u32 = 1_000_005;
const ENC_INCREMENT: u32 = 1_234_567_891;
const LCG_MULTIPLIER: u32 = 1_664_525;
const LCG_INCREMENT: u32 = 1_013_904_223;

const FALLBACK_PATHS: &[&str] = &["/hi/", "/fcf/", "/i5/", "/si/", "/i/", "/sii/", "/ii/"];
const PATH_PREFIXES: &[&str] = &["/i5/", "/sii/", "/si/", "/ii/", "/i/"];

fn header_or_context<'a>(
	headers: &'a HashMap<String, String>,
	context: &'a Option<PageContext>,
	key: &str,
) -> Option<&'a str> {
	if let Some(val) = headers.get(key) {
		return Some(val.as_str());
	}
	if let Some((_, val)) = headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)) {
		return Some(val.as_str());
	}
	if let Some(ctx) = context {
		if let Some(val) = ctx.get(key) {
			return Some(val.as_str());
		}
		if let Some((_, val)) = ctx.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)) {
			return Some(val.as_str());
		}
	}
	None
}

fn decode_scramble_hash(hash: Option<&str>) -> u32 {
	match hash.map(str::trim) {
		Some("03632") => 58414,
		Some("02900") => 117532,
		_ => 0,
	}
}

fn has_image_signature(bytes: &[u8]) -> bool {
	if bytes.len() < 12 {
		return false;
	}
	let is_webp = &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP";
	let is_jpeg = bytes[0] == 0xFF && bytes[1] == 0xD8;
	let is_png = &bytes[0..4] == b"\x89PNG";
	is_webp || is_jpeg || is_png
}

fn decode_with_xorshift(
	bytes: &[u8],
	initial_state: u32,
	length: usize,
	high_byte: bool,
) -> Vec<u8> {
	let mut result = bytes.to_vec();
	let mut state = initial_state;
	let limit = result.len().min(length);
	for byte in &mut result[..limit] {
		state ^= state << 13;
		state ^= state >> 17;
		state ^= state << 5;
		let key = if high_byte {
			(state >> 24) as u8
		} else {
			(state & 0xFF) as u8
		};
		*byte ^= key;
	}
	result
}

fn decode_with_lcg(bytes: &[u8], seed: u32, length: usize) -> Vec<u8> {
	let mut result = bytes.to_vec();
	let mut state = seed;
	let limit = result.len().min(length);
	for byte in &mut result[..limit] {
		state = state.wrapping_mul(ENC_MULTIPLIER).wrapping_add(ENC_INCREMENT);
		*byte ^= (state >> 24) as u8;
	}
	result
}

pub fn decode_encoded_bytes(bytes: &[u8], seed: u32, length: usize, algo: Option<&str>) -> Vec<u8> {
	if algo != Some("2") {
		return decode_with_lcg(bytes, seed, length);
	}

	let candidates = [
		decode_with_xorshift(bytes, seed | 1, length, false),
		decode_with_xorshift(bytes, seed, length, false),
		decode_with_xorshift(bytes, seed | 1, length, true),
		decode_with_lcg(bytes, seed, length),
	];

	candidates
		.iter()
		.find(|c| has_image_signature(c))
		.cloned()
		.unwrap_or_else(|| candidates[0].clone())
}

fn build_order(seed: u32, n: usize) -> Vec<usize> {
	let mut arr: Vec<usize> = (0..n).collect();
	let mut state = seed | 1;
	for i in (1..n).rev() {
		state ^= state << 13;
		state ^= state >> 17;
		state ^= state << 5;
		let j = (state as usize) % (i + 1);
		arr.swap(i, j);
	}
	let mut inverse = vec![0usize; n];
	for (i, &val) in arr.iter().enumerate() {
		inverse[val] = i;
	}
	inverse
}

fn build_order_lcg(seed: u32, n: usize) -> Vec<usize> {
	let mut arr: Vec<usize> = (0..n).collect();
	let mut state = seed;
	for i in (1..n).rev() {
		state = state.wrapping_mul(LCG_MULTIPLIER).wrapping_add(LCG_INCREMENT);
		let j = (state as usize) % (i + 1);
		arr.swap(i, j);
	}
	let mut inverse = vec![0usize; n];
	for (i, &val) in arr.iter().enumerate() {
		inverse[val] = i;
	}
	inverse
}

fn descramble_image(image: &ImageRef, seed: u32, algo: Option<&str>) -> ImageRef {
	let width = image.width();
	let height = image.height();
	let tile_w = ((width as i32) / GRID_COLS) as f32;
	let tile_h = ((height as i32) / GRID_ROWS) as f32;

	let order = if algo == Some("3") {
		build_order(seed, NUM_TILES)
	} else {
		build_order_lcg(seed, NUM_TILES)
	};

	let mut canvas = Canvas::new(width, height);
	canvas.draw_image(image, Rect::new(0.0, 0.0, width, height));

	for (dst_idx, &src_idx) in order.iter().enumerate() {
		let src_col = (src_idx as i32 % GRID_COLS) as f32;
		let src_row = (src_idx as i32 / GRID_COLS) as f32;
		let dst_col = (dst_idx as i32 % GRID_COLS) as f32;
		let dst_row = (dst_idx as i32 / GRID_COLS) as f32;

		let src_rect = Rect::new(src_col * tile_w, src_row * tile_h, tile_w, tile_h);
		let dst_rect = Rect::new(dst_col * tile_w, dst_row * tile_h, tile_w, tile_h);
		canvas.copy_image(image, src_rect, dst_rect);
	}

	canvas.get_image()
}

pub fn fallback_urls(url: &str) -> Vec<String> {
	let mut out = Vec::new();
	for prefix in PATH_PREFIXES {
		if let Some(pos) = url.find(prefix) {
			for replacement in FALLBACK_PATHS {
				if replacement == prefix {
					continue;
				}
				let candidate = format!(
					"{}{}{}",
					&url[..pos],
					replacement,
					&url[pos + prefix.len()..]
				);
				if candidate != url && !out.contains(&candidate) {
					out.push(candidate);
				}
			}
			break;
		}
	}
	out
}

pub fn process_page_image(
	response: ImageResponse,
	context: Option<PageContext>,
) -> Result<ImageRef> {
	let mut headers = response.headers;
	let mut current_image = response.image;

	// Retry fallback CDN paths on HTTP failure, matching Mihon Comix.kt
	if response.code >= 400
		&& let Some(orig_url) = response
			.request
			.url
			.as_deref()
			.or_else(|| context.as_ref().and_then(|c| c.get("url").map(String::as_str)))
	{
		for fallback in fallback_urls(orig_url) {
			if let Ok(resp) = Request::get(&fallback)?.send()
				&& resp.status_code() < 400
				&& let Ok(bytes) = resp.get_data()
			{
				let mut recovered_headers = HashMap::new();
				for h in [
					"x-scramble-seed",
					"x-scramble-grid",
					"x-scramble-algo",
					"x-scramble-hash",
					"x-enc-seed",
					"x-enc-len",
					"x-enc-algo",
				] {
					if let Some(val) = resp.get_header(h) {
						recovered_headers.insert(String::from(h), val);
					}
				}
				headers = recovered_headers;
				current_image = ImageRef::new(&bytes);
				break;
			}
		}
	}

	let enc_seed = header_or_context(&headers, &context, "x-enc-seed")
		.and_then(|s| s.trim().parse::<i64>().ok())
		.map(|v| v as u32);
	let enc_len = header_or_context(&headers, &context, "x-enc-len")
		.and_then(|s| s.trim().parse::<usize>().ok());
	let enc_algo = header_or_context(&headers, &context, "x-enc-algo");

	let scramble_seed = header_or_context(&headers, &context, "x-scramble-seed")
		.and_then(|s| s.trim().parse::<i64>().ok())
		.map(|v| v as u32);
	let scramble_grid = header_or_context(&headers, &context, "x-scramble-grid");
	let scramble_algo = header_or_context(&headers, &context, "x-scramble-algo");
	let scramble_hash = decode_scramble_hash(header_or_context(&headers, &context, "x-scramble-hash"));

	let needs_xor = matches!((enc_seed, enc_len), (Some(s), Some(_)) if s != 0);
	let should_descramble_grid = scramble_grid == Some("5x5")
		&& matches!(scramble_algo, None | Some("1") | Some("2") | Some("3"))
		&& scramble_seed.is_some_and(|s| s != 0);

	if !needs_xor && !should_descramble_grid {
		return Ok(current_image);
	}

	let decoded_image = if let (Some(seed), Some(len)) = (enc_seed, enc_len)
		&& seed != 0
	{
		let raw = current_image.data();
		let decoded = decode_encoded_bytes(&raw, seed, len, enc_algo);
		ImageRef::new(&decoded)
	} else {
		current_image
	};

	if should_descramble_grid && let Some(seed) = scramble_seed {
		return Ok(descramble_image(
			&decoded_image,
			seed ^ scramble_hash,
			scramble_algo,
		));
	}

	Ok(decoded_image)
}
