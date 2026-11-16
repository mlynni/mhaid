#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { parseMihonExtension, renderMihonDoc } from "./mihon-to-aidoku.mjs";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT_DIR = path.resolve(__dirname, "..");
const EXTENSIONS_FILE = path.join(ROOT_DIR, "extensions.json");
const CACHE_DIR = path.join(ROOT_DIR, ".cache");
const SCRATCH_UPSTREAM =
	"C:/Users/user/.gemini/antigravity/brain/7e2fff51-f1df-47e0-84d7-a6443bb6fd07/scratch/upstream";

const MIHON_REPO = "https://github.com/keiyoushi/extensions-source.git";
const AIDOKU_REPO = "https://github.com/Aidoku-Community/sources.git";

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

function writeJson(filePath, data) {
	fs.writeFileSync(filePath, JSON.stringify(data, null, "\t") + "\n", "utf8");
}

function findOrCloneRepo(cacheName, scratchName, repoUrl, branch, sparsePaths, offlineOnly) {
	const candidates = [
		path.join(CACHE_DIR, cacheName),
		path.join(SCRATCH_UPSTREAM, scratchName),
	];
	let repoDir = candidates.find((p) => fs.existsSync(path.join(p, ".git")));

	if (!repoDir) {
		if (offlineOnly) {
			throw new Error(`Upstream cache not found locally for ${cacheName}`);
		}
		repoDir = path.join(CACHE_DIR, cacheName);
		fs.mkdirSync(CACHE_DIR, { recursive: true });
		console.log(`Cloning ${repoUrl} (treeless sparse checkout)...`);
		runGit(
			[
				"clone",
				"--filter=blob:none",
				"--no-checkout",
				"--sparse",
				"--branch",
				branch,
				repoUrl,
				repoDir,
			],
			ROOT_DIR
		);
	}

	if (!offlineOnly) {
		console.log(`Fetching updates for ${cacheName}...`);
		const isShallow = runGit(["rev-parse", "--is-shallow-repository"], repoDir) === "true";
		const fetchArgs = ["fetch", "--filter=blob:none"];
		if (isShallow) {
			fetchArgs.push("--unshallow");
		}
		fetchArgs.push("origin", branch);
		try {
			runGit(fetchArgs, repoDir);
		} catch {
			// Fallback if repo was already unshallow or remote refuses --unshallow
			runGit(["fetch", "--filter=blob:none", "origin", branch], repoDir);
		}

		if (sparsePaths.length > 0) {
			runGit(["sparse-checkout", "set", "--no-cone", ...sparsePaths], repoDir);
		}
		runGit(["checkout", "-B", branch, "FETCH_HEAD"], repoDir);
	}

	return repoDir;
}

function getLatestCommitForPath(repoDir, relPath) {
	const sha = runGit(["log", "-n", "1", "--format=%H", "--", relPath], repoDir);
	if (sha) return sha;
	return runGit(["rev-parse", "HEAD"], repoDir);
}

function getChangedFilesBetween(repoDir, oldCommit, newCommit, relPath) {
	if (!oldCommit || oldCommit === newCommit) return [];
	try {
		// Ensure oldCommit exists if shallow; otherwise git diff throws and we fetch it
		runGit(["cat-file", "-e", `${oldCommit}^{commit}`], repoDir);
	} catch {
		try {
			runGit(["fetch", "--filter=blob:none", "origin", oldCommit], repoDir);
		} catch {
			// If commit cannot be fetched, treat folder as changed
			return [relPath];
		}
	}
	const out = runGit(["diff", "--name-only", oldCommit, newCommit, "--", relPath], repoDir);
	return out
		.split(/\r?\n/)
		.map((s) => s.trim())
		.filter(Boolean);
}

