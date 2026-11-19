#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT_DIR = path.resolve(__dirname, "..");
const EXTENSIONS_FILE = path.join(ROOT_DIR, "extensions.json");
const SOURCES_DIR = path.join(ROOT_DIR, "sources");
const PUBLIC_DIR = path.join(ROOT_DIR, "public");
const CACHE_DIR = path.join(ROOT_DIR, ".cache");
const SCRATCH_UPSTREAM =
	"C:/Users/user/.gemini/antigravity/brain/7e2fff51-f1df-47e0-84d7-a6443bb6fd07/scratch/upstream";

const MIHON_REPO = "https://github.com/keiyoushi/extensions-source.git";

function printHelp() {
	console.log(`Mlyn Aidoku Sources CLI (mhaid)

Usage:
  node scripts/mhaid.mjs <command> [args]

Commands:
  list                                Show all extensions, versions, and auto-sync status
  port <mihon-name> [options]         Scaffold or import a Mihon extension and generate a porting brief
      --lang <code>                   Preferred language folder in keiyoushi/extensions-source
      --community                     Import existing crate from Aidoku-Community/sources instead of scaffold
      --watch                         Mark extension autoSync: true in extensions.json
      --brief                         Force writing full Mihon Kotlin/JS bundle to .cache/briefs/<id>.md
  diff <id-or-name>                   Show upstream Mihon commits & diff since lastMihonCommit
  watch <id-or-name...>               Enable automatic upstream tracking (autoSync: true)
  unwatch <id-or-name...>             Disable automatic upstream tracking (autoSync: false)
  sync [--dry-run] [--id <id>] [--all] Run upstream Mihon sync for watched extensions
  bump <id-or-name>                   Increment source version in source.json and extensions.json
  pull-pages                          Sync built .aix, icons, and indexes from origin/gh-pages into public/ and verify
  verify                              Validate manifests, settings/filters schemas, 0.8.4 compatibility, and .aix archives
`);
}

function readJson(filePath) {
	return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function writeJson(filePath, data) {
	fs.writeFileSync(filePath, JSON.stringify(data, null, "\t") + "\n", "utf8");
}

function runGit(args, cwd = ROOT_DIR) {
	return execFileSync("git", args, {
		cwd,
		encoding: "utf8",
		stdio: ["ignore", "pipe", "pipe"],
	}).trim();
}

function runGitBuf(args, cwd = ROOT_DIR) {
	return execFileSync("git", args, {
		cwd,
		stdio: ["ignore", "pipe", "pipe"],
	});
}

function resolveEntries(registry, queries) {
	const matched = [];
	const seen = new Set();
	for (const raw of queries) {
		const q = raw.trim().toLowerCase();
		if (!q) continue;
		const hits = (registry.sources || []).filter((s) => {
			const tail = (s.mihonPath || "").split("/").pop()?.toLowerCase() || "";
			const idTail = s.id.split(".").pop()?.toLowerCase() || "";
			return (
				s.id.toLowerCase() === q ||
				s.name.toLowerCase() === q ||
				tail === q ||
				idTail === q
			);
		});
		if (hits.length === 0) {
			throw new Error(`No extension matched "${raw}". Run "node scripts/mhaid.mjs list" to see IDs.`);
		}
		for (const hit of hits) {
			if (!seen.has(hit.id)) {
				seen.add(hit.id);
				matched.push(hit);
			}
		}
	}
	return matched;
}

function ensureKeiyoushiRepoForPath(mihonPath) {
	const candidates = [
		path.join(CACHE_DIR, "keiyoushi"),
		path.join(SCRATCH_UPSTREAM, "keiyoushi"),
	];
	let repoDir = candidates.find((p) => fs.existsSync(path.join(p, ".git")));
	if (!repoDir) {
		repoDir = path.join(CACHE_DIR, "keiyoushi");
		fs.mkdirSync(CACHE_DIR, { recursive: true });
		console.log(`Cloning ${MIHON_REPO} into .cache/keiyoushi...`);
		runGit([
			"clone",
			"--filter=blob:none",
			"--no-checkout",
			"--sparse",
			"--branch",
			"main",
			MIHON_REPO,
			repoDir,
		]);
	}

	console.log(`Updating Mihon upstream for ${mihonPath}...`);
	try {
		runGit(["fetch", "--filter=blob:none", "origin", "main"], repoDir);
	} catch (err) {
		console.warn(`[warn] Fetch failed (${err.message}); using cached commits if available.`);
	}
	try {
		runGit(["sparse-checkout", "add", mihonPath], repoDir);
	} catch {
		runGit(["sparse-checkout", "set", "--no-cone", mihonPath], repoDir);
	}
	runGit(["checkout", "-B", "main", "FETCH_HEAD"], repoDir);
	return repoDir;
}

function collectFilesRecursive(dir) {
	const out = [];
	if (!fs.existsSync(dir)) return out;
	for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
		const full = path.join(dir, entry.name);
		if (entry.isDirectory()) {
			out.push(...collectFilesRecursive(full));
		} else {
			out.push(full);
		}
	}
	return out.sort();
}

