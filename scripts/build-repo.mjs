#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import zlib from "node:zlib";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT_DIR = path.resolve(__dirname, "..");
const PUBLIC_DIR = path.join(ROOT_DIR, "public");
const SOURCES_DIR = path.join(ROOT_DIR, "sources");
const EXTENSIONS_FILE = path.join(ROOT_DIR, "extensions.json");

const PNG_MAGIC = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
const WASM_MAGIC = Buffer.from([0x00, 0x61, 0x73, 0x6d]);

/**
 * Minimal cross-platform ZIP reader using Node built-in zlib.
 * Reads Central Directory entries and extracts requested entry buffers.
 */
function readZipEntries(buffer) {
	// Locate End of Central Directory (EOCD: 0x06054b50) scanning backwards
	const minOffset = Math.max(0, buffer.length - 65557);
	let eocdOffset = -1;
	for (let i = buffer.length - 22; i >= minOffset; i--) {
		if (
			buffer[i] === 0x50 &&
			buffer[i + 1] === 0x4b &&
			buffer[i + 2] === 0x05 &&
			buffer[i + 3] === 0x06
		) {
			eocdOffset = i;
			break;
		}
	}
	if (eocdOffset === -1) {
		throw new Error("Invalid ZIP archive: EOCD signature not found");
	}

	const totalEntries = buffer.readUInt16LE(eocdOffset + 10);
	const cdOffset = buffer.readUInt32LE(eocdOffset + 16);
	const entries = new Map();

	let ptr = cdOffset;
	for (let idx = 0; idx < totalEntries; idx++) {
		const sig = buffer.readUInt32LE(ptr);
		if (sig !== 0x02014b50) {
			throw new Error(`Invalid Central Directory header at offset ${ptr}`);
		}
		const compressionMethod = buffer.readUInt16LE(ptr + 10);
		const compressedSize = buffer.readUInt32LE(ptr + 20);
		const uncompressedSize = buffer.readUInt32LE(ptr + 24);
		const nameLen = buffer.readUInt16LE(ptr + 28);
		const extraLen = buffer.readUInt16LE(ptr + 30);
		const commentLen = buffer.readUInt16LE(ptr + 32);
		const localHeaderOffset = buffer.readUInt32LE(ptr + 42);
		const name = buffer.subarray(ptr + 46, ptr + 46 + nameLen).toString("utf8");
		ptr += 46 + nameLen + extraLen + commentLen;

		entries.set(name, {
			name,
			compressionMethod,
			compressedSize,
			uncompressedSize,
			localHeaderOffset,
			readBuffer() {
				const lhSig = buffer.readUInt32LE(localHeaderOffset);
				if (lhSig !== 0x04034b50) {
					throw new Error(`Invalid Local File Header for ${name}`);
				}
				const lhNameLen = buffer.readUInt16LE(localHeaderOffset + 26);
				const lhExtraLen = buffer.readUInt16LE(localHeaderOffset + 28);
				const dataStart = localHeaderOffset + 30 + lhNameLen + lhExtraLen;
				const slice = buffer.subarray(dataStart, dataStart + compressedSize);
				if (compressionMethod === 0) {
					return Buffer.from(slice);
				}
				if (compressionMethod === 8) {
					return zlib.inflateRawSync(slice);
				}
				throw new Error(`Unsupported ZIP compression method ${compressionMethod} in ${name}`);
			},
		});
	}

	return entries;
}

