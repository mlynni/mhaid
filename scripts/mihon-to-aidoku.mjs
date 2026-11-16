#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT_DIR = path.resolve(__dirname, "..");
const SOURCES_DIR = path.join(ROOT_DIR, "sources");
const TEMPLATES_DIR = path.join(ROOT_DIR, "templates");
const PUBLIC_DIR = path.join(ROOT_DIR, "public");
const EXTENSIONS_FILE = path.join(ROOT_DIR, "extensions.json");
const CACHE_DIR = path.join(ROOT_DIR, ".cache");

const SCRATCH_UPSTREAM =
	"C:/Users/user/.gemini/antigravity/brain/7e2fff51-f1df-47e0-84d7-a6443bb6fd07/scratch/upstream";

const MIHON_REPO = "https://github.com/keiyoushi/extensions-source.git";
const AIDOKU_REPO = "https://github.com/Aidoku-Community/sources.git";

const THEME_MAP = {
	madara: "madara",
	mangathemesia: "mangathemesia",
	mangabox: "mangabox",
	mmrcms: "mmrcms",
	gigaviewer: "gigaviewer",
	keyoapp: "keyoapp",
	iken: "iken",
	guya: "guya",
	liliana: "liliana",
	pizzareader: "pizzareader",
	wpcomics: "wpcomics",
	madtheme: "madtheme",
	heancms: "iken",
	libgroup: "libgroup",
	zmanga: "mangathemesia",
	mangareader: "mangareader",
};

function printHelp() {
	console.log(`Mihon-to-Aidoku Converter & Scaffolding Tool

Usage:
  node scripts/mihon-to-aidoku.mjs <extension-name> [options]
  node scripts/mihon-to-aidoku.mjs --sync-docs
  node scripts/mihon-to-aidoku.mjs --list

Options:
  --lang <code|all>         Preferred language folder in keiyoushi/extensions-source (e.g. en, uk, all)
  --use-community-fallback  Import an existing crate from Aidoku-Community/sources if present instead
                            of converting directly from Mihon Kotlin source
  --dry-run                 Inspect and print extracted Mihon metadata without writing files
  --sync-docs               Generate/update MIHON_SOURCE.md in every sources/<id>/ from Mihon upstream
  --list                    Show all extensions registered in extensions.json
  --help                    Show this help message

Examples:
  node scripts/mihon-to-aidoku.mjs bato --lang all
  node scripts/mihon-to-aidoku.mjs asurascans --lang en
  node scripts/mihon-to-aidoku.mjs weebcentral --use-community-fallback
`);
}

function runGit(args, cwd) {
	return execFileSync("git", args, {
		cwd,
		encoding: "utf8",
		stdio: ["ignore", "pipe", "pipe"],
	}).trim();
}

function readJson(filePath) {
	return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function writeJson(filePath, data, pretty = true) {
	const text = pretty ? JSON.stringify(data, null, "\t") + "\n" : JSON.stringify(data);
	fs.writeFileSync(filePath, text, "utf8");
}

function copyDirRecursive(src, dst) {
	fs.mkdirSync(dst, { recursive: true });
	for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
		const s = path.join(src, entry.name);
		const d = path.join(dst, entry.name);
		if (entry.isDirectory()) {
			if (entry.name === "target" || entry.name === ".git") continue;
			copyDirRecursive(s, d);
		} else {
			fs.copyFileSync(s, d);
		}
	}
}

function getLocalKeiyoushiRoots() {
	return [
		path.join(CACHE_DIR, "keiyoushi"),
		path.join(SCRATCH_UPSTREAM, "keiyoushi"),
	];
}

function getLocalAidokuRoots() {
	return {
		main: [
			path.join(CACHE_DIR, "aidoku-community-main"),
			path.join(SCRATCH_UPSTREAM, "aidoku-community-main"),
		],
		ghPages: [
			path.join(CACHE_DIR, "aidoku-community-gh-pages"),
			path.join(SCRATCH_UPSTREAM, "aidoku-community-gh-pages"),
		],
	};
}