function writePortingBrief(entry, keiyoushiDir) {
	const mihonDir = path.join(keiyoushiDir, entry.mihonPath);
	if (!fs.existsSync(mihonDir)) {
		throw new Error(`Mihon directory not found: ${mihonDir}`);
	}

	const briefsDir = path.join(CACHE_DIR, "briefs");
	fs.mkdirSync(briefsDir, { recursive: true });
	const briefPath = path.join(briefsDir, `${entry.id}.md`);

	const mihonFiles = collectFilesRecursive(mihonDir).filter(
		(f) => /\.(kt|kts|js|json|txt)$/i.test(f)
	);

	const latestSha = runGit(["log", "-n", "1", "--format=%H", "--", entry.mihonPath], keiyoushiDir);

	const parts = [
		`# Porting Brief: ${entry.name} (\`${entry.id}\`)`,
		``,
		`- **Mihon Path:** \`${entry.mihonPath}\``,
		`- **Mihon HEAD Commit:** \`${latestSha}\``,
		`- **Aidoku Crate:** \`${entry.aidokuPath}\``,
		`- **Target minAppVersion:** \`${entry.minAppVersion || "0.8.4"}\``,
		``,
		`## Aidoku Compatibility Checklist`,
		`- \`#![no_std]\` Rust edition 2024 (\`aidoku::alloc::{String, Vec, string::ToString, vec}\`).`,
		`- For \`minAppVersion < 0.9.0\`: NEVER combine \`DynamicSettings\` with \`LoginSetting\` (put static \`login\` items in \`res/settings.json\`) and NEVER call \`eval_async\`, \`add_user_script\`, \`set_rule_list\`, \`get_cookies\`, or \`delete_cookie\` (use \`WebView::new()\`, \`load()\`, \`eval()\` polling).`,
		`- Check \`ImageRequestProvider\` headers (strip \`Referer\` if CDNs block hotlinking) and verify with \`node scripts/build-repo.mjs --verify\`.`,
		``,
		`## Mihon Source Files (${mihonFiles.length})`,
		``,
	];

	for (const file of mihonFiles) {
		const rel = path.relative(mihonDir, file).replace(/\\/g, "/");
		const ext = path.extname(file).slice(1) || "text";
		const lang = ext === "kt" || ext === "kts" ? "kotlin" : ext;
		const content = fs.readFileSync(file, "utf8");
		parts.push(`### \`${rel}\``, ``, `\`\`\`${lang}`, content.trimEnd(), `\`\`\``, ``);
	}

	fs.writeFileSync(briefPath, parts.join("\n"), "utf8");
	console.log(`Generated porting brief: ${path.relative(ROOT_DIR, briefPath)}`);
}

function cmdList() {
	const registry = readJson(EXTENSIONS_FILE);
	const rows = (registry.sources || []).map((s) => ({
		id: s.id,
		name: s.name,
		ver: `v${s.version}`,
		sync: s.autoSync ? (s.customRust ? "WATCH*" : "WATCH") : "manual",
		minApp: s.minAppVersion || "-",
		mihon: s.mihonPath || "-",
		baseURL: s.baseURL || "-",
	}));

	const pad = (val, w) => String(val).padEnd(w);
	const wId = Math.max(4, ...rows.map((r) => r.id.length));
	const wName = Math.max(4, ...rows.map((r) => r.name.length));
	const wVer = Math.max(3, ...rows.map((r) => r.ver.length));
	const wSync = Math.max(6, ...rows.map((r) => r.sync.length));
	const wMin = Math.max(6, ...rows.map((r) => r.minApp.length));
	const wMihon = Math.max(10, ...rows.map((r) => r.mihon.length));

	const watchCount = (registry.sources || []).filter((s) => s.autoSync).length;
	console.log(
		`${registry.name} — ${(registry.sources || []).length} sources (${watchCount} auto-synced [* = custom Rust])`
	);
	console.log(
		`${pad("ID", wId)}  ${pad("Name", wName)}  ${pad("Ver", wVer)}  ${pad("Sync", wSync)}  ${pad("MinApp", wMin)}  ${pad("Mihon Path", wMihon)}  Base URL`
	);
	console.log("-".repeat(wId + wName + wVer + wSync + wMin + wMihon + 24));
	for (const r of rows) {
		console.log(
			`${pad(r.id, wId)}  ${pad(r.name, wName)}  ${pad(r.ver, wVer)}  ${pad(r.sync, wSync)}  ${pad(r.minApp, wMin)}  ${pad(r.mihon, wMihon)}  ${r.baseURL}`
		);
	}
}

