use aidoku::{
	alloc::String,
	imports::{defaults::defaults_get, html::Element},
};

const CLEAN_TITLE_KEY: &str = "cleanTitle";

pub fn remove_special_chars(string: String) -> String {
	let chars_to_replace = ['[', '!', '#', ':', '(', ')', ',', '-', ']'];
	let mut result = String::new();

	for c in string.chars() {
		if chars_to_replace.contains(&c) {
			result.push(' ');
		} else {
			result.push(c);
		}
	}

	result
}

pub fn clean_title_if_needed(title: String) -> String {
	let trimmed = title.trim();
	if !defaults_get::<bool>(CLEAN_TITLE_KEY).unwrap_or(false) {
		return String::from(trimmed);
	}

	let cleaned = strip_trailing_brackets(trimmed);
	String::from(cleaned.trim())
}

fn strip_trailing_brackets(s: &str) -> &str {
	let s = s.trim_end();
	if let Some(without_paren) = s.strip_suffix(')')
		&& let Some(open) = without_paren.rfind('(')
		&& !without_paren[open + 1..].contains('(')
	{
		return &without_paren[..open];
	}
	if let Some(without_bracket) = s.strip_suffix(']')
		&& let Some(open) = without_bracket.rfind('[')
		&& !without_bracket[open + 1..].contains('[')
	{
		return &without_bracket[..open];
	}
	s
}

pub fn source_img(el: &Element) -> Option<String> {
	if let Some(srcset) = el.select_first("source").and_then(|s| s.attr("srcset"))
		&& !srcset.is_empty()
	{
		return Some(srcset.replace("small", "normal"));
	}
	el.select_first("img")
		.and_then(|img| img.attr("abs:src").or_else(|| img.attr("src")))
}

pub fn has_season_prefix(title: &str) -> bool {
	let lower = title.to_ascii_lowercase();
	let bytes = lower.as_bytes();
	for (idx, _) in lower.match_indices("season") {
		if idx == 0 || !bytes[idx - 1].is_ascii_alphanumeric() {
			let rest = lower[idx + 6..].trim_start();
			if rest.as_bytes().first().is_some_and(|b| b.is_ascii_digit()) {
				return true;
			}
		}
	}
	for (idx, _) in lower.match_indices('s') {
		if idx == 0 || !bytes[idx - 1].is_ascii_alphanumeric() {
			let rest = lower[idx + 1..].trim_start();
			if rest.as_bytes().first().is_some_and(|b| b.is_ascii_digit()) {
				return true;
			}
		}
	}
	false
}