function ensureKeiyoushiCache() {
	for (const candidate of getLocalKeiyoushiRoots()) {
		if (fs.existsSync(path.join(candidate, ".git"))) {
			return candidate;
		}
	}
	const target = path.join(CACHE_DIR, "keiyoushi");
	fs.mkdirSync(CACHE_DIR, { recursive: true });
	console.log("Cloning keiyoushi/extensions-source (sparse metadata)...");
	runGit(
		["clone", "--filter=blob:none", "--no-checkout", "--depth", "1", "--sparse", MIHON_REPO, target],
		ROOT_DIR
	);
	return target;
}

function findMihonExtensionPath(name, preferredLang) {
	const cleanName = name.toLowerCase().replace(/[^a-z0-9_-]/g, "");

	for (const root of getLocalKeiyoushiRoots()) {
		const srcRoot = path.join(root, "src");
		if (!fs.existsSync(srcRoot)) continue;
		if (preferredLang) {
			const direct = path.join(srcRoot, preferredLang, cleanName);
			if (fs.existsSync(direct)) {
				return { repoRoot: root, relPath: `src/${preferredLang}/${cleanName}`, absPath: direct };
			}
		}
		for (const langDir of fs.readdirSync(srcRoot, { withFileTypes: true })) {
			if (!langDir.isDirectory()) continue;
			const candidate = path.join(srcRoot, langDir.name, cleanName);
			if (fs.existsSync(candidate)) {
				return { repoRoot: root, relPath: `src/${langDir.name}/${cleanName}`, absPath: candidate };
			}
		}
	}

	const repoRoot = ensureKeiyoushiCache();
	const treeOutput = runGit(["ls-tree", "-d", "--name-only", "HEAD:src"], repoRoot);
	const langs = preferredLang
		? [preferredLang, ...treeOutput.split(/\r?\n/).filter((l) => l && l !== preferredLang)]
		: ["en", "all", ...treeOutput.split(/\r?\n/).filter((l) => l && l !== "en" && l !== "all")];

	let matchedRel = null;
	for (const lang of langs) {
		try {
			const sub = runGit(["ls-tree", "-d", "--name-only", `HEAD:src/${lang}`], repoRoot);
			const entries = sub.split(/\r?\n/).map((s) => s.trim());
			if (entries.includes(cleanName)) {
				matchedRel = `src/${lang}/${cleanName}`;
				break;
			}
		} catch {
			// ignore missing lang dir
		}
	}

	if (!matchedRel) {
		throw new Error(
			`Extension "${name}" not found in keiyoushi/extensions-source${preferredLang ? ` (lang=${preferredLang})` : ""}.`
		);
	}

	console.log(`Checking out ${matchedRel} from keiyoushi/extensions-source...`);
	runGit(["sparse-checkout", "add", matchedRel], repoRoot);
	runGit(["checkout"], repoRoot);
	return { repoRoot, relPath: matchedRel, absPath: path.join(repoRoot, matchedRel) };
}

function collectFilesRecursive(dir, extFilter = null) {
	const results = [];
	if (!fs.existsSync(dir)) return results;
	for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
		const full = path.join(dir, entry.name);
		if (entry.isDirectory()) {
			results.push(...collectFilesRecursive(full, extFilter));
		} else if (!extFilter || extFilter(entry.name)) {
			results.push(full);
		}
	}
	return results;
}

function uniqueSorted(items) {
	return Array.from(new Set(items.filter(Boolean))).sort();
}

function extractPreferencesFromKotlin(code) {
	const prefs = [];
	const prefRegex =
		/(SwitchPreferenceCompat|ListPreference|MultiSelectListPreference|EditTextPreference)\([^)]*\)\.apply\s*\{([\s\S]*?)\}/g;
	for (const m of code.matchAll(prefRegex)) {
		const kind = m[1];
		const body = m[2];
		const key = body.match(/\bkey\s*=\s*"([^"]+)"/)?.[1] || body.match(/\bkey\s*=\s*(\w+)/)?.[1];
		const title = body.match(/\btitle\s*=\s*"([^"]+)"/)?.[1];
		if (!key || !title) continue;
		prefs.push({ kind, key, title });
	}
	return prefs;
}