function syncBaseUrls(crateDir, info) {
	const sourceJsonPath = path.join(crateDir, "res", "source.json");
	if (!fs.existsSync(sourceJsonPath)) return false;

	const srcManifest = readJson(sourceJsonPath);
	const manifestInfo = srcManifest.info || {};
	let changed = false;

	const mihonPrimary = info.baseUrl;
	const mihonMirrors = Array.isArray(info.mirrors) && info.mirrors.length > 0 ? info.mirrors : [];

	let previousUrl = null;
	if (typeof manifestInfo.url === "string" && mihonPrimary && manifestInfo.url !== mihonPrimary) {
		previousUrl = manifestInfo.url;
		manifestInfo.url = mihonPrimary;
		changed = true;
	} else if (Array.isArray(manifestInfo.urls) && mihonMirrors.length > 0) {
		const missing = mihonMirrors.filter((u) => !manifestInfo.urls.includes(u));
		if (missing.length > 0) {
			previousUrl = manifestInfo.urls[0];
			manifestInfo.urls = Array.from(new Set([...manifestInfo.urls, ...missing]));
			changed = true;
		}
	}

	if (changed) {
		srcManifest.info = manifestInfo;
		writeJson(sourceJsonPath, srcManifest);

		// Also update matching BASE_URL literal inside Rust files if the old primary URL was hardcoded
		if (previousUrl && mihonPrimary) {
			const rustDir = path.join(crateDir, "src");
			if (fs.existsSync(rustDir)) {
				for (const f of fs.readdirSync(rustDir)) {
					if (!f.endsWith(".rs")) continue;
					const rp = path.join(rustDir, f);
					const code = fs.readFileSync(rp, "utf8");
					const updated = code.replaceAll(`"${previousUrl}"`, `"${mihonPrimary}"`);
					if (updated !== code) {
						fs.writeFileSync(rp, updated, "utf8");
						console.log(`  [url] Updated "${previousUrl}" -> "${mihonPrimary}" in src/${f}`);
					}
				}
			}
		}
	}

	return changed;
}

function collectCrateFiles(crateDir) {
	const files = {};
	for (const sub of ["src", "res"]) {
		const dir = path.join(crateDir, sub);
		if (!fs.existsSync(dir)) continue;
		for (const f of fs.readdirSync(dir)) {
			if (f.endsWith(".png")) continue;
			const rel = `${sub}/${f}`;
			files[rel] = fs.readFileSync(path.join(dir, f), "utf8");
		}
	}
	return files;
}

