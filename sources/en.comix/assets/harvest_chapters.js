(() => {
    "use strict";

    window.__comixResult = null;
    window.__comixError = null;
    window.__comixWaf = false;

    try { localStorage.setItem("__pu1", "[" + Date.now() + "]"); } catch (e) {}

    const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

    const isWafPage = () => {
        const path = location.pathname || "";
        if (path === "/@waf/challenge" || path.startsWith("/@waf/")) return true;
        const title = (document.title || "").trim();
        if (title === "Security check" || title === "Just a moment...") return true;
        if (document.querySelector('iframe[src*="/@waf/challenge"]')) return true;
        return false;
    };

    const ensureViewportStyle = () => {
        if (!document.getElementById("__comix_wv_fix")) {
            const style = document.createElement("style");
            style.id = "__comix_wv_fix";
            style.textContent = "html, body { width: 390px !important; min-width: 390px !important; }";
            (document.head || document.documentElement).appendChild(style);
        }
    };

    const waitFor = async (check, { timeoutMs = 20000, intervalMs = 30 } = {}) => {
        const deadline = Date.now() + timeoutMs;
        for (;;) {
            if (isWafPage()) {
                window.__comixWaf = true;
                return "__WAF__";
            }
            let value = null;
            try { value = check(); } catch (e) { value = null; }
            if (value) return value;
            if (Date.now() >= deadline) return null;
            await sleep(Math.min(intervalMs, Math.max(1, deadline - Date.now())));
        }
    };

    const MANGA_PATH = /^\/title\/([a-z0-9]+)(?:-|\/|$)/i;
    const CHAPTER_PATH = /^\/title\/([a-z0-9]+)(?:-[^/]+)?\/(\d+)-(?:chapter|ch|volume|vol)-(\d+(?:\.\d+)?)/i;
    const pathOf = (href) => {
        if (!href) return "";
        if (/^https?:\/\//i.test(href)) {
            try { return new URL(href).pathname; } catch (e) { return href; }
        }
        return href.split(/[?#]/)[0];
    };
    const leavesPage = (el) => {
        const path = pathOf(el.getAttribute("href") || "");
        return path !== "" && path !== location.pathname;
    };
    const parseMangaPath = (path) => {
        const match = MANGA_PATH.exec(path || "");
        return match ? { hid: match[1] } : null;
    };
    const parseChapterPath = (path) => {
        const match = CHAPTER_PATH.exec(path || "");
        return match ? { hid: match[1], id: Number(match[2]), number: parseFloat(match[3]) } : null;
    };
    const textOf = (el) => (el && el.textContent ? el.textContent : "").replace(/\s+/g, " ").trim();
    const isChapterRow = (anchor) => !!anchor.closest("li");

    const rowRoot = (anchor) => {
        let el = anchor;
        for (let depth = 0; depth < 4 && el.parentElement; depth++) {
            const parent = el.parentElement;
            const links = Array.from(parent.querySelectorAll("a[href]"))
                .filter((x) => parseChapterPath(pathOf(x.getAttribute("href") || "")));
            if (links.length !== 1) break;
            el = parent;
        }
        return el;
    };

    const nameOf = (anchor) => {
        const titleEl = anchor.querySelector(".mchap-row__title");
        if (titleEl) {
            const t = textOf(titleEl);
            if (t) return t;
        }
        const parts = Array.from(anchor.children)
            .slice(1)
            .filter((el) => !el.classList || !el.classList.contains("mchap-row__vol"))
            .map(textOf)
            .filter(Boolean);
        return parts.join(" ").trim();
    };

    const volumeOf = (anchor) => {
        const volEl = anchor.querySelector(".mchap-row__vol");
        if (!volEl) return null;
        const m = /(\d+(?:\.\d+)?)/.exec(textOf(volEl));
        return m ? parseFloat(m[1]) : null;
    };

    const groupOf = (row) => {
        if (!row) return null;
        const group = row.querySelector('a[href^="/groups/"]');
        if (group && textOf(group)) {
            const id = /^\/groups\/(\d+)/.exec(group.getAttribute("href") || "");
            const isOfficial = group.classList && group.classList.contains("is-official");
            return { id: id ? Number(id[1]) : null, name: textOf(group), isOfficial };
        }
        const officialSpan = row.querySelector("span.mchap-row__group.is-official");
        if (officialSpan) {
            return { id: 10702, name: textOf(officialSpan) || "Official", isOfficial: true };
        }
        const uploader = row.querySelector('a[href^="/u/"]');
        return uploader && textOf(uploader) ? { id: null, name: textOf(uploader), isOfficial: false } : null;
    };

    const votesOf = (row) => {
        if (!row) return 0;
        const likes = row.querySelector(".mchap-row__likes");
        if (likes) {
            const raw = textOf(likes).replace(/,/g, "");
            const m = /^(\d+(?:\.\d+)?)(k?)$/i.exec(raw);
            if (m) return Math.round(parseFloat(m[1]) * (m[2] ? 1000 : 1));
        }
        for (const span of row.querySelectorAll("span")) {
            const match = span.querySelector("svg") && /^(\d+(?:\.\d+)?)(k?)$/i.exec(textOf(span).replace(/,/g, ""));
            if (match) return Math.round(parseFloat(match[1]) * (match[2] ? 1000 : 1));
        }
        return 0;
    };

    const timeTextOf = (row, anchor) => {
        if (row) {
            const timeEl = row.querySelector(".mchap-row__time");
            if (timeEl && textOf(timeEl)) return textOf(timeEl);
            return textOf(row);
        }
        return textOf(anchor);
    };

    const findMoreControl = (scope) => {
        const inOrder = Array.from(document.querySelectorAll("a[href], button, [role='button']"));
        const orderOf = new Map(inOrder.map((el, i) => [el, i]));
        let lastAnchorIdx = -1;
        for (const el of inOrder) {
            const href = el.getAttribute("href");
            if (href && parseChapterPath(pathOf(href)) && isChapterRow(el)) lastAnchorIdx = orderOf.get(el);
        }
        const currents = inOrder.filter((el) => (orderOf.get(el) ?? -1) > lastAnchorIdx &&
            (el.getAttribute("aria-current") === "page" || /\bis-active\b/.test(String(el.className || ""))) &&
            /^\d{1,4}$/.test((textOf(el) || "").trim()));
        if (currents.length) {
            const cur = currents[0];
            const curNum = parseInt((textOf(cur) || "").trim(), 10);
            let box = cur.parentElement;
            for (let d = 0; d < 3 && box && box.tagName !== "NAV"; d++) box = box.parentElement;
            const boxEls = Array.from((box || cur.parentElement || document)
                .querySelectorAll("button, a[href], [role='button']"));
            let nextBtn = null;
            let numbered = null;
            for (const el of boxEls) {
                if (el.disabled || el.getAttribute("aria-disabled") === "true" || leavesPage(el)) continue;
                const label = (textOf(el) || el.getAttribute("aria-label") || "").trim();
                if (/next/i.test(label)) nextBtn = el;
                else if (/^\d{1,4}$/.test(label) && parseInt(label, 10) === curNum + 1) numbered = el;
            }
            return nextBtn || numbered || null;
        }
        const controls = Array.from((scope || document).querySelectorAll("button, a[href], [role='button']"));
        const scored = [];
        for (const el of controls) {
            if (el.disabled) continue;
            if (el.getAttribute("aria-current") === "page") continue;
            if (el.getAttribute("aria-disabled") === "true") continue;
            if (leavesPage(el)) continue;
            if (/desc/i.test(String(el.className || ""))) continue;
            const label = (textOf(el) || el.getAttribute("aria-label") || "").trim();
            if (!label) continue;
            const afterList = lastAnchorIdx < 0 || (orderOf.get(el) ?? -1) > lastAnchorIdx;
            let score = -1;
            if (/next/i.test(label)) score = afterList ? 5 : 2;
            else if (/load\s*more|show\s*more|view\s*more|more\s+chapters/i.test(label)) score = afterList ? 4 : -1;
            else if (/^\d{1,4}$/.test(label)) score = 1;
            if (score >= 0) scored.push({ el, score });
        }
        scored.sort((a, b) => b.score - a.score);
        return scored.length ? scored[0].el : null;
    };

    const reportedTotal = () => {
        if (!document.body) return null;
        const m = /Showing\s+\d+\s+to\s+\d+\s+of\s+([\d,]+)\s+items/i.exec(textOf(document.body));
        return m ? parseInt(m[1].replace(/,/g, ""), 10) : null;
    };

    const isChaptersEmpty = () => !!document.querySelector(".mpage__chapters .uempty");

    const OPTIONS = { timeoutMs: 20000, settleMs: 1800, pollMs: 30, maxRounds: 60 };

    const harvest = async () => {
        if (isWafPage()) {
            window.__comixWaf = true;
            return null;
        }
        ensureViewportStyle();

        const here = parseMangaPath(location.pathname || pathOf(location.href || ""));
        const hid = here ? here.hid : null;

        const byId = new Map();
        const absorb = () => {
            let added = 0;
            for (const anchor of Array.from(document.querySelectorAll("a[href]"))) {
                const path = pathOf(anchor.getAttribute("href") || "");
                const chapter = parseChapterPath(path);
                if (!chapter || !isChapterRow(anchor)) continue;
                if (hid && chapter.hid !== hid) continue;
                if (byId.has(chapter.id)) continue;
                const row = rowRoot(anchor);
                byId.set(chapter.id, {
                    id: chapter.id,
                    number: chapter.number,
                    volume: volumeOf(anchor),
                    url: path,
                    text: timeTextOf(row, anchor),
                    name: nameOf(anchor),
                    votes: votesOf(row),
                    group: groupOf(row),
                });
                added++;
            }
            return added;
        };

        const initWait = await waitFor(
            () => (absorb(), byId.size > 0 || isChaptersEmpty()) ? true : null,
            { timeoutMs: OPTIONS.timeoutMs, intervalMs: OPTIONS.pollMs }
        );
        if (initWait === "__WAF__") return null;

        if (byId.size === 0 && isChaptersEmpty()) {
            return { page: "manga", hid, count: 0, complete: true, chapters: [] };
        }

        const sectionRoot = () => {
            const first = Array.from(document.querySelectorAll("a[href]"))
                .find((a) => {
                    const c = parseChapterPath(pathOf(a.getAttribute("href") || ""));
                    return c && (!hid || c.hid === hid) && isChapterRow(a);
                });
            if (!first) return document.body;
            const rendered = Array.from(document.querySelectorAll("a[href]"))
                .filter((a) => {
                    const c = parseChapterPath(pathOf(a.getAttribute("href") || ""));
                    return c && (!hid || c.hid === hid) && isChapterRow(a);
                }).length;
            let root = first;
            for (let depth = 0; depth < 12 && root.parentElement; depth++) {
                const parent = root.parentElement;
                const inside = Array.from(parent.querySelectorAll("a[href]"))
                    .filter((a) => {
                        const c = parseChapterPath(pathOf(a.getAttribute("href") || ""));
                        return c && (!hid || c.hid === hid) && isChapterRow(a);
                    }).length;
                if (inside >= rendered) root = parent;
                else break;
            }
            return root;
        };

        const activate = async (el) => {
            try { el.click(); } catch (e) {
                try {
                    el.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
                } catch (_) {}
            }
        };

        let rounds = 0;
        let complete = false;
        while (rounds < OPTIONS.maxRounds) {
            if (isWafPage()) {
                window.__comixWaf = true;
                return null;
            }
            let control = findMoreControl(sectionRoot());
            if (!control) control = findMoreControl(document);
            if (!control) {
                complete = byId.size > 0;
                break;
            }
            const before = byId.size;
            rounds++;
            await activate(control);
            let grew = await waitFor(
                () => (absorb(), byId.size > before) || null,
                { timeoutMs: OPTIONS.settleMs, intervalMs: OPTIONS.pollMs }
            );
            if (grew === "__WAF__") return null;
            if (!grew) {
                await activate(control);
                grew = await waitFor(
                    () => (absorb(), byId.size > before) || null,
                    { timeoutMs: 600, intervalMs: OPTIONS.pollMs }
                );
                if (grew === "__WAF__") return null;
            }
            const now = parseMangaPath(location.pathname);
            if (hid && (!now || now.hid !== hid)) {
                throw new Error("Left the title page for " + location.pathname);
            }
            if (!grew) {
                complete = byId.size > 0;
                break;
            }
        }

        const total = reportedTotal();
        if (total != null && byId.size < total) complete = false;
        const chapters = Array.from(byId.values())
            .sort((a, b) => (b.number - a.number) || (b.id - a.id));
        return {
            page: "manga",
            hid,
            count: chapters.length,
            complete,
            chapters,
        };
    };

    (async () => {
        try {
            const result = await harvest();
            if (!window.__comixWaf && result) {
                window.__comixResult = result;
            }
        } catch (e) {
            window.__comixError = String((e && e.message) || e);
        }
    })();

    return "started";
})();