function extractFilterSpecsFromKotlin(code) {
	const specs = [];
	for (const m of code.matchAll(/class\s+(\w*Filter\w*)[^:]*:\s*([^\n{]+)/g)) {
		const className = m[1];
		const base = m[2].trim();
		let aidokuType = null;
		if (base.includes("Sort")) aidokuType = "sort";
		else if (base.includes("Text")) aidokuType = "text";
		else if (base.includes("CheckBox")) aidokuType = "check";
		else if (base.includes("Select") || base.includes("UriPartFilter")) aidokuType = "select";
		else if (base.includes("Group") || base.includes("MultiSelect") || base.includes("TriSelect")) {
			aidokuType = "multi-select";
		}
		if (aidokuType) {
			specs.push({ className, aidokuType });
		}
	}
	return specs;
}

export function parseMihonExtension(extDir, relPath = "") {
	const gradleKtsPath = path.join(extDir, "build.gradle.kts");
	const gradlePath = path.join(extDir, "build.gradle");
	const gradleFile = fs.existsSync(gradleKtsPath)
		? gradleKtsPath
		: fs.existsSync(gradlePath)
			? gradlePath
			: null;

	const gradleText = gradleFile ? fs.readFileSync(gradleFile, "utf8") : "";

	const nameMatch =
		gradleText.match(/\bname\s*=\s*"([^"]+)"/) ||
		gradleText.match(/\bextName\s*=\s*['"]([^'"]+)['"]/);
	const versionMatch =
		gradleText.match(/\bversionCode\s*=\s*(\d+)/) ||
		gradleText.match(/\bextVersionCode\s*=\s*(\d+)/);
	const themeMatch =
		gradleText.match(/\bthemePkg\s*=\s*['"]([^'"]+)['"]/) ||
		gradleText.match(/\bmultisrc\s*[:=]\s*['"]([^'"]+)['"]/);
	const contentWarningMatch = gradleText.match(/ContentWarning\.(\w+)/);
	const isNsfwLegacy = /\bisNsfw\s*=\s*true\b/.test(gradleText);

	const languages = [];
	const listMatch = gradleText.match(/listOf\(([\s\S]*?)\)\.forEach/);
	if (listMatch) {
		for (const m of listMatch[1].matchAll(/"([^"]+)"/g)) {
			if (m[1] !== "all") languages.push(m[1]);
		}
	}
	for (const m of gradleText.matchAll(/\blang\s*=\s*['"]([^'"]+)['"]/g)) {
		if (m[1] !== "it" && m[1] !== "all" && !languages.includes(m[1])) {
			languages.push(m[1]);
		}
	}
	const folderLang = relPath.split("/")[1] || "en";
	if (languages.length === 0) {
		languages.push(folderLang === "all" ? "en" : folderLang);
	}

	const baseUrls = [];
	for (const m of gradleText.matchAll(/\bbaseUrl\s*=\s*['"](https?:\/\/[^'"]+)['"]/g)) {
		baseUrls.push(m[1]);
	}
	const mirrorsBlock = gradleText.match(/mirrors\(([\s\S]*?)\)/);
	if (mirrorsBlock) {
		for (const m of mirrorsBlock[1].matchAll(/"(https?:\/\/[^"]+)"/g)) {
			baseUrls.push(m[1]);
		}
	}

	const ktFiles = collectFilesRecursive(path.join(extDir, "src"), (n) => n.endsWith(".kt"));
	const assetFiles = collectFilesRecursive(path.join(extDir, "assets"));
	const iconCandidates = [
		"res/mipmap-xxxhdpi/ic_launcher.png",
		"res/mipmap-xxhdpi/ic_launcher.png",
		"res/mipmap-xhdpi/ic_launcher.png",
		"res/mipmap-hdpi/ic_launcher.png",
		"res/mipmap-mdpi/ic_launcher.png",
	];
	const iconPath = iconCandidates
		.map((rel) => path.join(extDir, rel))
		.find((p) => fs.existsSync(p));

	const classes = [];
	const baseClasses = [];
	const selectors = [];
	const endpoints = [];
	const headers = [];
	const filters = [];
	const filterSpecs = [];
	const preferences = [];

	for (const ktPath of ktFiles) {
		const code = fs.readFileSync(ktPath, "utf8");
		for (const m of code.matchAll(/(?:abstract\s+|open\s+)?class\s+(\w+)\s*(?:\([^)]*\))?\s*:\s*([^\n{]+)/g)) {
			classes.push(m[1]);
			if (!m[0].includes("private ") && !m[1].endsWith("Filter")) {
				const bases = m[2]
					.replace(/\([^)]*\)/g, "")
					.split(",")
					.map((s) => s.trim().replace(/\(.*$/, ""))
					.filter((s) => /^[A-Z]/.test(s));
				baseClasses.push(...bases);
			}
		}
		for (const m of code.matchAll(/\.select(?:First)?\(\s*"([^"]+)"\s*\)/g)) {
			selectors.push(m[1]);
		}
		for (const m of code.matchAll(/override\s+fun\s+\w*Selector\w*\([^)]*\)\s*=\s*"([^"]+)"/g)) {
			selectors.push(m[1]);
		}
		for (const m of code.matchAll(/"(\$baseUrl\/[^"]*|https?:\/\/[^"]+|\/api\/[^"]+)"/g)) {
			endpoints.push(m[1]);
		}
		for (const m of code.matchAll(/(?:add|set)\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*\)/g)) {
			headers.push(`${m[1]}: ${m[2]}`);
		}
		for (const m of code.matchAll(/class\s+(\w*Filter\w*)\b/g)) {
			filters.push(m[1]);
		}
		filterSpecs.push(...extractFilterSpecsFromKotlin(code));
		preferences.push(...extractPreferencesFromKotlin(code));
		if (baseUrls.length === 0) {
			const ktBase = code.match(/override\s+val\s+baseUrl\s*=\s*"([^"]+)"/);
			if (ktBase) baseUrls.push(ktBase[1]);
		}
	}

	let contentRating = 0;
	if (isNsfwLegacy || contentWarningMatch?.[1] === "NSFW") contentRating = 2;
	else if (contentWarningMatch?.[1] === "MIXED" || contentWarningMatch?.[1] === "SUGGESTIVE") {
		contentRating = 1;
	}

	const folderName = path.basename(extDir);
	const idPrefix = folderLang === "all" || languages.length > 1 ? "multi" : languages[0];
	const suggestedId = `${idPrefix}.${folderName}`;

	return {
		folderName,
		relPath,
		suggestedId,
		name: nameMatch ? nameMatch[1] : folderName,
		mihonVersionCode: versionMatch ? Number.parseInt(versionMatch[1], 10) : 1,
		baseUrl: baseUrls[0] || "",
		mirrors: uniqueSorted(baseUrls),
		languages: uniqueSorted(languages),
		contentRating,
		themePkg: themeMatch ? themeMatch[1].toLowerCase() : null,
		aidokuTemplate: themeMatch ? THEME_MAP[themeMatch[1].toLowerCase()] || null : null,
		gradleFile: gradleFile ? path.basename(gradleFile) : null,
		classes: uniqueSorted(classes),
		baseClasses: uniqueSorted(baseClasses),
		selectors: uniqueSorted(selectors),
		endpoints: uniqueSorted(endpoints),
		headers: uniqueSorted(headers),
		filters: uniqueSorted(filters),
		filterSpecs,
		preferences,
		ktFiles: ktFiles.map((p) => path.relative(extDir, p).replace(/\\/g, "/")),
		assetFiles: assetFiles.map((p) => path.relative(extDir, p).replace(/\\/g, "/")),
		iconPath: iconPath || null,
	};
}