async function applyGeminiPort(entry, crateDir, diffText) {
	const apiKey = process.env.GEMINI_API_KEY;
	if (!apiKey || !diffText.trim()) {
		return { applied: false, reason: apiKey ? "empty-diff" : "no-api-key" };
	}

	const currentFiles = collectCrateFiles(crateDir);
	const filesBlock = Object.entries(currentFiles)
		.map(([rel, content]) => `### ${rel}\n\`\`\`\n${content.slice(0, 12000)}\n\`\`\``)
		.join("\n\n");

	const prompt = `You are porting an upstream Mihon extension update (Kotlin diff) to an existing Aidoku Rust (aidoku v0.3.0 / #![no_std]) extension crate "${entry.id}".

Upstream Mihon Kotlin diff for ${entry.mihonPath}:
\`\`\`diff
${diffText.slice(0, 35000)}
\`\`\`

Current Aidoku crate files:
${filesBlock}

Task:
Determine whether the Mihon diff changes runtime behavior (API endpoints, CSS selectors, auth/headers, descrambling/crypto, URL structures, filters, etc.) that is NOT already handled by the Rust code.
- If the diff is only Kotlin refactoring, build.gradle tweaks already covered, or irrelevant Android UI boilerplate, respond with:
  { "needsUpdate": false, "summary": "brief reason", "files": {} }
- Otherwise, respond with ONLY valid JSON containing the complete updated file contents for ONLY the files in src/ or res/ that need changes (never bump res/source.json version yourself — the script bumps it):
  { "needsUpdate": true, "summary": "what changed", "files": { "src/lib.rs": "full updated file content" } }`;

	const endpoint = `https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-pro:generateContent?key=${encodeURIComponent(apiKey)}`;
	const resp = await fetch(endpoint, {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify({
			contents: [{ role: "user", parts: [{ text: prompt }] }],
			generationConfig: {
				temperature: 0.1,
				responseMimeType: "application/json",
			},
		}),
	});

	if (!resp.ok) {
		const errText = await resp.text();
		return { applied: false, reason: `gemini-http-${resp.status}: ${errText.slice(0, 200)}` };
	}

	const data = await resp.json();
	const rawText = data?.candidates?.[0]?.content?.parts?.[0]?.text;
	if (!rawText) {
		return { applied: false, reason: "empty-gemini-response" };
	}

	let parsed;
	try {
		parsed = JSON.parse(rawText);
	} catch {
		return { applied: false, reason: "invalid-gemini-json" };
	}

	if (!parsed?.needsUpdate || !parsed.files || typeof parsed.files !== "object") {
		return { applied: false, reason: parsed?.summary || "no-rust-changes-needed" };
	}

	const backups = new Map();
	let wroteAny = false;
	try {
		for (const [relPath, newContent] of Object.entries(parsed.files)) {
			if (typeof newContent !== "string") continue;
			if (!relPath.startsWith("src/") && !relPath.startsWith("res/")) continue;
			if (relPath === "res/source.json") continue;
			const abs = path.join(crateDir, relPath);
			backups.set(abs, fs.existsSync(abs) ? fs.readFileSync(abs, "utf8") : null);
			fs.mkdirSync(path.dirname(abs), { recursive: true });
			fs.writeFileSync(abs, newContent, "utf8");
			wroteAny = true;
		}

		if (!wroteAny) {
			return { applied: false, reason: "no-applicable-files" };
		}

		// Verify compilation with cargo / aidoku if available in environment
		try {
			execFileSync("cargo", ["check", "--target", "wasm32-unknown-unknown"], {
				cwd: crateDir,
				encoding: "utf8",
				stdio: ["ignore", "pipe", "pipe"],
			});
		} catch (compileErr) {
			if (compileErr?.code !== "ENOENT") {
				// Revert Gemini patch if it breaks compilation
				for (const [abs, prev] of backups.entries()) {
					if (prev === null) fs.rmSync(abs, { force: true });
					else fs.writeFileSync(abs, prev, "utf8");
				}
				return {
					applied: false,
					reason: `compile-check-failed: ${String(compileErr.stderr || compileErr.message).slice(0, 300)}`,
				};
			}
		}

		return { applied: true, summary: parsed.summary || "Gemini auto-ported changes" };
	} catch (err) {
		for (const [abs, prev] of backups.entries()) {
			if (prev === null) fs.rmSync(abs, { force: true });
			else fs.writeFileSync(abs, prev, "utf8");
		}
		return { applied: false, reason: err.message };
	}
}

function bumpCrateVersion(crateDir, entry) {
	const sourceJsonPath = path.join(crateDir, "res", "source.json");
	const srcManifest = readJson(sourceJsonPath);
	const currentVersion = Number(srcManifest?.info?.version ?? entry.version ?? 1);
	const nextVersion = currentVersion + 1;
	srcManifest.info.version = nextVersion;
	writeJson(sourceJsonPath, srcManifest);

	entry.version = nextVersion;
	entry.baseURL =
		srcManifest.info.url ||
		(Array.isArray(srcManifest.info.urls) ? srcManifest.info.urls[0] : entry.baseURL);
	entry.packageFile = `${entry.id}-v${nextVersion}.aix`;
	entry.iconFile = `${entry.id}-v${nextVersion}.png`;
	return nextVersion;
}

