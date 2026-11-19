use crate::models::HarvestedChapter;
use aidoku::{
	HashMap,
	alloc::{
		String, Vec, format,
		string::ToString,
	},
	imports::std::current_date,
};

pub fn resolve_url(base_url: &str, raw: &str) -> String {
	let trimmed = raw.trim();
	if trimmed.starts_with("http://") || trimmed.starts_with("https://") || trimmed.starts_with("data:") {
		return trimmed.to_string();
	}
	if trimmed.starts_with("//") {
		return format!("https:{trimmed}");
	}
	let base = base_url.trim_end_matches('/');
	if trimmed.starts_with('/') {
		format!("{base}{trimmed}")
	} else {
		format!("{base}/{trimmed}")
	}
}

/// Swap Comix cover size suffixes while avoiding non-existent sizes (`@100.jpg` often isn't generated).
pub fn apply_cover_quality(url: &str, quality: &str) -> String {
	if let Some(pos) = url.rfind("/images/covers/") {
		let (prefix, suffix) = url.split_at(pos);
		let cleaned = suffix
			.replace("@280.jpg", ".jpg")
			.replace("@280.jpeg", ".jpeg")
			.replace("@280.webp", ".webp")
			.replace("@100.jpg", ".jpg")
			.replace("@100.webp", ".webp");
		return match quality {
			"medium" | "small" => {
				if let Some(base) = cleaned.strip_suffix(".jpg") {
					format!("{prefix}{base}@280.jpg")
				} else if let Some(base) = cleaned.strip_suffix(".webp") {
					format!("{prefix}{base}@280.webp")
				} else {
					format!("{prefix}{cleaned}")
				}
			}
			_ => format!("{prefix}{cleaned}"),
		};
	}
	url.to_string()
}

pub fn extract_path(url_or_path: &str) -> String {
	let s = url_or_path.trim();
	if let Some(rest) = s
		.strip_prefix("https://")
		.or_else(|| s.strip_prefix("http://"))
	{
		if let Some(slash) = rest.find('/') {
			return rest[slash..].split(['?', '#']).next().unwrap_or("/").to_string();
		}
		return "/".into();
	}
	if s.starts_with('/') {
		s.split(['?', '#']).next().unwrap_or("/").to_string()
	} else {
		format!("/{}", s.split(['?', '#']).next().unwrap_or(s))
	}
}

pub fn extract_hid_from_url(url_or_path: &str) -> Option<String> {
	let path = extract_path(url_or_path);
	let rest = path.strip_prefix("/title/")?;
	let seg = rest.split('/').next()?;
	let hid = seg.split('-').next().unwrap_or(seg).trim();
	(!hid.is_empty()).then(|| hid.to_string())
}

pub fn extract_initial_data_json(html: &str) -> Option<&str> {
	let marker_pos = html
		.find("id=\"initial-data\"")
		.or_else(|| html.find("id='initial-data'"))?;
	let after_marker = &html[marker_pos..];
	let gt_offset = after_marker.find('>')?;
	let content_start = &after_marker[gt_offset + 1..];
	let end_offset = content_start.find("</script>")?;
	Some(content_start[..end_offset].trim())
}

pub fn is_waf_or_cf_html(html: &str) -> bool {
	html.contains("<title>Security check</title>")
		|| html.contains("<title>Just a moment...</title>")
		|| html.contains("/@waf/challenge")
		|| html.contains("cf-mitigated")
}

pub fn parse_relative_date(token: &str) -> Option<i64> {
	let now = current_date();
	let s = token.trim().to_ascii_lowercase();
	if s.is_empty() {
		return None;
	}
	if s == "just now" || s == "today" || s == "now" {
		return Some(now);
	}
	if s == "yesterday" {
		return Some(now - 86400);
	}

	let cleaned = s.strip_suffix("ago").unwrap_or(&s).trim();
	let mut num = String::new();
	let mut unit = String::new();

	for c in cleaned.chars() {
		if c.is_ascii_digit() && unit.is_empty() {
			num.push(c);
		} else if c.is_ascii_alphabetic() {
			unit.push(c);
		} else if !c.is_whitespace() {
			return None;
		}
	}

	let n: i64 = num.parse().ok()?;
	let seconds = match unit.as_str() {
		"s" | "sec" | "secs" | "second" | "seconds" => n,
		"m" | "min" | "mins" | "minute" | "minutes" => n * 60,
		"h" | "hr" | "hrs" | "hour" | "hours" => n * 3600,
		"d" | "day" | "days" => n * 86400,
		"w" | "wk" | "wks" | "week" | "weeks" => n * 7 * 86400,
		"mo" | "mos" | "month" | "months" => n * 30 * 86400,
		"y" | "yr" | "yrs" | "year" | "years" => n * 365 * 86400,
		_ => return None,
	};

	Some(now - seconds)
}

pub fn extract_relative_date(text: &str) -> Option<i64> {
	let trimmed = text.trim();
	if let Some(ts) = parse_relative_date(trimmed) {
		return Some(ts);
	}

	let lower = trimmed.to_ascii_lowercase();
	if lower.contains("just now") {
		return Some(current_date());
	}
	if lower.contains("yesterday") {
		return Some(current_date() - 86400);
	}

	// Scan backwards for `<num><unit> ago` or `<num> <unit> ago`
	let words: Vec<&str> = lower.split_whitespace().collect();
	for idx in (0..words.len()).rev() {
		if let Some(ts) = parse_relative_date(words[idx]) {
			return Some(ts);
		}
		if idx > 0 {
			let pair = format!("{} {}", words[idx - 1], words[idx]);
			if let Some(ts) = parse_relative_date(&pair) {
				return Some(ts);
			}
		}
	}

	None
}

pub fn format_chapter_num(num: f32) -> String {
	let raw = num.to_string();
	if let Some(stripped) = raw.strip_suffix(".0") {
		stripped.to_string()
	} else {
		raw
	}
}

pub fn format_number_commas(n: i64) -> String {
	let digits = n.to_string();
	if digits.len() <= 3 {
		return digits;
	}
	let mut out = String::new();
	for (i, ch) in digits.chars().rev().enumerate() {
		if i > 0 && i % 3 == 0 && ch != '-' {
			out.push(',');
		}
		out.push(ch);
	}
	out.chars().rev().collect()
}

pub fn format_score(avg: f64, count: Option<i64>) -> String {
	let scaled = (avg * 100.0 + 0.5) as i64;
	let whole = scaled / 100;
	let frac = (scaled % 100).abs();
	match count.filter(|&c| c > 0) {
		Some(c) => format!("★ {whole}.{frac:02} ({})", format_number_commas(c)),
		None => format!("★ {whole}.{frac:02}"),
	}
}

fn is_official_like(ch: &HarvestedChapter) -> bool {
	ch.group.as_ref().is_some_and(|g| {
		g.is_official || g.id == Some(10702) || g.name.eq_ignore_ascii_case("official")
	})
}

fn is_better_chapter(new_ch: &HarvestedChapter, cur: &HarvestedChapter) -> bool {
	let off_new = is_official_like(new_ch);
	let off_cur = is_official_like(cur);
	if off_new != off_cur {
		return off_new;
	}
	if new_ch.votes != cur.votes {
		return new_ch.votes > cur.votes;
	}
	new_ch.id > cur.id
}

pub fn dedup_insert(map: &mut HashMap<String, HarvestedChapter>, ch: HarvestedChapter) {
	let key = format_chapter_num(ch.number);
	let replace = match map.get(&key) {
		None => true,
		Some(cur) => is_better_chapter(&ch, cur),
	};
	if replace {
		map.insert(key, ch);
	}
}