function readJson(filePath) {
	return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function parseAixPackage(aixPath) {
	const buf = fs.readFileSync(aixPath);
	const entries = readZipEntries(buf);

	const sourceEntry = entries.get("Payload/source.json");
	if (!sourceEntry) {
		throw new Error(`${path.basename(aixPath)}: missing Payload/source.json`);
	}
	const wasmEntry = entries.get("Payload/main.wasm");
	if (!wasmEntry) {
		throw new Error(`${path.basename(aixPath)}: missing Payload/main.wasm`);
	}

	const wasmBuf = wasmEntry.readBuffer();
	if (wasmBuf.length < 8 || !wasmBuf.subarray(0, 4).equals(WASM_MAGIC)) {
		throw new Error(`${path.basename(aixPath)}: Payload/main.wasm is not a valid WebAssembly binary`);
	}

	const sourceMeta = JSON.parse(sourceEntry.readBuffer().toString("utf8"));
	const iconEntry = entries.get("Payload/icon.png");

	return {
		entries,
		sourceMeta,
		wasmSize: wasmBuf.length,
		iconBuffer: iconEntry ? iconEntry.readBuffer() : null,
	};
}

function findLatestAixForId(id) {
	const pubSourcesDir = path.join(PUBLIC_DIR, "sources");
	if (!fs.existsSync(pubSourcesDir)) return null;
	const prefix = `${id}-v`;
	const matches = fs
		.readdirSync(pubSourcesDir)
		.filter((f) => f.startsWith(prefix) && f.endsWith(".aix"))
		.map((f) => {
			const m = f.slice(prefix.length, -4);
			return { file: f, version: Number.parseInt(m, 10) || 0 };
		})
		.sort((a, b) => b.version - a.version);
	return matches.length > 0 ? matches[0].file : null;
}

function buildIndex(baseUrl = "") {
	const registry = fs.existsSync(EXTENSIONS_FILE)
		? readJson(EXTENSIONS_FILE)
		: { name: "MHAID Sources", sources: [] };
	const repoName = registry.name || "MHAID Sources";

	const pubSourcesDir = path.join(PUBLIC_DIR, "sources");
	const pubIconsDir = path.join(PUBLIC_DIR, "icons");
	fs.mkdirSync(pubSourcesDir, { recursive: true });
	fs.mkdirSync(pubIconsDir, { recursive: true });

	const normalizedBase = baseUrl ? baseUrl.replace(/\/+$/, "") + "/" : "";
	const builtSources = [];

	const sourceDirs = fs.existsSync(SOURCES_DIR)
		? fs
				.readdirSync(SOURCES_DIR, { withFileTypes: true })
				.filter((d) => d.isDirectory())
				.map((d) => d.name)
				.sort()
		: [];

	for (const dirName of sourceDirs) {
		const sourceJsonPath = path.join(SOURCES_DIR, dirName, "res", "source.json");
		if (!fs.existsSync(sourceJsonPath)) continue;
		const srcManifest = readJson(sourceJsonPath);
		const info = srcManifest.info || {};
		const id = info.id || dirName;

		// Check if source crate has a freshly built package.aix or an existing public/sources/<id>-v<ver>.aix
		const cratePackage = path.join(SOURCES_DIR, dirName, "package.aix");
		let aixFileName = `${id}-v${info.version}.aix`;
		let aixPath = path.join(pubSourcesDir, aixFileName);

		if (fs.existsSync(cratePackage)) {
			fs.copyFileSync(cratePackage, aixPath);
		} else if (!fs.existsSync(aixPath)) {
			const fallbackAix = findLatestAixForId(id);
			if (!fallbackAix) {
				console.warn(`[warn] Skipping ${id} in index: no compiled .aix package found in public/sources/`);
				continue;
			}
			aixFileName = fallbackAix;
			aixPath = path.join(pubSourcesDir, aixFileName);
		}

		const parsed = parseAixPackage(aixPath);
		const pkgInfo = parsed.sourceMeta.info || info;
		const pkgVersion = pkgInfo.version ?? info.version;
		const iconFileName = `${pkgInfo.id}-v${pkgVersion}.png`;
		const iconOutPath = path.join(pubIconsDir, iconFileName);

		if (!fs.existsSync(iconOutPath)) {
			const localIcon = path.join(SOURCES_DIR, dirName, "res", "icon.png");
			if (parsed.iconBuffer) {
				fs.writeFileSync(iconOutPath, parsed.iconBuffer);
			} else if (fs.existsSync(localIcon)) {
				fs.copyFileSync(localIcon, iconOutPath);
			}
		}

		const entry = {
			id: pkgInfo.id,
			name: pkgInfo.name,
			version: pkgVersion,
			iconURL: `${normalizedBase}icons/${iconFileName}`,
			downloadURL: `${normalizedBase}sources/${aixFileName}`,
			languages: pkgInfo.languages || ["en"],
			contentRating: pkgInfo.contentRating ?? 0,
			baseURL: pkgInfo.url || "",
		};
		if (pkgInfo.minAppVersion) {
			entry.minAppVersion = pkgInfo.minAppVersion;
		}
		if (Array.isArray(pkgInfo.altNames) && pkgInfo.altNames.length > 0) {
			entry.altNames = pkgInfo.altNames;
		}

		builtSources.push(entry);
	}

	const indexObj = {
		name: repoName,
		sources: builtSources,
	};

	fs.writeFileSync(
		path.join(PUBLIC_DIR, "index.min.json"),
		JSON.stringify(indexObj),
		"utf8"
	);
	fs.writeFileSync(
		path.join(PUBLIC_DIR, "index.json"),
		JSON.stringify(indexObj, null, "\t") + "\n",
		"utf8"
	);
	fs.writeFileSync(path.join(PUBLIC_DIR, ".nojekyll"), "", "utf8");

	console.log(`Built index.min.json and index.json with ${builtSources.length} source(s).`);
	return indexObj;
}

function verifyRepo() {
	const errors = [];
	const minPath = path.join(PUBLIC_DIR, "index.min.json");
	const prettyPath = path.join(PUBLIC_DIR, "index.json");

	for (const p of [minPath, prettyPath]) {
		if (!fs.existsSync(p)) {
			errors.push(`Missing required manifest: ${path.relative(ROOT_DIR, p)}`);
		}
	}

	if (errors.length > 0) {
		throw new Error(errors.join("\n"));
	}

	const indexMin = readJson(minPath);
	const indexPretty = readJson(prettyPath);
	if (!Array.isArray(indexMin.sources) || indexMin.sources.length === 0) {
		errors.push("public/index.min.json has no sources");
	}
	if (JSON.stringify(indexMin) !== JSON.stringify(indexPretty)) {
		errors.push("public/index.min.json and public/index.json differ in content");
	}

	// Verify all ready entries in extensions.json are present in index.min.json
	if (fs.existsSync(EXTENSIONS_FILE)) {
		const registry = readJson(EXTENSIONS_FILE);
		for (const regSrc of registry.sources || []) {
			if (regSrc.status === "ready") {
				const found = (indexMin.sources || []).find((s) => s.id === regSrc.id);
				if (!found) {
					errors.push(`Ready source "${regSrc.id}" from extensions.json is missing in public/index.min.json`);
				}
			}
		}
	}

	for (const entry of indexMin.sources || []) {
		for (const field of ["id", "name", "version", "iconURL", "downloadURL", "languages", "baseURL"]) {
			if (entry[field] === undefined || entry[field] === null) {
				errors.push(`Source "${entry.id || "?"}" missing field "${field}"`);
			}
		}

		const relAix = String(entry.downloadURL || "").replace(/^https?:\/\/[^/]+\//, "");
		const aixPath = path.join(PUBLIC_DIR, relAix);
		if (!fs.existsSync(aixPath)) {
			errors.push(`Missing .aix archive for ${entry.id}: ${relAix}`);
		} else {
			try {
				const parsed = parseAixPackage(aixPath);
				const info = parsed.sourceMeta.info || {};
				if (info.id !== entry.id) {
					errors.push(`${relAix}: Payload/source.json id "${info.id}" != "${entry.id}"`);
				}
				if (info.version !== entry.version) {
					errors.push(`${relAix}: Payload/source.json version ${info.version} != ${entry.version}`);
				}
				console.log(
					`[ok] ${entry.id} v${entry.version} | wasm=${parsed.wasmSize}B | pkg=${relAix}`
				);
			} catch (err) {
				errors.push(`Invalid .aix archive (${relAix}): ${err.message}`);
			}
		}

		const relIcon = String(entry.iconURL || "").replace(/^https?:\/\/[^/]+\//, "");
		const iconPath = path.join(PUBLIC_DIR, relIcon);
		if (!fs.existsSync(iconPath)) {
			errors.push(`Missing icon for ${entry.id}: ${relIcon}`);
		} else {
			const header = fs.readFileSync(iconPath).subarray(0, 8);
			if (!header.equals(PNG_MAGIC)) {
				errors.push(`Invalid PNG signature in icon ${relIcon}`);
			}
		}
	}

	if (errors.length > 0) {
		console.error("Verification failed:\n - " + errors.join("\n - "));
		process.exit(1);
	}
	console.log(`Verification passed: ${indexMin.sources.length} source(s) valid.`);
}

function main() {
	const args = process.argv.slice(2);
	const verifyOnly = args.includes("--verify");
	let baseUrl = "";
	const baseIdx = args.indexOf("--base-url");
	if (baseIdx !== -1 && args[baseIdx + 1]) {
		baseUrl = args[baseIdx + 1];
	}

	if (!verifyOnly) {
		buildIndex(baseUrl);
	}
	verifyRepo();
}

main();