async function main() {
	const args = process.argv.slice(2);
	const initCommits = args.includes("--init-commits");
	const dryRun = args.includes("--dry-run");

	if (!fs.existsSync(EXTENSIONS_FILE)) {
		throw new Error("extensions.json not found");
	}

	const registry = readJson(EXTENSIONS_FILE);
	const mihonPaths = Array.from(
		new Set(registry.sources.map((s) => s.mihonPath).filter(Boolean))
	);
	const aidokuPaths = Array.from(
		new Set(registry.sources.map((s) => s.aidokuPath).filter(Boolean))
	);

	const keiyoushiRepo = findOrCloneRepo(
		"keiyoushi",
		"keiyoushi",
		MIHON_REPO,
		"main",
		mihonPaths,
		initCommits
	);
	const aidokuRepo = findOrCloneRepo(
		"aidoku-community-main",
		"aidoku-community-main",
		AIDOKU_REPO,
		"main",
		[...aidokuPaths, "templates/mangabox", "templates/madara"],
		initCommits
	);

	let registryDirty = false;
	const summaryLines = [];

	for (const entry of registry.sources || []) {
		const crateDir = path.join(ROOT_DIR, entry.aidokuPath);
		const aidokuExists = fs.existsSync(path.join(aidokuRepo, "sources", entry.id, "Cargo.toml"));

		const latestMihon = entry.mihonPath
			? getLatestCommitForPath(keiyoushiRepo, entry.mihonPath)
			: null;
		const latestAidoku = aidokuExists
			? getLatestCommitForPath(aidokuRepo, `sources/${entry.id}`)
			: null;

		if (initCommits) {
			if (entry.lastMihonCommit !== latestMihon) {
				entry.lastMihonCommit = latestMihon;
				registryDirty = true;
			}
			if (entry.lastAidokuCommit !== latestAidoku) {
				entry.lastAidokuCommit = latestAidoku;
				registryDirty = true;
			}
			continue;
		}

		if (!entry.mihonPath) continue;

		const changedFiles = getChangedFilesBetween(
			keiyoushiRepo,
			entry.lastMihonCommit,
			latestMihon,
			entry.mihonPath
		);

		if (changedFiles.length === 0) {
			if (latestMihon && entry.lastMihonCommit !== latestMihon) {
				entry.lastMihonCommit = latestMihon;
				registryDirty = true;
			}
			if (latestAidoku && entry.lastAidokuCommit !== latestAidoku) {
				entry.lastAidokuCommit = latestAidoku;
				registryDirty = true;
			}
			continue;
		}

		console.log(`[update] ${entry.id}: Mihon changed ${changedFiles.length} file(s)`);
		const mihonDir = path.join(keiyoushiRepo, entry.mihonPath);
		const info = parseMihonExtension(mihonDir, entry.mihonPath);

		if (dryRun) {
			summaryLines.push(`${entry.id}: would sync ${changedFiles.join(", ")}`);
			continue;
		}

		// 1. Update MIHON_SOURCE.md
		fs.writeFileSync(path.join(crateDir, "MIHON_SOURCE.md"), renderMihonDoc(info), "utf8");

		// 2. Sync base URLs / mirrors
		const urlUpdated = syncBaseUrls(crateDir, info);

		// 3. Auto-port Kotlin changes via Gemini if any .kt files changed and API key is set
		let aiResult = { applied: false, reason: "no-kt-diff" };
		const hasKtChange = changedFiles.some((f) => f.endsWith(".kt"));
		if (hasKtChange) {
			const diffText = runGit(
				["diff", entry.lastMihonCommit, latestMihon, "--", entry.mihonPath],
				keiyoushiRepo
			);
			aiResult = await applyGeminiPort(entry, crateDir, diffText);
			console.log(`  [ai] ${entry.id}: applied=${aiResult.applied} (${aiResult.summary || aiResult.reason})`);
		}

		// Bump package version if Rust/res code changed so Aidoku offers the update
		if (urlUpdated || aiResult.applied) {
			const newVer = bumpCrateVersion(crateDir, entry);
			summaryLines.push(
				`${entry.id} -> v${newVer} (${aiResult.applied ? aiResult.summary : "baseURL synced"})`
			);
		} else {
			summaryLines.push(
				`${entry.id}: updated MIHON_SOURCE.md only (${aiResult.reason})`
			);
		}

		entry.lastMihonCommit = latestMihon;
		if (latestAidoku) entry.lastAidokuCommit = latestAidoku;
		registryDirty = true;
	}

	if (registryDirty && !dryRun) {
		writeJson(EXTENSIONS_FILE, registry);
		console.log("Updated extensions.json");
	}

	if (summaryLines.length > 0) {
		console.log("Sync summary:\n - " + summaryLines.join("\n - "));
	} else {
		console.log(initCommits ? "Initialized upstream commit SHAs." : "All extensions are up to date.");
	}
}

main().catch((err) => {
	console.error(err);
	process.exit(1);
});