function buildFiltersJson(info) {
	const result = [];
	const seenIds = new Set();
	for (const spec of info.filterSpecs || []) {
		const id = spec.className.replace(/Filter$/, "").toLowerCase() || spec.aidokuType;
		if (seenIds.has(id)) continue;
		seenIds.add(id);
		const title = spec.className.replace(/Filter$/, "").replace(/([A-Z])/g, " $1").trim() || spec.className;
		if (spec.aidokuType === "sort") {
			result.push({
				type: "sort",
				id,
				title,
				canAscend: true,
				options: ["Popular", "Latest"],
				default: { index: 0, ascending: false },
			});
		} else if (spec.aidokuType === "text") {
			result.push({ type: "text", id, title, placeholder: title });
		} else if (spec.aidokuType === "check") {
			result.push({ type: "check", id, title });
		}
	}
	return result;
}

function buildSettingsJson(info) {
	if (!info.preferences || info.preferences.length === 0) return null;
	const items = [];
	const seen = new Set();
	for (const pref of info.preferences) {
		if (seen.has(pref.key)) continue;
		seen.add(pref.key);
		if (pref.kind === "SwitchPreferenceCompat") {
			items.push({
				type: "switch",
				key: pref.key,
				title: pref.title,
				default: false,
				refreshes: ["content"],
			});
		} else if (pref.kind === "EditTextPreference") {
			items.push({
				type: "text",
				key: pref.key,
				title: pref.title,
				default: "",
				refreshes: ["content"],
			});
		} else if (pref.kind === "ListPreference") {
			items.push({
				type: "select",
				key: pref.key,
				title: pref.title,
				values: ["default"],
				titles: ["Default"],
				default: "default",
				refreshes: ["content"],
			});
		} else if (pref.kind === "MultiSelectListPreference") {
			items.push({
				type: "multi-select",
				key: pref.key,
				title: pref.title,
				values: [],
				titles: [],
				refreshes: ["content"],
			});
		}
	}
	return items.length > 0 ? [{ type: "group", title: "Preferences", items }] : null;
}