function cmdPort(args) {
	const langIdx = args.indexOf("--lang");
	const lang = langIdx !== -1 ? args[langIdx + 1] : null;
	const watch = args.includes("--watch");
	const community = args.includes("--community");
	const brief = args.includes("--brief");

	const name = args.find((a, idx) => !a.startsWith("-") && (langIdx === -1 || idx !== langIdx + 1));
	if (!name) {
		throw new Error("Usage: node scripts/mhaid.mjs port <mihon-name> [--lang <code>] [--watch] [--community] [--brief]");
	}

	const subArgs = [path.join(__dirname, "mihon-to-aidoku.mjs"), name];
	if (lang) subArgs.push("--lang", lang);
	if (community) subArgs.push("--use-community-fallback");

	execFileSync(process.execPath, subArgs, { cwd: ROOT_DIR, stdio: "inherit" });

	const registry = readJson(EXTENSIONS_FILE);
	const matched = resolveEntries(registry, [name]);
	if (watchedUpdate(registry, matched, watch)) {
		writeJson(EXTENSIONS_FILE, registry);
	}

	for (const entry of matched) {
		if (brief || entry.status === "scaffold") {
			const keiyoushiDir = ensureKeiyoushiRepoForPath(entry.mihonPath);
			writePortingBrief(entry, keiyoushiDir);
		}
	}
}

function watchedUpdate(registry, matched, enable) {
	if (!enable) return false;
	let dirty = false;
	for (const entry of matched) {
		if (entry.autoSync !== true) {
			entry.autoSync = true;
			dirty = true;
			console.log(`[watch] Enabled autoSync for ${entry.id}`);
		}
	}
	return dirty;
}

function cmdSetWatch(queries, enable) {
	if (queries.length === 0) {
		throw new Error(`Usage: node scripts/mhaid.mjs ${enable ? "watch" : "unwatch"} <id-or-name...>`);
	}
	const registry = readJson(EXTENSIONS_FILE);
	const matched = resolveEntries(registry, queries);
	for (const entry of matched) {
		entry.autoSync = enable;
		console.log(`[${enable ? "watch" : "unwatch"}] ${entry.id}: autoSync=${enable}`);
	}
	writeJson(EXTENSIONS_FILE, registry);
}

function cmdDiff(args) {
	const query = args.find((a) => !a.startsWith("-"));
	if (!query) {
		throw new Error("Usage: node scripts/mhaid.mjs diff <id-or-name>");
	}
	const registry = readJson(EXTENSIONS_FILE);
	const [entry] = resolveEntries(registry, [query]);
	if (!entry.mihonPath) {
		throw new Error(`${entry.id} has no mihonPath recorded in extensions.json`);
	}

	const repoDir = ensureKeiyoushiRepoForPath(entry.mihonPath);
	const latest = runGit(["log", "-n", "1", "--format=%H", "--", entry.mihonPath], repoDir);
	const baseSha = entry.lastMihonCommit;

	if (!baseSha) {
		console.log(`${entry.id}: no lastMihonCommit stored; showing latest 10 commits for ${entry.mihonPath}:`);
		console.log(runGit(["log", "-n", "10", "--oneline", "--", entry.mihonPath], repoDir));
		writePortingBrief(entry, repoDir);
		return;
	}

	try {
		runGit(["cat-file", "-e", `${baseSha}^{commit}`], repoDir);
	} catch {
		try {
			runGit(["fetch", "--filter=blob:none", "origin", baseSha], repoDir);
		} catch {
			// Ignore if shallow object unavailable
		}
	}

	if (baseSha === latest) {
		console.log(`${entry.id} (${entry.mihonPath}) is already at latest Mihon commit ${latest.slice(0, 10)}.`);
		return;
	}

	console.log(`=== ${entry.id}: ${entry.mihonPath} (${baseSha.slice(0, 10)} -> ${latest.slice(0, 10)}) ===`);
	console.log(runGit(["log", "--oneline", `${baseSha}..${latest}`, "--", entry.mihonPath], repoDir));
	console.log("");

	const diffText = runGit(["diff", baseSha, latest, "--", entry.mihonPath], repoDir);
	const lines = diffText.split(/\r?\n/);
	if (lines.length > 350) {
		const diffsDir = path.join(CACHE_DIR, "diffs");
		fs.mkdirSync(diffsDir, { recursive: true });
		const diffFile = path.join(diffsDir, `${entry.id}.diff`);
		fs.writeFileSync(diffFile, diffText, "utf8");
		console.log(runGit(["diff", "--stat", baseSha, latest, "--", entry.mihonPath], repoDir));
		console.log(`\nFull diff (${lines.length} lines) written to ${path.relative(ROOT_DIR, diffFile)}`);
	} else {
		console.log(diffText);
	}
}

