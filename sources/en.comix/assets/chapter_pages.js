(() => {
    "use strict";

    window.__comixResult = null;
    window.__comixError = null;
    window.__comixWaf = false;

    try { localStorage.setItem("__pu1", "[" + Date.now() + "]"); } catch (e) {}

    const startedAt = Date.now();
    const FIRST_PAGE_TIMEOUT_MS = 20000;
    const WALK_TIMEOUT_MS = 60000;
    const SLOT_TIMEOUT_MS = 5000;
    const SETTLE_MS = 45;
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
            style.textContent = "html, body { width: 390px !important; min-width: 390px !important; } .rpage-page { min-height: 200px !important; }";
            (document.head || document.documentElement).appendChild(style);
        }
    };

    // Temporarily switch reader state to vertical strip ("ttb") with preload "all"
    const savedReader = {};
    const addedKeys = [];
    for (let i = 0; i < localStorage.length; i++) {
        const key = localStorage.key(i);
        try {
            const raw = localStorage.getItem(key);
            const stored = JSON.parse(raw);
            if (stored && stored.state && "readingDirection" in stored.state) {
                savedReader[key] = raw;
                stored.state.readingDirection = "ttb";
                stored.state.preload = "all";
                localStorage.setItem(key, JSON.stringify(stored));
            }
        } catch (e) {}
    }
    for (const defaultKey of ["reader.default.v3", "reader.webtoon.v3"]) {
        if (!(defaultKey in savedReader) && localStorage.getItem(defaultKey) === null) {
            try {
                addedKeys.push(defaultKey);
                localStorage.setItem(defaultKey, JSON.stringify({
                    state: { readingDirection: "ttb", preload: "all", pageLayout: "single" },
                    version: 0
                }));
            } catch (e) {}
        }
    }
    const restoreReader = () => {
        Object.entries(savedReader).forEach(([key, raw]) => {
            try { localStorage.setItem(key, raw); } catch (e) {}
        });
        Object.keys(savedReader).forEach((key) => delete savedReader[key]);
        for (const k of addedKeys) {
            try { localStorage.removeItem(k); } catch (e) {}
        }
        addedKeys.length = 0;
    };

    let container = null;
    let slots = [];
    let total = 0;
    const shapeOf = (element) => element.tagName + "." + (element.getAttribute("class") || "");
    const site = location.hostname.replace(/^www\./, "");
    const isForeignImage = (img) => {
        try {
            const host = new URL(img.getAttribute("src") || "", location.href).hostname;
            return host !== site && !host.endsWith("." + site);
        } catch (e) {
            return false;
        }
    };
    const scrolls = (element) => element.scrollHeight > element.clientHeight &&
        (element === document.scrollingElement || /(auto|scroll)/.test(getComputedStyle(element).overflowY));
    const insideScroller = (element) => {
        for (let parent = element.parentElement; parent; parent = parent.parentElement) if (scrolls(parent)) return true;
        return false;
    };

    const findReader = () => {
        if (!document.body) return null;
        const totals = new Set();
        for (const element of document.body.querySelectorAll("*")) {
            if (element.firstElementChild) continue;
            const match = /^\s*\d+\s*\/\s*(\d+)\s*$/.exec(element.textContent || "");
            if (match) totals.add(parseInt(match[1], 10));
        }
        const matches = [];
        for (const element of document.body.querySelectorAll("*")) {
            const withImage = Array.prototype.find.call(element.children,
                (child) => Array.prototype.some.call(child.querySelectorAll("img"), isForeignImage));
            if (!withImage) continue;
            const shape = shapeOf(withImage);
            const count = Array.prototype.filter.call(element.children, (child) => shapeOf(child) === shape).length;
            if (totals.size && !totals.has(count)) continue;
            if (insideScroller(element)) matches.push({ element, shape });
        }
        let best = null;
        for (const match of matches) {
            if (matches.some((other) => other !== match && match.element.contains(other.element))) continue;
            if (!best || match.element.offsetHeight > best.element.offsetHeight) best = match;
        }
        if (best) {
            return {
                element: best.element,
                slots: Array.prototype.filter.call(best.element.children, (child) => shapeOf(child) === best.shape),
            };
        }
        // Fallback for 0x0 WKWebView geometry before scroller heights settle
        const rpageSlots = Array.from(document.querySelectorAll(".rpage-page[data-page]"));
        if (rpageSlots.length > 0) {
            return {
                element: rpageSlots[0].parentElement || document.body,
                slots: rpageSlots,
            };
        }
        return null;
    };

    let candidate = null;
    const captured = new Map();
    const missed = new Set();

    const slotIndex = (img) => {
        const dp = img.closest ? img.closest("[data-page]") : null;
        if (dp) {
            const p = parseInt(dp.getAttribute("data-page") || "", 10) - 1;
            if (Number.isFinite(p) && p >= 0 && p < slots.length) return p;
        }
        let slot = img;
        while (slot.parentElement && slot.parentElement !== container) slot = slot.parentElement;
        return slots.indexOf(slot);
    };

    const record = (img) => {
        if (!container) return;
        let url;
        try { url = new URL(img.getAttribute("src") || "", location.href); } catch (e) { return; }
        if (!/^https?:$/.test(url.protocol)) return;
        if (!isForeignImage(img)) return;
        const index = slotIndex(img);
        if (index < 0 || captured.get(index) === url.href) return;
        captured.set(index, url.href);
        missed.delete(index);
    };

    const attachReader = () => {
        if (container && container.isConnected) {
            if (slots.length) return true;
        }
        const reader = findReader();
        if (!reader || !reader.slots || !reader.slots.length) {
            candidate = reader;
            return false;
        }
        const hasExplicitPages = reader.slots[0].hasAttribute && reader.slots[0].hasAttribute("data-page");
        const settled = hasExplicitPages || (candidate && reader.element === candidate.element &&
            reader.slots.length === candidate.slots.length);
        candidate = reader;
        if (!settled) return false;
        ({ element: container, slots } = reader);
        total = slots.length;
        container.querySelectorAll("img[src]").forEach(record);
        return true;
    };

    const recordImage = (node) => {
        if (!node) return;
        if (node.tagName === "IMG") record(node);
        else if (node.querySelectorAll) node.querySelectorAll("img[src]").forEach(record);
    };
    const observer = new MutationObserver((mutations) => {
        for (const mutation of mutations) {
            if (mutation.type === "attributes") recordImage(mutation.target);
            else mutation.addedNodes.forEach(recordImage);
        }
    });
    observer.observe(document, { subtree: true, childList: true, attributes: true, attributeFilter: ["src"] });

    const waitForReader = async () => {
        while (Date.now() < startedAt + FIRST_PAGE_TIMEOUT_MS) {
            if (isWafPage()) {
                window.__comixWaf = true;
                return false;
            }
            if (attachReader() && total > 0) return true;
            await sleep(40);
        }
        throw new Error("The reader did not load any pages");
    };

    const waited = new Map();
    const walk = async () => {
        ensureViewportStyle();
        const ready = await waitForReader();
        if (!ready) {
            restoreReader();
            observer.disconnect();
            return null;
        }

        const done = (i) => captured.has(i) || missed.has(i);
        while (Date.now() < startedAt + WALK_TIMEOUT_MS) {
            if (isWafPage()) {
                window.__comixWaf = true;
                restoreReader();
                observer.disconnect();
                return null;
            }
            let target = -1;
            for (let i = 0; i < total; i++) {
                if (!done(i)) { target = i; break; }
            }
            if (target < 0) break;

            const time = (waited.get(target) || 0) + SETTLE_MS;
            waited.set(target, time);
            if (time > SLOT_TIMEOUT_MS) {
                missed.add(target);
                continue;
            }

            if (attachReader() && slots[target]) {
                container.querySelectorAll("img[src]").forEach(record);
                if (captured.has(target)) continue;
                try { slots[target].scrollIntoView({ block: "center" }); } catch (e) {}
                try {
                    window.dispatchEvent(new Event("scroll"));
                    document.dispatchEvent(new Event("scroll"));
                } catch (e) {}
                if (time >= SETTLE_MS * 3) {
                    const seg = document.querySelector(`button.rpage-progress__seg[aria-label="Go to page ${target + 1}"]`);
                    if (seg) {
                        try { seg.click(); } catch (e) {}
                    }
                }
            }
            await sleep(SETTLE_MS);
        }

        restoreReader();
        observer.disconnect();
        const pages = Array.from(captured.entries())
            .sort((a, b) => a[0] - b[0])
            .map(([index, url]) => ({ index, url }));
        return { total, pages };
    };

    (async () => {
        try {
            const result = await walk();
            if (!window.__comixWaf && result) {
                window.__comixResult = result;
            }
        } catch (e) {
            restoreReader();
            observer.disconnect();
            window.__comixError = String((e && e.message) || e);
        }
    })();

    return "started";
})();