function renderMihonDoc(info) {
	const lines = [
		`# Mihon Source Reference: ${info.name}`,
		"",
		`- **Mihon Path:** \`${info.relPath || "(local)"}\``,
		`- **Mihon Version Code:** \`${info.mihonVersionCode}\``,
		`- **Aidoku Suggested ID:** \`${info.suggestedId}\``,
		`- **Base URL:** ${info.baseUrl || "N/A"}`,
		`- **Languages (${info.languages.length}):** ${info.languages.join(", ")}`,
		`- **Content Rating:** ${info.contentRating === 2 ? "NSFW (2)" : info.contentRating === 1 ? "Contains NSFW / Mixed (1)" : "Safe (0)"}`,
		info.themePkg
			? `- **Mihon Theme:** \`${info.themePkg}\` -> Aidoku template: \`${info.aidokuTemplate || "custom"}\``
			: `- **Architecture:** Custom (${info.baseClasses.join(", ") || "KeiSource"})`,
		"",
		"## Extracted Kotlin Files",
		...info.ktFiles.map((f) => `- \`${f}\``),
	];

	if (info.assetFiles.length > 0) {
		lines.push("", "## Extracted Assets", ...info.assetFiles.map((f) => `- \`${f}\``));
	}
	if (info.endpoints.length > 0) {
		lines.push("", "## Endpoints & URLs", ...info.endpoints.slice(0, 40).map((e) => `- \`${e}\``));
	}
	if (info.selectors.length > 0) {
		lines.push("", "## CSS Selectors", ...info.selectors.slice(0, 40).map((s) => `- \`${s}\``));
	}
	if (info.filters.length > 0) {
		lines.push("", "## Filter Classes", ...info.filters.map((f) => `- \`${f}\``));
	}
	if (info.headers.length > 0) {
		lines.push("", "## Custom Headers", ...info.headers.map((h) => `- \`${h}\``));
	}

	lines.push("");
	return lines.join("\n");
}

function findAidokuCommunitySource(candidates) {
	const roots = getLocalAidokuRoots();
	for (const mainRoot of roots.main) {
		for (const id of candidates) {
			const srcDir = path.join(mainRoot, "sources", id);
			if (fs.existsSync(path.join(srcDir, "Cargo.toml"))) {
				const ghPagesRoot = roots.ghPages.find((g) => fs.existsSync(path.join(g, "sources")));
				return { id, srcDir, mainRoot, ghPagesRoot };
			}
		}
	}

	const cacheMain = path.join(CACHE_DIR, "aidoku-community-main");
	try {
		if (!fs.existsSync(path.join(cacheMain, ".git"))) {
			fs.mkdirSync(CACHE_DIR, { recursive: true });
			runGit(
				["clone", "--filter=blob:none", "--no-checkout", "--depth", "1", "--sparse", AIDOKU_REPO, cacheMain],
				ROOT_DIR
			);
		}
		const list = runGit(["ls-tree", "-d", "--name-only", "HEAD:sources"], cacheMain).split(/\r?\n/);
		for (const id of candidates) {
			if (list.includes(id)) {
				runGit(["sparse-checkout", "add", `sources/${id}`], cacheMain);
				runGit(["checkout"], cacheMain);
				return {
					id,
					srcDir: path.join(cacheMain, "sources", id),
					mainRoot: cacheMain,
					ghPagesRoot: null,
				};
			}
		}
	} catch {
		// Remote lookup optional
	}
	return null;
}