function cmdSync(args) {
	execFileSync(process.execPath, [path.join(__dirname, "sync-upstream.mjs"), ...args], {
		cwd: ROOT_DIR,
		stdio: "inherit",
	});
}

function cmdBump(args) {
	const query = args.find((a) => !a.startsWith("-"));
	if (!query) {
		throw new Error("Usage: node scripts/mhaid.mjs bump <id-or-name>");
	}
	const registry = readJson(EXTENSIONS_FILE);
	const [entry] = resolveEntries(registry, [query]);
	const sourceJsonPath = path.join(ROOT_DIR, entry.aidokuPath, "res", "source.json");
	const manifest = readJson(sourceJsonPath);
	const oldVer = Number(manifest?.info?.version ?? entry.version ?? 1);
	const newVer = oldVer + 1;
	manifest.info.version = newVer;
	writeJson(sourceJsonPath, manifest);

	entry.version = newVer;
	entry.packageFile = `${entry.id}-v${newVer}.aix`;
	entry.iconFile = `${entry.id}-v${newVer}.png`;
	writeJson(EXTENSIONS_FILE, registry);
	console.log(`Bumped ${entry.id}: v${oldVer} -> v${newVer} (res/source.json & extensions.json)`);
}

function cmdPullPages() {
	console.log("Fetching origin/gh-pages...");
	runGit(["fetch", "origin", "gh-pages"]);

	const files = runGit(["ls-tree", "-r", "--name-only", "origin/gh-pages"])
		.split(/\r?\n/)
		.map((s) => s.trim())
		.filter(Boolean);

	const ghSources = new Set();
	const ghIcons = new Set();

	for (const rel of files) {
		const isTracked =
			rel === "index.json" ||
			rel === "index.min.json" ||
			rel === ".nojekyll" ||
			rel.startsWith("sources/") ||
			rel.startsWith("icons/");
		if (!isTracked) continue;

		if (rel.startsWith("sources/")) ghSources.add(path.basename(rel));
		if (rel.startsWith("icons/")) ghIcons.add(path.basename(rel));

		const dst = path.join(PUBLIC_DIR, rel);
		fs.mkdirSync(path.dirname(dst), { recursive: true });
		const buf = runGitBuf(["show", `origin/gh-pages:${rel}`]);
		fs.writeFileSync(dst, buf);
	}

	const pubSourcesDir = path.join(PUBLIC_DIR, "sources");
	if (fs.existsSync(pubSourcesDir)) {
		for (const f of fs.readdirSync(pubSourcesDir)) {
			if (!ghSources.has(f)) {
				fs.unlinkSync(path.join(pubSourcesDir, f));
				console.log(`Removed superseded public/sources/${f}`);
			}
		}
	}

	const pubIconsDir = path.join(PUBLIC_DIR, "icons");
	if (fs.existsSync(pubIconsDir)) {
		for (const f of fs.readdirSync(pubIconsDir)) {
			if (!ghIcons.has(f)) {
				fs.unlinkSync(path.join(pubIconsDir, f));
				console.log(`Removed superseded public/icons/${f}`);
			}
		}
	}

	execFileSync(process.execPath, [path.join(__dirname, "build-repo.mjs"), "--verify"], {
		cwd: ROOT_DIR,
		stdio: "inherit",
	});
}

function cmdVerify() {
	execFileSync(process.execPath, [path.join(__dirname, "build-repo.mjs"), "--verify"], {
		cwd: ROOT_DIR,
		stdio: "inherit",
	});
}

function main() {
	const [cmd, ...rest] = process.argv.slice(2);
	if (!cmd || cmd === "--help" || cmd === "-h" || cmd === "help") {
		printHelp();
		return;
	}

	switch (cmd) {
		case "list":
			cmdList();
			break;
		case "port":
			cmdPort(rest);
			break;
		case "diff":
			cmdDiff(rest);
			break;
		case "watch":
			cmdSetWatch(rest, true);
			break;
		case "unwatch":
			cmdSetWatch(rest, false);
			break;
		case "sync":
			cmdSync(rest);
			break;
		case "bump":
			cmdBump(rest);
			break;
		case "pull-pages":
			cmdPullPages();
			break;
		case "verify":
			cmdVerify();
			break;
		default:
			console.error(`Unknown command: ${cmd}\n`);
			printHelp();
			process.exit(1);
	}
}

main();