function copyReferencedTemplates(cargoTomlPath, aidokuMainRoot) {
	if (!fs.existsSync(cargoTomlPath) || !aidokuMainRoot) return;
	const toml = fs.readFileSync(cargoTomlPath, "utf8");
	for (const m of toml.matchAll(/path\s*=\s*"\.\.\/\.\.\/templates\/([^"]+)"/g)) {
		const tplName = m[1];
		const srcTpl = path.join(aidokuMainRoot, "templates", tplName);
		const dstTpl = path.join(TEMPLATES_DIR, tplName);
		if (fs.existsSync(srcTpl) && !fs.existsSync(dstTpl)) {
			copyDirRecursive(srcTpl, dstTpl);
			console.log(`Copied shared template: templates/${tplName}`);
		}
	}
}

function scaffoldAidokuCrate(id, info) {
	const crateDir = path.join(SOURCES_DIR, id);
	const resDir = path.join(crateDir, "res");
	const srcDir = path.join(crateDir, "src");
	const cargoCfgDir = path.join(crateDir, ".cargo");
	fs.mkdirSync(resDir, { recursive: true });
	fs.mkdirSync(srcDir, { recursive: true });
	fs.mkdirSync(cargoCfgDir, { recursive: true });

	const crateName = info.folderName.replace(/[^a-zA-Z0-9_-]/g, "_");
	const structName = info.name.replace(/[^a-zA-Z0-9]/g, "") || "CustomSource";

	fs.writeFileSync(
		path.join(cargoCfgDir, "config.toml"),
		`[build]\ntarget = "wasm32-unknown-unknown"\n\n[target.wasm32-unknown-unknown]\nrunner = "aidoku-test-runner"\n`,
		"utf8"
	);

	fs.writeFileSync(
		path.join(crateDir, "Cargo.toml"),
		`[package]
name = "${crateName}"
version = "0.1.0"
edition = "2024"

[dependencies]
aidoku = { git = "https://github.com/Aidoku/aidoku-rs.git", version = "0.3.0", features = ["json"] }
serde = { version = "1.0", default-features = false, features = ["derive", "alloc"] }
serde_json = { version = "1.0", default-features = false, features = ["alloc"] }

[dev-dependencies]
aidoku = { git = "https://github.com/Aidoku/aidoku-rs.git", features = ["test"] }
aidoku-test = { git = "https://github.com/Aidoku/aidoku-rs.git" }

[lib]
crate-type = ["cdylib"]

[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
opt-level = "s"
strip = true
lto = true
`,
		"utf8"
	);

	writeJson(path.join(resDir, "source.json"), {
		info: {
			id,
			name: info.name,
			version: 1,
			url: info.baseUrl || `https://${info.folderName}.com`,
			contentRating: info.contentRating,
			languages: info.languages,
			minAppVersion: "0.7.1",
		},
		listings: [
			{ id: "popular", name: "Popular" },
			{ id: "latest", name: "Latest Updates" },
		],
	});

	writeJson(path.join(resDir, "filters.json"), buildFiltersJson(info));
	const settingsJson = buildSettingsJson(info);
	if (settingsJson) {
		writeJson(path.join(resDir, "settings.json"), settingsJson);
	}

	if (info.iconPath && fs.existsSync(info.iconPath)) {
		fs.copyFileSync(info.iconPath, path.join(resDir, "icon.png"));
	}

	const firstSelector = info.selectors[0] || "div.manga-item a";
	const libRs = `#![no_std]
use aidoku::{
	Chapter, DeepLinkHandler, DeepLinkResult, FilterValue, Listing, ListingProvider, Manga,
	MangaPageResult, Page, PageContent, Result, Source,
	alloc::{String, Vec},
	imports::net::Request,
	prelude::*,
};

const BASE_URL: &str = "${info.baseUrl || "https://example.com"}";

// Ported directly from Mihon (${info.relPath})
// Selectors found: ${info.selectors.slice(0, 6).join(", ") || "none"}
// Endpoints found: ${info.endpoints.slice(0, 6).join(", ") || "none"}
struct ${structName};

impl Source for ${structName} {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		_filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		let url = match query {
			Some(q) if !q.is_empty() => format!("{BASE_URL}/search?q={q}&page={page}"),
			_ => format!("{BASE_URL}/browse?page={page}"),
		};
		let html = Request::get(&url)?.html()?;
		let entries = html
			.select("${firstSelector}")
			.map(|els| {
				els.filter_map(|el| {
					let title = el.text()?;
					let href = el.attr("abs:href").or_else(|| el.attr("href"))?;
					let key = href.strip_prefix(BASE_URL).map(String::from).unwrap_or(href.clone());
					Some(Manga {
						key,
						title,
						url: Some(href),
						..Default::default()
					})
				})
				.collect()
			})
			.unwrap_or_default();
		Ok(MangaPageResult {
			entries,
			has_next_page: false,
		})
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		_needs_details: bool,
		_needs_chapters: bool,
	) -> Result<Manga> {
		// TODO: Complete details and chapters parsing using MIHON_SOURCE.md reference
		manga.chapters = Some(Vec::<Chapter>::new());
		Ok(manga)
	}

	fn get_page_list(&self, _manga: Manga, _chapter: Chapter) -> Result<Vec<Page>> {
		// TODO: Complete page extraction using MIHON_SOURCE.md reference
		let _ = PageContent::url("");
		Ok(Vec::<Page>::new())
	}
}

impl ListingProvider for ${structName} {
	fn get_manga_list(&self, _listing: Listing, page: i32) -> Result<MangaPageResult> {
		self.get_search_manga_list(None, page, Vec::new())
	}
}

impl DeepLinkHandler for ${structName} {
	fn handle_deep_link(&self, url: String) -> Result<Option<DeepLinkResult>> {
		let Some(path) = url.strip_prefix(BASE_URL) else {
			return Ok(None);
		};
		Ok(Some(DeepLinkResult::Manga { key: path.into() }))
	}
}

register_source!(${structName}, ListingProvider, DeepLinkHandler);
`;
	fs.writeFileSync(path.join(srcDir, "lib.rs"), libRs, "utf8");
	return crateDir;
}

function updateExtensionsRegistry(id, info, mihonRelPath, status) {
	const registry = fs.existsSync(EXTENSIONS_FILE)
		? readJson(EXTENSIONS_FILE)
		: {
				name: "Mlyns Personal Sources",
				repository: "mhaid",
				mihonUpstream: "https://github.com/keiyoushi/extensions-source",
				aidokuUpstream: "https://github.com/Aidoku-Community/sources",
				sources: [],
			};

	const sourceJsonPath = path.join(SOURCES_DIR, id, "res", "source.json");
	const srcManifest = fs.existsSync(sourceJsonPath) ? readJson(sourceJsonPath).info || {} : {};
	const version = srcManifest.version ?? 1;

	const entry = {
		id,
		name: srcManifest.name || info.name,
		mihonPath: mihonRelPath,
		aidokuPath: `sources/${id}`,
		version,
		languages: srcManifest.languages || info.languages,
		contentRating: srcManifest.contentRating ?? info.contentRating,
		baseURL: srcManifest.url || info.baseUrl,
		minAppVersion: srcManifest.minAppVersion || "0.7.1",
		packageFile: `${id}-v${version}.aix`,
		iconFile: `${id}-v${version}.png`,
		status,
	};

	const idx = registry.sources.findIndex((s) => s.id === id);
	if (idx === -1) {
		registry.sources.push(entry);
	} else {
		registry.sources[idx] = entry;
	}
	writeJson(EXTENSIONS_FILE, registry);
}

function syncDocs() {
	if (!fs.existsSync(EXTENSIONS_FILE)) {
		throw new Error("extensions.json not found");
	}
	const registry = readJson(EXTENSIONS_FILE);
	let updated = 0;

	for (const entry of registry.sources || []) {
		const mihonPath = entry.mihonPath;
		if (!mihonPath) continue;
		const parts = mihonPath.split("/");
		const lang = parts[1];
		const extName = parts[2];
		const found = findMihonExtensionPath(extName, lang);
		const parsed = parseMihonExtension(found.absPath, found.relPath);
		const outPath = path.join(ROOT_DIR, entry.aidokuPath, "MIHON_SOURCE.md");
		fs.writeFileSync(outPath, renderMihonDoc(parsed), "utf8");
		console.log(`[doc] Updated ${path.relative(ROOT_DIR, outPath)}`);
		updated++;
	}
	console.log(`Synced MIHON_SOURCE.md for ${updated} extension(s).`);
}

function listExtensions() {
	const registry = readJson(EXTENSIONS_FILE);
	console.log(`Repository: ${registry.name} (${registry.sources.length} extensions)`);
	for (const s of registry.sources) {
		console.log(`- ${s.id} (v${s.version}) [${s.status}] <- ${s.mihonPath} (${s.baseURL})`);
	}
}

function addExtension(name, opts) {
	const found = findMihonExtensionPath(name, opts.lang);
	const info = parseMihonExtension(found.absPath, found.relPath);

	if (opts.dryRun) {
		console.log(JSON.stringify(info, null, 2));
		return;
	}

	const candidateIds = uniqueSorted([
		info.suggestedId,
		`en.${info.folderName}`,
		`multi.${info.folderName}`,
		...(opts.lang ? [`${opts.lang}.${info.folderName}`] : []),
	]);

	let finalId = info.suggestedId;
	let status = "scaffold";

	const communityMatch = opts.useCommunityFallback
		? findAidokuCommunitySource(candidateIds)
		: null;

	if (communityMatch) {
		finalId = communityMatch.id;
		const destDir = path.join(SOURCES_DIR, finalId);
		copyDirRecursive(communityMatch.srcDir, destDir);
		copyReferencedTemplates(path.join(destDir, "Cargo.toml"), communityMatch.mainRoot);

		const srcJson = readJson(path.join(destDir, "res", "source.json")).info || {};
		const ver = srcJson.version || 1;

		if (communityMatch.ghPagesRoot) {
			const aixName = `${finalId}-v${ver}.aix`;
			const iconName = `${finalId}-v${ver}.png`;
			const aixSrc = path.join(communityMatch.ghPagesRoot, "sources", aixName);
			const iconSrc = path.join(communityMatch.ghPagesRoot, "icons", iconName);
			if (fs.existsSync(aixSrc)) {
				fs.mkdirSync(path.join(PUBLIC_DIR, "sources"), { recursive: true });
				fs.copyFileSync(aixSrc, path.join(PUBLIC_DIR, "sources", aixName));
			}
			if (fs.existsSync(iconSrc)) {
				fs.mkdirSync(path.join(PUBLIC_DIR, "icons"), { recursive: true });
				fs.copyFileSync(iconSrc, path.join(PUBLIC_DIR, "icons", iconName));
			}
		}
		status = "ready";
		console.log(`Imported Aidoku crate "${finalId}" from Aidoku-Community/sources (--use-community-fallback).`);
	} else {
		scaffoldAidokuCrate(finalId, info);
		status = "scaffold";
		console.log(
			`Converted Mihon source "${found.relPath}" into Aidoku Rust crate in sources/${finalId}/ (complete any custom Rust parsing using MIHON_SOURCE.md, then push to main to compile via GitHub Actions).`
		);
	}

	fs.writeFileSync(path.join(SOURCES_DIR, finalId, "MIHON_SOURCE.md"), renderMihonDoc(info), "utf8");
	updateExtensionsRegistry(finalId, info, found.relPath, status);

	execFileSync(process.execPath, [path.join(__dirname, "build-repo.mjs")], {
		cwd: ROOT_DIR,
		stdio: "inherit",
	});
}

function main() {
	const args = process.argv.slice(2);
	if (args.length === 0 || args.includes("--help") || args.includes("-h")) {
		printHelp();
		return;
	}
	if (args.includes("--list")) {
		listExtensions();
		return;
	}
	if (args.includes("--sync-docs")) {
		syncDocs();
		return;
	}

	let lang = null;
	const langIdx = args.indexOf("--lang");
	if (langIdx !== -1 && args[langIdx + 1]) {
		lang = args[langIdx + 1];
	}
	const useCommunityFallback = args.includes("--use-community-fallback");
	const dryRun = args.includes("--dry-run");

	const name = args.find((a, idx) => !a.startsWith("-") && (langIdx === -1 || idx !== langIdx + 1));
	if (!name) {
		printHelp();
		process.exit(1);
	}

	addExtension(name, { lang, useCommunityFallback, dryRun });
}

const isCli = process.argv[1] && path.resolve(process.argv[1]) === __filename;
if (isCli) {
	main();
}
