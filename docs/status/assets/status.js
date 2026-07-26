/* status-app.js - client for status-hub@3 (one tabless canvas).
   Reads status-feed@1 from #sv3-data. Zero runtime deps; works over file://.
   Selection model: one {kind, id} drives cross-highlighting in every band.
   Spec chunks load lazily from D.specDir (window.CS_SPEC) when task.sp is set. */
(function () {
  "use strict";

  /* ═══ 0 · data + indexes ═══════════════════════════════════════════════ */
  const D = JSON.parse(document.getElementById("sv3-data").textContent);
  const TASKS = D.tasks || [];
  const BY = {}; TASKS.forEach(t => { BY[t.i] = t; });
  const REV = {}; // dependents: dep id -> [task ids that depend on it]
  TASKS.forEach(t => (t.d || []).forEach(d => (REV[d] = REV[d] || []).push(t.i)));
  const MODS = D.modules || [];
  const MOD = {}; MODS.forEach(m => { MOD[m.id] = m; });
  const RELS = D.releases || [];
  const RELBYV = {}; RELS.forEach(r => { RELBYV[r.v] = r; });
  const TODAY = D.snapshot;

  const esc = s => String(s == null ? "" : s)
    .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  const pct = (n, d) => d ? Math.round(100 * n / d) : 0;
  const num = n => Number(n || 0).toLocaleString("en-US");
  const short = id => id.replace(/^TASK-/, "");
  const stLabel = s => String(s || "").replace(/_/g, " ");
  const dayMs = 864e5;
  const dnum = s => { const t = Date.parse(s); return isNaN(t) ? null : t; };
  const daysBetween = (a, b) => Math.round((dnum(b) - dnum(a)) / dayMs);

  /* ═══ 1 · derived truth ════════════════════════════════════════════════ */
  const unfinished = t => t.b !== "done";
  const unmetDeps = t => (t.d || []).filter(d => BY[d] && BY[d].b !== "done");
  const BLOCKED = TASKS.filter(t => unfinished(t) && unmetDeps(t).length);
  const BLOCKEDSET = new Set(BLOCKED.map(t => t.i));
  const FRONTIER = TASKS.filter(t => (t.b === "draft" || t.b === "active") && !unmetDeps(t).length);
  const FRONTSET = new Set(FRONTIER.map(t => t.i));
  const sDone = TASKS.filter(t => t.s === "done").length;
  const retired = TASKS.filter(t => t.b === "done" && t.s !== "done").length;
  const nActive = TASKS.filter(t => t.b === "active").length;
  const nHold = TASKS.filter(t => t.b === "hold").length;
  const nDraft = TASKS.filter(t => t.b === "draft").length;
  const STALE = TASKS.filter(t => t.b === "draft" && t.cr && daysBetween(t.cr, TODAY) > 45);

  /* critical path: longest chain of unfinished work along dep -> dependent edges */
  const wOf = t => (t.e && t.e > 0 ? t.e : 4);
  const cpMemo = {}, cpState = {};
  function cpLongest(id) {
    if (cpMemo[id]) return cpMemo[id];
    const t = BY[id];
    if (!t || !unfinished(t)) return { w: 0, next: null };
    if (cpState[id] === 1) return { w: 0, next: null }; // cycle guard
    cpState[id] = 1;
    let best = { w: 0, next: null };
    for (const c of (REV[id] || [])) {
      if (!BY[c] || !unfinished(BY[c])) continue;
      const r = cpLongest(c);
      if (r.w + wOf(BY[c]) > best.w) best = { w: r.w + wOf(BY[c]), next: c };
    }
    cpState[id] = 2;
    return (cpMemo[id] = { w: best.w + wOf(t), next: best.next });
  }
  let cpStart = null, cpBest = 0;
  for (const t of TASKS) {
    if (!unfinished(t)) continue;
    const r = cpLongest(t.i);
    if (r.w > cpBest) { cpBest = r.w; cpStart = t.i; }
  }
  const CRIT = [];
  for (let id = cpStart; id; id = cpMemo[id] && cpMemo[id].next) {
    CRIT.push(id); if (CRIT.length > 60) break;
  }
  const CRITSET = new Set(CRIT);
  const CRITNEXT = {}; CRIT.forEach((id, i) => { if (CRIT[i + 1]) CRITNEXT[id] = CRIT[i + 1]; });
  const cpHours = Math.round(CRIT.reduce((s, id) => s + wOf(BY[id]), 0));

  /* traceability: task -> releases / commits that cite it */
  const RELOF = {}, COMOF = {};
  function citeRel(v, id) { (RELOF[id] = RELOF[id] || []).push(v); }
  RELS.forEach(r => {
    const seen = new Set();
    ["features", "fixes", "improvements"].forEach(k => (r.notes[k] || []).forEach(it =>
      (it.ids || []).forEach(id => { if (!seen.has(id)) { seen.add(id); citeRel(r.v, id); } })));
    (r.cov.commits || []).forEach(c => (c.ids || []).forEach(id => {
      (COMOF[id] = COMOF[id] || []).push(c);
      if (!seen.has(id)) { seen.add(id); citeRel(r.v, id); }
    }));
  });
  (function citeUnreleased() {
    const un = D.unreleased || {};
    const seen = new Set();
    ["features", "fixes", "improvements"].forEach(k =>
      (((un.notes || {})[k]) || []).forEach(it => (it.ids || []).forEach(id => {
        if (!seen.has(id)) { seen.add(id); citeRel("unreleased", id); }
      })));
    ((un.cov || {}).commits || []).forEach(c => (c.ids || []).forEach(id => {
      (COMOF[id] = COMOF[id] || []).push(c);
      if (!seen.has(id)) { seen.add(id); citeRel("unreleased", id); }
    }));
  })();

  const sumCov = list => list.reduce((a, r) => ({
    linked: a.linked + (r.cov.linked || 0), exempt: a.exempt + (r.cov.exempt || 0),
    unlinked: a.unlinked + (r.cov.unlinked || 0),
  }), { linked: 0, exempt: 0, unlinked: 0 });
  const LEGACY = RELS.filter(r => r.lg);
  const CURRELS = RELS.filter(r => !r.lg);
  const covCur = sumCov(CURRELS.concat([{ cov: (D.unreleased || {}).cov || {} }]));
  const covLeg = sumCov(LEGACY);
  const covPct = pct(covCur.linked, covCur.linked + covCur.unlinked);
  const relLabel = v => v === "unreleased" ? "next" : v === "first-epoch" ? "epoch 1" : "v" + v;

  /* GitHub deep links (derived from origin at build time) */
  const GH = D.repoUrl || "";
  const TAGSET = new Set(D.tags || []);
  const ghc = h => GH ? GH + "/commit/" + h : null;
  const relHref = r => !GH ? null
    : TAGSET.has("v" + r.v) ? GH + "/releases/tag/v" + r.v
    : r.rc ? ghc(r.rc) : null;
  const extA = (href, cls, inner, title) => href
    ? '<a class="' + cls + '" target="_blank" rel="noopener" href="' + esc(href) + '"' +
      (title ? ' title="' + esc(title) + '"' : "") + ">" + inner + "</a>"
    : '<span class="' + cls + '">' + inner + "</span>";
  const hasNotes = r => r.notes && (r.notes.features.length + r.notes.fixes.length + r.notes.improvements.length) > 0;
  const staleDays = Math.floor((Date.now() - (dnum(TODAY) || Date.now())) / dayMs);

  /* per-module blocked counts */
  const modBlocked = {};
  BLOCKED.forEach(t => { modBlocked[t.m] = (modBlocked[t.m] || 0) + 1; });

  /* ═══ 2 · state ════════════════════════════════════════════════════════ */
  const S = { sel: null, flow: "focus", flowDone: true, lastSetHash: null };

  /* ═══ 3 · skeleton ═════════════════════════════════════════════════════ */
  const app = document.getElementById("app");
  const ICONS = {
    search: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6"><circle cx="7" cy="7" r="4.6"/><path d="m10.6 10.6 3 3"/></svg>',
    sun: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="3.2"/><path d="M8 1v2M8 13v2M1 8h2M13 8h2M3 3l1.4 1.4M11.6 11.6 13 13M13 3l-1.4 1.4M4.4 11.6 3 13"/></svg>',
    moon: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M13.5 9.5A6 6 0 1 1 6.5 2.5a5 5 0 0 0 7 7z"/></svg>',
    warn: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6"><path d="M8 2 1.5 13.5h13L8 2z"/><path d="M8 6.5v3.2M8 11.8v.2"/></svg>',
    ok: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8"><path d="m2.5 8.5 3.5 3.5 7.5-8"/></svg>',
    bolt: '<svg viewBox="0 0 16 16" fill="currentColor"><path d="M9 1 3 9h4l-1 6 6-8H8l1-6z"/></svg>',
    link: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M6.5 9.5 9.5 6.5M5 11l-1.2 1.2a2.5 2.5 0 0 1-3.5-3.5L3.5 5.5a2.5 2.5 0 0 1 3.5 0M11 5l1.2-1.2a2.5 2.5 0 0 1 3.5 3.5L12.5 10.5a2.5 2.5 0 0 1-3.5 0" transform="translate(-1.5 -0.5)"/></svg>',
    ship: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M2 11h12l-1.5 3h-9L2 11zM8 2v6M5 5l3-3 3 3"/></svg>',
    open: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M6 3H3v10h10v-3M9 2h5v5M14 2 7.5 8.5"/></svg>',
    graph: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="3.5" cy="8" r="1.8"/><circle cx="12.5" cy="3.5" r="1.8"/><circle cx="12.5" cy="12.5" r="1.8"/><path d="M5.2 7.2 10.8 4.2M5.2 8.8l5.6 3"/></svg>',
    x: '<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M3.5 3.5l9 9M12.5 3.5l-9 9"/></svg>',
  };

  const initial = esc((D.project || "C").charAt(0).toUpperCase());
  app.innerHTML =
    '<div class="bar"><div class="bar-in">' +
      '<div class="logo" aria-hidden="true">' + initial + '</div>' +
      '<div class="ttl">' + esc(D.project) + ' · Status<small>one page, no tabs</small></div>' +
      extA(GH ? GH + "/releases" : null, "vpill", "v" + esc(D.version)) +
      (D.branch ? extA(GH ? GH + "/tree/" + encodeURIComponent(D.branch) : null, "vpill branch",
        "⎇ " + esc(D.branch), "snapshot branch on GitHub") : "") +
      '<span class="stamp">' + extA(ghc(D.head), "stamp-a", "snapshot " + esc(TODAY) +
        (D.head ? " @ " + esc(D.head) : "") +
        (D.commit ? " · " + esc(D.commit) : "")) + "</span>" +
      ((D.coverageAsOf || D.head)
        ? '<span class="vpill" title="Commit coverage tip at render time. A commit that stages this page cannot include its own hash — CI/deploy closes the gap.">coverage as of parent ' +
          esc(D.coverageAsOf || D.head) + "</span>"
        : "") +
      (staleDays > 1 ? '<span class="vpill stale" title="Snapshot is more than a day old — regenerate via status-page / pre-commit">snapshot ' +
        staleDays + "d old</span>" : "") +
      (D.noGit ? '<span class="vpill stale" title="Git history was unavailable at render time">no git history available</span>' : "") +
      '<div class="bar-spring"></div>' +
      '<div id="selslot"></div>' +
      '<div class="search"><span class="sicon">' + ICONS.search + '</span>' +
        '<input id="q" type="text" placeholder="Find task, module, release…" autocomplete="off" spellcheck="false">' +
        '<kbd>/</kbd><div class="sr-drop" id="srdrop" hidden></div></div>' +
      '<button class="theme-btn" data-act="theme" title="Toggle theme" aria-label="Toggle theme">' + ICONS.moon + '</button>' +
    '</div></div>' +
    '<div class="wrap">' +
      '<section class="band" id="pulse"><div class="band-hd"><span class="num">01</span><h2>Pulse</h2>' +
        '<span class="sub">where the project stands, zero clicks</span></div>' +
        '<div class="hero" id="hero"></div><div class="att-row" id="attrow"></div></section>' +
      '<section class="band" id="roadmap"><div class="band-hd"><span class="num">02</span><h2>Roadmap</h2>' +
        '<span class="sub">program lanes, releases on the real calendar, shipped burn-up</span></div>' +
        '<div class="card road-card"><div class="road-scroll" id="roadwrap"></div>' +
        '<div class="road-foot"><div class="legend">' +
          '<span><i style="background:var(--done)"></i>shipped burn-up</span>' +
          '<span><i style="background:var(--accent)"></i>release</span>' +
          '<span><i style="background:var(--bad)"></i>release with unlinked commits</span>' +
          '<span><i style="background:var(--panel-3);border:1px solid var(--line)"></i>lane span · fill = done</span></div>' +
          '<div class="rel-strip" id="relstrip"></div></div></div></section>' +
      '<section class="band" id="sysmap"><div class="band-hd"><span class="num">03</span><h2>System map</h2>' +
        '<span class="sub">every module, sized by scope, wired by real task dependencies</span></div>' +
        '<div class="map-grid"><div class="card map-card" id="mapcard"></div><div class="rank" id="rank"></div></div></section>' +
      '<section class="band" id="flowband"><div class="band-hd"><span class="num">04</span><h2>Flow</h2>' +
        '<span class="sub">the dependency graph — what is ready, what is blocked, what the longest chain is</span>' +
        '<div class="tools" id="flowtools"></div></div>' +
        '<div class="card flow-card"><div class="flow-scroll" id="flowwrap"></div><div class="flow-foot" id="flowfoot"></div></div></section>' +
      '<section class="band" id="ledger"><div class="band-hd"><span class="num">05</span><h2>Releases &amp; traceability</h2>' +
        '<span class="sub">release notes in three types, every commit accounted for</span></div>' +
        '<div id="ledgerwrap"></div></section>' +
      '<section class="band" id="indexband"><div class="band-hd"><span class="num">06</span><h2>Index</h2>' +
        '<span class="sub">every task, grouped by module — click any row for the full record</span>' +
        '<div class="tools" id="idxtools"></div></div>' +
        '<div class="card idx-card" id="idxcard"></div></section>' +
      '<div class="foot">' +
        '<span>Generated from task frontmatter + CHANGELOG.md + git history.</span>' +
        '<span class="mono">' + esc(D.commit || D.fp || "") + '</span>' +
        '<span>Feed <span class="mono">' + esc(D.fp || "") + '</span></span>' +
        '<span><a href="status-legacy.html">Legacy status page (v2 lenses)</a></span>' +
      '</div>' +
    '</div>' +
    '<div class="scrim" id="scrim" data-act="close-drawer"></div>' +
    '<aside class="drawer" id="drawer" aria-label="Task detail"></aside>' +
    '<div id="tip" role="tooltip"></div>';

  /* ═══ 4 · tooltip ══════════════════════════════════════════════════════ */
  const tip = document.getElementById("tip");
  function tipShow(html, x, y) {
    tip.innerHTML = html;
    tip.classList.add("on");
    const w = tip.offsetWidth, h = tip.offsetHeight;
    let tx = x + 14, ty = y + 14;
    if (tx + w > innerWidth - 8) tx = x - w - 12;
    if (ty + h > innerHeight - 8) ty = y - h - 12;
    tip.style.left = Math.max(6, tx) + "px";
    tip.style.top = Math.max(6, ty) + "px";
  }
  const tipHide = () => tip.classList.remove("on");

  /* ═══ 5 · pulse ════════════════════════════════════════════════════════ */
  function ringSvg(p) {
    const r = 20, c = 2 * Math.PI * r;
    return '<svg class="ring" viewBox="0 0 52 52">' +
      '<circle cx="26" cy="26" r="' + r + '" fill="none" stroke="var(--panel-3)" stroke-width="6"/>' +
      '<circle cx="26" cy="26" r="' + r + '" fill="none" stroke="var(--done)" stroke-width="6" stroke-linecap="round" ' +
      'stroke-dasharray="' + (c * p / 100).toFixed(1) + " " + c.toFixed(1) + '" transform="rotate(-90 26 26)"/>' +
      '<text x="26" y="30" text-anchor="middle">' + p + "</text></svg>";
  }
  function sparkSvg() {
    const B = D.burnup || [];
    if (B.length < 2) return "";
    const t0 = dnum(B[0].d), t1 = dnum(B[B.length - 1].d) || 1;
    const max = B[B.length - 1].n || 1;
    const px = d => (84 * (dnum(d) - t0) / Math.max(1, t1 - t0)).toFixed(1);
    const py = n => (24 - 22 * n / max).toFixed(1);
    let path = "M" + px(B[0].d) + " " + py(B[0].n);
    B.forEach(b => { path += " L" + px(b.d) + " " + py(b.n); });
    return '<svg class="trend" viewBox="0 0 84 26"><path d="' + path +
      '" fill="none" stroke="var(--done)" stroke-width="1.6"/></svg>';
  }
  const nextNotes = (D.unreleased || {}).notes || { features: [], fixes: [], improvements: [] };
  const nextCount = nextNotes.features.length + nextNotes.fixes.length + nextNotes.improvements.length;
  const unrelCov = ((D.unreleased || {}).cov || { linked: 0, exempt: 0, unlinked: 0 });

  document.getElementById("hero").innerHTML =
    '<button class="vt card tone-done" data-act="go" data-id="indexband">' +
      '<span class="lbl">Shipped</span>' + ringSvg(pct(sDone, TASKS.length)) +
      '<div class="big">' + sDone + '<small> / ' + TASKS.length + '</small></div>' +
      '<div class="sub"><b>' + retired + '</b> retired (closed · dup)</div>' + sparkSvg() + '</button>' +
    '<button class="vt card tone-active" data-act="go-flow" data-id="focus">' +
      '<span class="lbl">In flight</span><div class="big">' + nActive + '</div>' +
      '<div class="sub"><b>' + nDraft + '</b> drafts queued · <b>' + nHold + '</b> on hold</div></button>' +
    '<button class="vt card tone-done" data-act="go-flow" data-id="focus">' +
      '<span class="lbl">Ready to start</span><div class="big">' + FRONTIER.length + '</div>' +
      '<div class="sub">every dependency met</div></button>' +
    '<button class="vt card tone-bad" data-act="go-flow" data-id="focus">' +
      '<span class="lbl">Blocked</span><div class="big">' + BLOCKED.length + '</div>' +
      '<div class="sub">waiting on unfinished deps</div></button>' +
    '<button class="vt card ' + (covCur.unlinked ? "tone-bad" : "tone-done") + '" data-act="go" data-id="ledger">' +
      '<span class="lbl">Traceability</span><div class="big">' + covPct + '<small>%</small></div>' +
      '<div class="sub"><b>' + num(covCur.unlinked) + '</b> unlinked commits since the version reset</div></button>' +
    '<button class="vt card tone-hold" data-act="go" data-id="ledger">' +
      '<span class="lbl">Next release</span><div class="big">' + nextCount + '</div>' +
      '<div class="sub">notes staged · <b>' + (unrelCov.commits ? unrelCov.commits.length : 0) + '</b> commits pending</div></button>';

  /* attention cards */
  const topBlocker = (() => {
    const cnt = {};
    BLOCKED.forEach(t => unmetDeps(t).forEach(d => { cnt[d] = (cnt[d] || 0) + 1; }));
    let best = null; Object.keys(cnt).forEach(k => { if (!best || cnt[k] > cnt[best]) best = k; });
    return best ? { id: best, n: cnt[best] } : null;
  })();
  const att = [];
  if (covCur.unlinked)
    att.push({ sev: "crit", ic: ICONS.link, b: num(covCur.unlinked) + " commits break the traceability rule",
      s: "No task link in subject or body since the v0.1.0 reset" +
        (covLeg.unlinked ? " (+" + num(covLeg.unlinked) + " in the first epoch)" : "") +
        ". The proposed commit-msg gate + CI check makes this structurally impossible.",
      act: "go", id: "ledger" });
  if (topBlocker)
    att.push({ sev: "crit", ic: ICONS.warn, b: short(topBlocker.id) + " is the biggest blocker",
      s: "Finishing it unblocks " + topBlocker.n + " task" + (topBlocker.n > 1 ? "s" : "") + ". " + BLOCKED.length + " tasks are blocked in total.",
      act: "task", id: topBlocker.id });
  if (CRIT.length > 1)
    att.push({ sev: "warn", ic: ICONS.bolt, b: "Critical path: " + CRIT.length + " tasks · ~" + cpHours + "h",
      s: short(CRIT[0]) + " → … → " + short(CRIT[CRIT.length - 1]) + ". Longest unfinished dependency chain.",
      act: "go-flow", id: "focus" });
  if (STALE.length)
    att.push({ sev: "warn", ic: ICONS.warn, b: STALE.length + " drafts stale for 45+ days",
      s: "Oldest from " + STALE.reduce((a, t) => a < t.cr ? a : t.cr, TODAY) + ". Groom, close, or schedule.",
      act: "go", id: "indexband" });
  if (nextCount)
    att.push({ sev: "info", ic: ICONS.ship, b: "Next release carries " + nextCount + " staged notes",
      s: nextNotes.features.length + " features · " + nextNotes.fixes.length + " fixes · " + nextNotes.improvements.length + " improvements in Unreleased.",
      act: "go", id: "ledger" });
  att.push({ sev: "ok", ic: ICONS.ok, b: FRONTIER.length + " tasks are ready to start now",
    s: "All dependencies met — the frontier is wide open.", act: "go-flow", id: "focus" });
  document.getElementById("attrow").innerHTML = att.slice(0, 6).map(a =>
    '<button class="att ' + a.sev + '" data-act="' + a.act + '" data-id="' + esc(a.id) + '">' +
    '<span class="ic">' + a.ic + '</span><span><b>' + esc(a.b) + '</b><span>' + esc(a.s) + '</span></span></button>').join("");

  /* ═══ 6 · roadmap ══════════════════════════════════════════════════════ */
  (function roadmap() {
    const phases = D.phases || [];
    const W = 1180, LX = 96, RX = W - 26;
    const dates = [];
    TASKS.forEach(t => { if (t.cr) dates.push(t.cr); });
    RELS.forEach(r => { if (r.d) dates.push(r.d); });
    dates.push(TODAY);
    const t0 = Math.min.apply(null, dates.map(dnum).filter(Boolean)) - 3 * dayMs;
    const t1 = Math.max.apply(null, dates.map(dnum).filter(Boolean)) + 9 * dayMs;
    const X = d => LX + (RX - LX) * ((typeof d === "number" ? d : dnum(d)) - t0) / (t1 - t0);

    const BURN_H = 66, BURN_Y = 20;
    const laneH = 24, laneGap = 7;
    const lanesY = BURN_Y + BURN_H + 24;
    const axisY = lanesY + phases.length * (laneH + laneGap) + 26;
    const H = axisY + 44;

    let s = '<svg id="roadsvg" viewBox="0 0 ' + W + " " + H + '" role="img" aria-label="Roadmap">';

    /* month grid */
    const d0 = new Date(t0); d0.setDate(1);
    for (let d = new Date(d0); d.getTime() < t1; d.setMonth(d.getMonth() + 1)) {
      const x = X(d.getTime());
      if (x < LX - 1) continue;
      s += '<line class="month-line" x1="' + x.toFixed(1) + '" y1="14" x2="' + x.toFixed(1) + '" y2="' + (axisY + 6) + '"/>' +
        '<text class="month-lab" x="' + (x + 4).toFixed(1) + '" y="12">' +
        d.toLocaleString("en", { month: "short" }) + " " + String(d.getFullYear()).slice(2) + "</text>";
    }

    /* burn-up */
    const B = D.burnup || [];
    if (B.length) {
      const maxN = B[B.length - 1].n;
      const Y = n => BURN_Y + BURN_H - BURN_H * n / Math.max(1, maxN);
      let line = "M" + X(B[0].d).toFixed(1) + " " + Y(0).toFixed(1);
      B.forEach(b => { line += " L" + X(b.d).toFixed(1) + " " + Y(b.n).toFixed(1); });
      const area = line + " L" + X(B[B.length - 1].d).toFixed(1) + " " + (BURN_Y + BURN_H).toFixed(1) +
        " L" + X(B[0].d).toFixed(1) + " " + (BURN_Y + BURN_H).toFixed(1) + " Z";
      s += '<path class="burn-area" d="' + area + '"/><path class="burn-line" d="' + line + '"/>' +
        '<text class="burn-lab" x="' + (X(B[B.length - 1].d) + 6).toFixed(1) + '" y="' + (Y(maxN) + 4).toFixed(1) + '">' +
        maxN + " shipped</text>" +
        '<text class="ph-count" x="' + LX + '" y="' + (BURN_Y - 4) + '">cumulative tasks shipped</text>';
    }

    /* phase lanes */
    phases.forEach((p, i) => {
      const y = lanesY + i * (laneH + laneGap);
      const x0 = Math.max(LX, X(p.start || TODAY));
      const ongoing = p.done < p.total;
      const x1 = Math.min(RX, ongoing ? X(TODAY) + 14 : X(p.lastShip || TODAY));
      const wBar = Math.max(26, x1 - x0);
      const fill = wBar * (p.total ? p.done / p.total : 0);
      s += '<g class="ph" data-phase="' + esc(p.id) + '" data-act="phase" data-id="' + esc(p.id) + '">' +
        '<text class="ph-name" x="8" y="' + (y + 16) + '">' + esc(p.label) + '</text>' +
        '<rect class="ph-track" x="' + x0.toFixed(1) + '" y="' + y + '" width="' + wBar.toFixed(1) + '" height="' + laneH + '" rx="7"/>' +
        (fill > 1 ? '<rect class="ph-fill" x="' + x0.toFixed(1) + '" y="' + y + '" width="' + fill.toFixed(1) + '" height="' + laneH + '" rx="7"/>' : "") +
        '<text class="ph-count" x="' + (x0 + wBar + 8).toFixed(1) + '" y="' + (y + 16) + '">' + p.done + "/" + p.total + "</text>" +
        '<rect class="ph-hit" x="0" y="' + (y - 2) + '" width="' + W + '" height="' + (laneH + 4) + '"/></g>';
    });

    /* release axis with date-true clustering */
    s += '<line class="ax" x1="' + LX + '" y1="' + axisY + '" x2="' + RX + '" y2="' + axisY + '"/>';
    const clusters = [];
    CURRELS.slice().reverse().forEach(r => {
      const x = r.d ? X(r.d) : RX;
      const c = clusters.length && (x - clusters[clusters.length - 1].x) < 16
        ? clusters[clusters.length - 1] : null;
      if (c) { c.items.push(r); }
      else clusters.push({ x, items: [r] });
    });
    clusters.forEach(c => {
      const bad = c.items.some(r => r.cov.unlinked > 0);
      const one = c.items.length === 1;
      const idAttr = ' data-act="rel" data-id="' + esc(c.items[c.items.length - 1].v) + '" data-cluster="' +
        esc(c.items.map(r => r.v).join(",")) + '"';
      s += '<g class="rel"' + idAttr + ' transform="translate(' + c.x.toFixed(1) + "," + axisY + ')">' +
        '<circle r="' + (one ? 6 : 9) + '"/>' +
        (one ? '<text y="-11">' + esc(c.items[0].v) + "</text>"
             : '<text class="cnt" y="3.5">' + c.items.length + "</text>") +
        (bad ? '<rect class="viol-tick" x="-4" y="12" width="8" height="3" rx="1.5"/>' : "") +
        "</g>";
    });

    /* today */
    const xT = X(TODAY);
    s += '<line class="today-line" x1="' + xT.toFixed(1) + '" y1="14" x2="' + xT.toFixed(1) + '" y2="' + (axisY + 18) + '"/>' +
      '<text class="today-lab" x="' + (xT + 5).toFixed(1) + '" y="' + (axisY + 30) + '">today · ' + esc(TODAY) + "</text>";

    s += "</svg>";
    document.getElementById("roadwrap").innerHTML = s;

    document.getElementById("relstrip").innerHTML = CURRELS.map(r =>
      '<button class="rv-chip" data-act="rel" data-id="' + esc(r.v) + '">v' + esc(r.v) +
      '<span class="d">' + esc((r.d || "").slice(5)) + "</span></button>").join("");
  })();

  /* ═══ 7 · system map ═══════════════════════════════════════════════════ */
  function arcPath(r, a0, a1) {
    const large = (a1 - a0) > Math.PI ? 1 : 0;
    return "M" + (r * Math.cos(a0)).toFixed(1) + " " + (r * Math.sin(a0)).toFixed(1) +
      " A" + r + " " + r + " 0 " + large + " 1 " +
      (r * Math.cos(a1)).toFixed(1) + " " + (r * Math.sin(a1)).toFixed(1);
  }
  (function sysmap() {
    let s = '<svg id="map" viewBox="0 0 980 640">' +
      '<defs><marker id="marr" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="5.5" markerHeight="5.5" orient="auto-start-reverse">' +
      '<path d="M0 0L8 4L0 8z" fill="currentColor" opacity=".55"/></marker></defs>';
    (D.medges || []).forEach((e, i) => {
      const a = MOD[e.from], b = MOD[e.to];
      if (!a || !b) return;
      const dx = b.x - a.x, dy = b.y - a.y, d = Math.hypot(dx, dy) || 1;
      const x1 = a.x + dx / d * (a.r + 6), y1 = a.y + dy / d * (a.r + 6);
      const x2 = b.x - dx / d * (b.r + 12), y2 = b.y - dy / d * (b.r + 12);
      const mx = (x1 + x2) / 2 - dy / d * 22, my = (y1 + y2) / 2 + dx / d * 22;
      s += '<path class="medge" data-from="' + esc(e.from) + '" data-to="' + esc(e.to) + '" data-ei="' + i +
        '" d="M' + x1.toFixed(0) + " " + y1.toFixed(0) + " Q" + mx.toFixed(0) + " " + my.toFixed(0) +
        " " + x2.toFixed(0) + " " + y2.toFixed(0) + '" stroke-width="' +
        Math.min(4.5, 0.7 + e.w * 0.45).toFixed(1) + '" marker-end="url(#marr)"/>';
    });
    MODS.forEach(m => {
      const segs = [
        ["a-done", m.done || 0], ["a-active", m.active || 0],
        ["a-hold", m.hold || 0], ["a-draft", m.draft || 0],
      ].filter(x => x[1] > 0);
      const gr = m.r + 5;
      let a = -Math.PI / 2, arcs = "";
      const gap = segs.length > 1 ? 0.05 : 0.0001;
      segs.forEach(sg => {
        const span = 2 * Math.PI * sg[1] / m.total - gap;
        if (span > 0.02) arcs += '<path class="arc ' + sg[0] + '" stroke-width="4.5" d="' + arcPath(gr, a, a + span) + '"/>';
        a += span + gap;
      });
      const blocked = modBlocked[m.id] || 0;
      const fs = m.r > 42 ? 13 : m.r > 26 ? 11.5 : 10.5;
      s += '<g class="mnode" data-act="mod" data-id="' + esc(m.id) + '" transform="translate(' + m.x + "," + m.y + ')">' +
        '<circle class="body" r="' + m.r + '"/>' + arcs +
        ((m.active || 0) > 0 ? '<circle class="pulse" r="3" cx="' + (gr * 0.72).toFixed(0) + '" cy="-' + (gr * 0.72).toFixed(0) + '"/>' +
          '<circle class="pulse-halo" r="3" cx="' + (gr * 0.72).toFixed(0) + '" cy="-' + (gr * 0.72).toFixed(0) + '"/>' : "") +
        '<text class="nm" y="-1" font-size="' + fs + '">' + esc(m.id) + '</text>' +
        '<text class="ct" y="12" font-size="' + (fs - 2) + '">' + (m.done || 0) + "/" + m.total + "</text>" +
        (blocked ? '<g class="bbadge" transform="translate(' + (gr * 0.75).toFixed(0) + "," + (gr * 0.72).toFixed(0) + ')">' +
          '<circle r="8"/><text y="3">' + blocked + "</text></g>" : "") +
        "</g>";
    });
    s += "</svg>";
    document.getElementById("mapcard").innerHTML = s +
      '<div class="map-hint">hover a module to trace its couplings · click to focus the whole page on it</div>';

    /* ranking rail */
    const openOf = m => m.total - (m.done || 0);
    const rows = (items, valFn, color) => {
      const max = Math.max.apply(null, items.map(valFn).concat([1]));
      return items.map(m =>
        '<button class="rrow" data-act="mod" data-id="' + esc(m.id) + '">' +
        '<span class="rr-id">' + esc(m.id) + '</span>' +
        '<span class="rr-bar"><i style="width:' + pct(valFn(m), max) + "%;background:" + color + '"></i></span>' +
        '<span class="rr-n">' + valFn(m) + "</span></button>").join("");
    };
    const byOpen = MODS.filter(m => openOf(m) > 0).sort((a, b) => openOf(b) - openOf(a)).slice(0, 6);
    const byBlocked = MODS.filter(m => modBlocked[m.id]).sort((a, b) => (modBlocked[b.id] || 0) - (modBlocked[a.id] || 0)).slice(0, 6);
    const bySize = MODS.slice().sort((a, b) => b.total - a.total).slice(0, 6);
    document.getElementById("rank").innerHTML =
      '<div class="card rank-block"><h4>Most open work</h4>' + rows(byOpen, openOf, "var(--accent)") + "</div>" +
      '<div class="card rank-block"><h4>Most blocked</h4>' +
        (byBlocked.length ? rows(byBlocked, m => modBlocked[m.id] || 0, "var(--bad)") :
        '<div class="ig-meta">no blocked tasks — clean frontier</div>') + "</div>" +
      '<div class="card rank-block"><h4>Largest scope</h4>' + rows(bySize, m => m.total, "var(--draft)") + "</div>";
  })();

  function mapFocus(mod) {
    const touching = {};
    if (mod) {
      touching[mod] = 1;
      (D.medges || []).forEach(e => {
        if (e.from === mod) touching[e.to] = 1;
        if (e.to === mod) touching[e.from] = 1;
      });
    }
    document.querySelectorAll("#map .medge").forEach(p => {
      const hot = mod && (p.getAttribute("data-from") === mod || p.getAttribute("data-to") === mod);
      p.classList.toggle("hot", !!hot);
      p.classList.toggle("dim", !!mod && !hot);
    });
    document.querySelectorAll("#map .mnode").forEach(g => {
      const id = g.getAttribute("data-id");
      g.classList.toggle("sel", mod === id && S.sel && S.sel.kind === "module" && S.sel.id === id);
      g.classList.toggle("dim", !!mod && !touching[id]);
    });
  }

  /* ═══ 8 · flow (dependency graph) ══════════════════════════════════════ */
  function flowScopeIds() {
    if (S.flow === "sel" && S.sel) {
      if (S.sel.kind === "module")
        return TASKS.filter(t => t.m === S.sel.id && (S.flowDone || unfinished(t))).map(t => t.i);
      if (S.sel.kind === "phase")
        return TASKS.filter(t => t.pg === S.sel.id && (S.flowDone || unfinished(t))).map(t => t.i);
      if (S.sel.kind === "task") {
        const set = new Set([S.sel.id]);
        const up = [S.sel.id], dn = [S.sel.id];
        while (up.length) { const id = up.pop(); (BY[id] ? BY[id].d : []).forEach(d => { if (BY[d] && !set.has(d)) { set.add(d); up.push(d); } }); }
        while (dn.length) { const id = dn.pop(); (REV[id] || []).forEach(c => { if (!set.has(c)) { set.add(c); dn.push(c); } }); }
        return [...set].filter(id => BY[id] && (S.flowDone || unfinished(BY[id]) || id === S.sel.id));
      }
      if (S.sel.kind === "release") {
        const r = RELBYV[S.sel.id];
        const set = new Set();
        if (r) {
          ["features", "fixes", "improvements"].forEach(k => (r.notes[k] || []).forEach(it => (it.ids || []).forEach(id => set.add(id))));
          (r.cov.commits || []).forEach(c => (c.ids || []).forEach(id => set.add(id)));
          [...set].forEach(id => (BY[id] ? BY[id].d : []).forEach(d => { if (BY[d]) set.add(d); }));
        }
        return [...set].filter(id => BY[id]);
      }
    }
    if (S.flow === "open")
      return TASKS.filter(t => unfinished(t) && ((t.d || []).length || (REV[t.i] || []).length)).map(t => t.i);
    /* focus: critical path ∪ blocked ∪ unblocking frontier, scored, capped */
    const score = {};
    CRIT.forEach((id, i) => { score[id] = Math.max(score[id] || 0, 1000 - i); });
    BLOCKED.forEach(t => { score[t.i] = Math.max(score[t.i] || 0, 500 + (REV[t.i] || []).length); });
    FRONTIER.forEach(t => {
      const outs = (REV[t.i] || []).filter(c => BY[c] && unfinished(BY[c])).length;
      if (outs) score[t.i] = Math.max(score[t.i] || 0, 200 + outs);
    });
    return Object.keys(score).sort((a, b) => score[b] - score[a]).slice(0, 64);
  }

  function renderFlowTools(nScoped) {
    const selLabel = S.sel ? (S.sel.kind === "task" ? short(S.sel.id) : S.sel.id) : null;
    document.getElementById("flowtools").innerHTML =
      '<button class="fchip' + (S.flow === "focus" ? " on" : "") + '" data-act="flow-mode" data-id="focus">Focus<span class="n">auto</span></button>' +
      (selLabel ? '<button class="fchip' + (S.flow === "sel" ? " on" : "") + '" data-act="flow-mode" data-id="sel">' +
        esc(S.sel.kind) + ": " + esc(selLabel) + "</button>" : "") +
      '<button class="fchip' + (S.flow === "open" ? " on" : "") + '" data-act="flow-mode" data-id="open">All open<span class="n">' +
        TASKS.filter(t => unfinished(t) && ((t.d || []).length || (REV[t.i] || []).length)).length + "</span></button>" +
      '<button class="fchip" data-act="flow-done">' + (S.flowDone ? "hide" : "show") + " done</button>";
  }

  function renderFlow() {
    const ids = flowScopeIds();
    const wrap = document.getElementById("flowwrap");
    renderFlowTools(ids.length);
    if (!ids.length) {
      wrap.innerHTML = '<div class="flow-empty">Nothing in this scope — pick a module, phase, or release, or switch to All open.</div>';
      document.getElementById("flowfoot").innerHTML = "";
      return;
    }
    const set = new Set(ids);

    /* longest-path layering (cycle-safe) */
    const depth = {}, st = {};
    function dep(id) {
      if (id in depth) return depth[id];
      if (st[id]) return 0;
      st[id] = 1;
      let d = 0;
      (BY[id].d || []).forEach(p => { if (set.has(p)) d = Math.max(d, dep(p) + 1); });
      st[id] = 0;
      return (depth[id] = d);
    }
    ids.forEach(dep);
    const nCols = Math.max.apply(null, ids.map(id => depth[id])) + 1;
    let cols = Array.from({ length: nCols }, () => []);
    ids.forEach(id => cols[depth[id]].push(id));
    cols.forEach(c => c.sort((a, b) => (BY[a].m + a) < (BY[b].m + b) ? -1 : 1));

    /* barycenter sweeps to cut crossings */
    const pos = {};
    const reindex = () => cols.forEach(c => c.forEach((id, i) => { pos[id] = i; }));
    reindex();
    for (let pass = 0; pass < 3; pass++) {
      for (let ci = 1; ci < nCols; ci++) {
        cols[ci].sort((a, b) => bary(a, ci, -1) - bary(b, ci, -1) || (a < b ? -1 : 1)); reindex();
      }
      for (let ci = nCols - 2; ci >= 0; ci--) {
        cols[ci].sort((a, b) => bary(a, ci, 1) - bary(b, ci, 1) || (a < b ? -1 : 1)); reindex();
      }
    }
    function bary(id, ci, dir) {
      const nb = dir < 0
        ? (BY[id].d || []).filter(p => set.has(p) && depth[p] === ci - 1)
        : (REV[id] || []).filter(c => set.has(c) && depth[c] === ci + 1);
      if (!nb.length) return pos[id];
      return nb.reduce((s, n) => s + pos[n], 0) / nb.length;
    }

    const NW = 158, NH = 26, GX = 52, GY = 9, PADX = 18, PADY = 30;
    const maxRows = Math.max.apply(null, cols.map(c => c.length));
    const Wsvg = PADX * 2 + nCols * NW + (nCols - 1) * GX;
    const Hsvg = PADY + maxRows * (NH + GY) + 16;
    const xy = {};
    cols.forEach((col, ci) => {
      const x = PADX + ci * (NW + GX);
      const y0 = PADY + (maxRows - col.length) * (NH + GY) / 2;
      col.forEach((id, ri) => { xy[id] = { x, y: y0 + ri * (NH + GY) }; });
    });

    let edges = "", nodes = "", nEdges = 0, nBlockedE = 0;
    ids.forEach(id => {
      (BY[id].d || []).forEach(p => {
        if (!set.has(p)) return;
        const a = xy[p], b = xy[id];
        const x1 = a.x + NW, y1 = a.y + NH / 2, x2 = b.x, y2 = b.y + NH / 2;
        const blocked = unfinished(BY[id]) && BY[p] && unfinished(BY[p]);
        const crit = CRITNEXT[p] === id;
        nEdges++; if (blocked) nBlockedE++;
        edges += '<path class="fedge' + (blocked ? " blocked" : "") + (crit ? " crit" : "") +
          '" data-a="' + esc(p) + '" data-b="' + esc(id) + '" d="M' + x1 + " " + y1 +
          " C" + (x1 + GX / 2) + " " + y1 + " " + (x2 - GX / 2) + " " + y2 + " " + x2 + " " + y2 + '"/>';
      });
    });
    ids.forEach(id => {
      const t = BY[id], p = xy[id];
      const ext = (t.d || []).filter(d => !set.has(d)).length;
      const ttl = t.t.length > 21 ? t.t.slice(0, 20) + "…" : t.t;
      nodes += '<g class="fnode b-' + t.b + (CRITSET.has(id) ? " crit" : "") + (BLOCKEDSET.has(id) ? " blocked" : "") +
        '" data-act="task" data-id="' + esc(id) + '" transform="translate(' + p.x + "," + p.y + ')">' +
        '<rect width="' + NW + '" height="' + NH + '" rx="7"/>' +
        '<circle class="fdot b-' + t.b + '" cx="12" cy="' + NH / 2 + '" r="4"/>' +
        '<text class="fid" x="22" y="11">' + esc(short(id)) + "</text>" +
        '<text class="fttl" x="22" y="21">' + esc(ttl) + "</text>" +
        (t.p === "p0" ? '<text class="pflag" x="' + (NW - 18) + '" y="11">P0</text>' : "") +
        (ext ? '<text class="stub" x="-14" y="' + (NH / 2 + 3) + '">◂' + ext + "</text>" : "") +
        "</g>";
    });

    wrap.innerHTML = '<svg id="flow" width="' + Wsvg + '" height="' + Hsvg + '" viewBox="0 0 ' + Wsvg + " " + Hsvg + '">' +
      '<text class="col-lab" x="' + PADX + '" y="16">UPSTREAM (build first)</text>' +
      '<text class="col-lab" x="' + (Wsvg - PADX) + '" y="16" text-anchor="end">DOWNSTREAM (unlocked later)</text>' +
      edges + nodes + "</svg>";

    document.getElementById("flowfoot").innerHTML =
      "<span><b>" + ids.length + "</b> tasks · <b>" + nEdges + "</b> edges in view</span>" +
      '<span class="legend"><i style="background:var(--done)"></i>done <i style="background:var(--active)"></i>active ' +
      '<i style="background:var(--draft)"></i>draft <i style="background:var(--hold)"></i>hold</span>' +
      '<span style="color:var(--bad)">dashed red = blocking edge (' + nBlockedE + ")</span>" +
      '<span style="color:var(--accent-ink)">thick amber = critical path</span>' +
      "<span>hover = trace the full cone · click = task record · ◂n = deps outside this view</span>";
  }

  function flowCone(id, on) {
    const svg = document.getElementById("flow");
    if (!svg) return;
    if (!on) {
      svg.querySelectorAll(".dim, .hot").forEach(n => n.classList.remove("dim", "hot"));
      return;
    }
    const cone = new Set([id]);
    const up = [id], dn = [id];
    while (up.length) { const x = up.pop(); (BY[x] ? BY[x].d : []).forEach(d => { if (!cone.has(d)) { cone.add(d); up.push(d); } }); }
    while (dn.length) { const x = dn.pop(); (REV[x] || []).forEach(c => { if (!cone.has(c)) { cone.add(c); dn.push(c); } }); }
    svg.querySelectorAll(".fnode").forEach(n => {
      n.classList.toggle("dim", !cone.has(n.getAttribute("data-id")));
    });
    svg.querySelectorAll(".fedge").forEach(e => {
      const inCone = cone.has(e.getAttribute("data-a")) && cone.has(e.getAttribute("data-b"));
      e.classList.toggle("hot", inCone);
      e.classList.toggle("dim", !inCone);
    });
  }

  /* ═══ 9 · ledger ═══════════════════════════════════════════════════════ */
  function taskChip(id) {
    const t = BY[id];
    return '<button class="tchip" data-act="task" data-id="' + id + '"><span class="dt ' +
      (t ? "b-" + t.b : "ghost") + '"></span>' + short(id) + "</button>";
  }
  /* token-substitution so canonical ids, shorthand ids, and PR refs never nest */
  function chipify(text) {
    const toks = [];
    const tok = html => { toks.push(html); return "\u0000" + (toks.length - 1) + "\u0000"; };
    let s = esc(text);
    s = s.replace(/\bTASK-[A-Z][A-Z0-9]*-\d+\b/g, id => tok(taskChip(id)));
    s = s.replace(/\b([A-Z]{2,10})-(\d{1,4})\b/g, (m0, pre, n) => {
      const a = "TASK-" + pre + "-" + String(n).padStart(3, "0"), b = "TASK-" + pre + "-" + n;
      const id = BY[a] ? a : BY[b] ? b : null;
      return id ? tok(taskChip(id)) : m0;
    });
    if (GH) s = s.replace(/#(\d{2,6})\b/g, (m0, n) =>
      tok('<a class="prlink" target="_blank" rel="noopener" href="' + GH + "/pull/" + n + '">#' + n + "</a>"));
    return s.replace(/\u0000(\d+)\u0000/g, (m0, i) => toks[+i]);
  }
  function covBarHtml(cov) {
    const tot = (cov.linked + cov.exempt + cov.unlinked) || 1;
    return '<div class="covwrap"><div class="covbar">' +
      '<i class="c-linked" style="width:' + (100 * cov.linked / tot) + '%"></i>' +
      '<i class="c-exempt" style="width:' + (100 * cov.exempt / tot) + '%"></i>' +
      '<i class="c-unlinked" style="width:' + (100 * cov.unlinked / tot) + '%"></i></div>' +
      '<span class="covlab' + (cov.unlinked ? " bad" : "") + '">' + cov.linked + " linked · " +
      cov.exempt + " exempt" + (cov.unlinked ? " · " + cov.unlinked + " UNLINKED" : "") + "</span></div>";
  }
  function notesCols(notes) {
    const col = (key, label, cls) => {
      const items = notes[key] || [];
      return '<div class="ncol ' + cls + '"><h4><span class="k"></span>' + label + " · " + items.length + "</h4>" +
        (items.length ? items.map(it => '<div class="nitem">' + chipify(it.x) + "</div>").join("")
          : '<div class="nitem"><span class="none">none</span></div>') + "</div>";
    };
    return '<div class="notes3">' + col("features", "Features", "feat") +
      col("fixes", "Fixes", "fix") + col("improvements", "Improvements", "impr") + "</div>";
  }
  const commitStore = {};
  function commitsHtml(cov, storeKey) {
    commitStore[storeKey] = cov.commits || [];
    const n = (cov.commits || []).length;
    if (!n) return "";
    return '<details class="commits" data-ck="' + esc(storeKey) + '"><summary>' + n + " commits · " +
      cov.linked + " linked · " + cov.exempt + " exempt" +
      (cov.unlinked ? ' · <span class="vio">' + cov.unlinked + " unlinked — rule violations</span>" : " · fully traced") +
      "</summary><div class=\"cbody\"></div></details>";
  }
  function renderCommitRows(box, key) {
    if (box.dataset.done) return;
    box.dataset.done = "1";
    const rows = commitStore[key] || [];
    box.innerHTML = rows.map(c =>
      '<div class="crow ' + c.k + '">' + extA(ghc(c.h), "h", esc(c.h), "open commit on GitHub") +
      '<span class="s">' + chipify(c.s) + "</span>" +
      (c.k === "unlinked" ? '<span class="flag">NO TASK LINK</span>' :
        c.k === "exempt" ? '<span class="flag ok">exempt</span>' :
        c.via === "ledger" ? '<span class="flag ok" title="linked via commit-links.yaml">ledger</span>' : "") +
      "</div>").join("");
  }

  (function ledger() {
    let html =
      '<div class="card rule-banner"><span class="ic">' + ICONS.link + "</span>" +
      '<div class="rule-txt"><b>The rule</b>' + esc(D.rule) +
      '<div class="enf"><b>Enforcement:</b> ' + esc(D.enforcement) + "</div></div>" +
      '<div class="rule-stats">' +
      '<div class="rs good"><b>' + num(covCur.linked) + "</b><span>linked</span></div>" +
      '<div class="rs"><b>' + num(covCur.exempt) + "</b><span>exempt</span></div>" +
      '<div class="rs bad"><b>' + num(covCur.unlinked) + "</b><span>unlinked</span></div>" +
      '<div class="rs' + (covPct >= 85 ? " good" : " bad") + '"><b>' + covPct + "%</b><span>coverage</span></div></div></div>";

    /* trend (current epoch only) */
    const asc = CURRELS.slice().reverse();
    const bw = Math.max(14, Math.min(38, Math.floor(1100 / Math.max(1, asc.length)) - 6));
    let tx = 10, bars = "";
    asc.forEach(r => {
      const denom = r.cov.linked + r.cov.unlinked;
      const p = denom ? Math.round(100 * r.cov.linked / denom) : 100;
      const cls = p < 50 ? "low" : p < 85 ? "mid" : "";
      bars += '<g class="tb ' + cls + '" data-act="rel" data-id="' + esc(r.v) + '">' +
        '<rect class="bgb" x="' + tx + '" y="8" width="' + bw + '" height="44" rx="3"/>' +
        '<rect class="fgb" x="' + tx + '" y="' + (8 + 44 * (1 - p / 100)).toFixed(1) + '" width="' + bw +
        '" height="' + (44 * p / 100).toFixed(1) + '" rx="3"/>' +
        '<text x="' + (tx + bw / 2) + '" y="62">' + esc(r.v.replace(/\.0$/, "")) + "</text>" +
        "<title>v" + esc(r.v) + " — " + p + "% of gated commits linked</title></g>";
      tx += bw + 6;
    });
    html += '<div class="card trend-card"><h4>Commit-to-task coverage per release (green ≥ 85%)</h4>' +
      '<svg id="trend" viewBox="0 0 ' + (tx + 10) + ' 70" preserveAspectRatio="xMinYMid meet">' + bars + "</svg></div>";

    html += '<div class="ledger" id="relcards">';
    const unrel = D.unreleased || { notes: nextNotes, cov: { commits: [], linked: 0, exempt: 0, unlinked: 0 } };
    if ((unrel.cov.commits || []).length || nextCount) {
      html += '<div class="card rel-card" id="rel-unreleased"><div class="rel-hd">' +
        '<span class="rv">Unreleased</span><span class="rd">staged for the next cut</span>' +
        '<span class="cnt feat">' + unrel.notes.features.length + ' features</span>' +
        '<span class="cnt fix">' + unrel.notes.fixes.length + ' fixes</span>' +
        '<span class="cnt impr">' + unrel.notes.improvements.length + ' improvements</span>' +
        '<span class="sp"></span>' + covBarHtml(unrel.cov) + "</div>" +
        notesCols(unrel.notes) + commitsHtml(unrel.cov, "unreleased") + "</div>";
    }

    /* featured cards only for releases that actually carry notes */
    const featured = CURRELS.filter(hasNotes);
    const plumbing = CURRELS.filter(r => !hasNotes(r));
    const FOLD = 6;
    featured.forEach((r, i) => {
      html += '<div class="card rel-card' + (i >= FOLD ? " paged" : "") + '" id="rel-' + esc(r.v).replace(/\./g, "-") +
        '"><div class="rel-hd">' +
        extA(relHref(r), "rv", "v" + esc(r.v), "open on GitHub") +
        '<span class="rd">' + esc(r.d || "") + "</span>" +
        '<span class="cnt feat">' + r.notes.features.length + "</span>" +
        '<span class="cnt fix">' + r.notes.fixes.length + "</span>" +
        '<span class="cnt impr">' + r.notes.improvements.length + "</span>" +
        '<span class="sp"></span>' + covBarHtml(r.cov) + "</div>" +
        notesCols(r.notes) + commitsHtml(r.cov, r.v) + "</div>";
    });
    if (featured.length > FOLD)
      html += '<button class="show-more" id="relmore" data-act="show-rels">Show ' +
        (featured.length - FOLD) + " older releases with notes</button>";

    /* zero-note plumbing releases: one compact group, one line each */
    if (plumbing.length) {
      html += '<div class="card rel-slimcard"><details class="commits" open><summary>' + plumbing.length +
        " releases without notes (version plumbing) — the changelog gap the rule closes</summary>" +
        plumbing.map(r =>
          '<div class="slimrow" id="rel-' + esc(r.v).replace(/\./g, "-") + '" data-act="rel" data-id="' + esc(r.v) + '">' +
          extA(relHref(r), "rv rv-s", "v" + esc(r.v)) +
          '<span class="rd">' + esc(r.d || "") + "</span>" +
          '<span class="slim-cov' + (r.cov.unlinked ? " bad" : "") + '">' + (r.cov.commits || []).length + " commits · " +
          r.cov.linked + " linked" + (r.cov.unlinked ? " · " + r.cov.unlinked + " UNLINKED" : "") + "</span></div>").join("") +
        "</details></div>";
    }

    /* first-epoch history, folded */
    RELS.filter(r => r.lg).forEach(r => {
      html += '<div class="card rel-card" id="rel-first-epoch"><div class="rel-hd">' +
        '<span class="rv">first epoch</span><span class="rd">history before the version reset of ' + esc(r.d || "") + "</span>" +
        '<span class="sp"></span>' + covBarHtml(r.cov) + "</div>" +
        '<div class="ncol"><div class="nitem"><span class="none">Version numbers up to v1.9.1 were reissued after the ' +
        esc(r.d || "") + " rollback, so this whole span is kept as one entry. The enforcement gate applies from the cutoff commit forward; this history stays visible, not counted.</span></div></div>" +
        commitsHtml(r.cov, r.v) + "</div>";
    });
    html += "</div>";
    document.getElementById("ledgerwrap").innerHTML = html;
  })();

  /* ═══ 10 · index (filtered, capped, lazy) ══════════════════════════════ */
  const IDX = { f: "open", cap: 30 };
  const IDXF = {
    all: { label: "All", fn: () => true },
    open: { label: "Open", fn: t => t.b !== "done" },
    blocked: { label: "Blocked", fn: t => BLOCKEDSET.has(t.i) },
    ready: { label: "Ready", fn: t => FRONTSET.has(t.i) },
    done: { label: "Done", fn: t => t.b === "done" },
  };
  function idxMatch(t) {
    if (S.sel && S.sel.kind === "phase" && t.pg !== S.sel.id) return false;
    return IDXF[IDX.f].fn(t);
  }
  function renderIdxTools() {
    document.getElementById("idxtools").innerHTML = Object.keys(IDXF).map(k =>
      '<button class="fchip' + (IDX.f === k ? " on" : "") + '" data-act="idx-filter" data-id="' + k + '">' +
      IDXF[k].label + '<span class="n">' + TASKS.filter(t => IDXF[k].fn(t)).length + "</span></button>").join("");
  }
  function renderIndex() {
    const openMods = new Set([...document.querySelectorAll(".idx-group[open]")].map(g => g.getAttribute("data-mod")));
    const groups = MODS.map(m => ({ m, n: TASKS.filter(t => t.m === m.id && idxMatch(t)).length }))
      .filter(g => g.n > 0)
      .sort((a, b) => b.n - a.n || b.m.total - a.m.total);
    document.getElementById("idxcard").innerHTML = groups.map(g => {
      const m = g.m;
      const segs = [["done", m.done || 0], ["active", m.active || 0], ["hold", m.hold || 0], ["draft", m.draft || 0]];
      const bar = segs.map(sg => '<i style="width:' + (100 * sg[1] / m.total) + "%;background:var(--" +
        (sg[0] === "draft" ? "draft" : sg[0]) + ')"></i>').join("");
      return '<details class="idx-group" data-mod="' + esc(m.id) + '"' + (openMods.has(m.id) ? " open" : "") + "><summary>" +
        '<svg class="car" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2"><path d="m6 3.5 4.5 4.5L6 12.5"/></svg>' +
        '<span class="ig-name">' + esc(m.id) + '</span>' +
        '<span class="ig-fc">' + g.n + " " + (IDX.f === "all" ? "task" : IDXF[IDX.f].label.toLowerCase()) + (g.n > 1 && IDX.f === "all" ? "s" : "") + "</span>" +
        '<span class="ig-meta">' + m.total + " total · " + (m.done || 0) + " done" +
        (modBlocked[m.id] ? " · " + modBlocked[m.id] + " blocked" : "") + "</span>" +
        '<span class="ig-bar">' + bar + "</span></summary><div class=\"ig-body\"></div></details>";
    }).join("") || '<div class="flow-empty">No task matches this filter.</div>';
    renderIdxTools();
    document.querySelectorAll(".idx-group[open]").forEach(fillGroup);
  }
  function fillGroup(det, showAll) {
    const body = det.querySelector(".ig-body");
    const mod = det.getAttribute("data-mod");
    const order = { active: 0, hold: 1, draft: 2, done: 3 };
    const list = TASKS.filter(t => t.m === mod && idxMatch(t))
      .sort((a, b) => (order[a.b] - order[b.b]) || (a.i < b.i ? -1 : 1));
    const shown = showAll ? list : list.slice(0, IDX.cap);
    body.innerHTML = shown.map(t =>
      '<button class="irow" data-act="task" data-id="' + esc(t.i) + '">' +
      '<span class="iid">' + esc(short(t.i)) + '</span>' +
      '<span class="ittl">' + esc(t.t) + '</span>' +
      '<span class="pill b-' + t.b + '"><i></i>' + esc(stLabel(t.s)) + "</span>" +
      '<span class="prio ' + esc(t.p) + '">' + esc((t.p || "").toUpperCase()) + "</span>" +
      '<span class="ideps">' + ((t.d || []).length ? "◂" + t.d.length : "") +
      ((REV[t.i] || []).length ? " ▸" + REV[t.i].length : "") + "</span></button>").join("") +
      (list.length > shown.length
        ? '<button class="show-more" data-act="idx-more" data-id="' + esc(mod) + '">Show ' + (list.length - shown.length) + " more</button>"
        : "");
  }

  /* ═══ 11 · drawer ══════════════════════════════════════════════════════ */
  const drawer = document.getElementById("drawer");
  const scrim = document.getElementById("scrim");
  function chipsOf(idList) {
    if (!idList || !idList.length) return '<span class="ig-meta">none</span>';
    return idList.map(id => {
      const t = BY[id];
      return '<button class="tchip" data-act="task" data-id="' + esc(id) + '" title="' +
        esc(t ? t.t + " [" + stLabel(t.s) + "]" : "not in live corpus (archived or future)") + '">' +
        '<span class="dt ' + (t ? "b-" + t.b : "ghost") + '"></span>' + esc(short(id)) + "</button>";
    }).join("");
  }
  function openTask(id) {
    const t = BY[id];
    if (!t) return;
    const unmet = unmetDeps(t);
    const meta = [
      ["status", stLabel(t.s)], ["type", t.c], ["priority", (t.p || "—").toUpperCase()],
      ["module", t.m], ["phase", t.ph || "—"], ["owner", t.o || "—"],
      ["created", t.cr || "—"], ["shipped", t.sh || "—"], ["effort", t.e ? t.e + "h" : "—"],
    ];
    drawer.innerHTML =
      '<div class="d-hd"><div><div class="d-id">' + esc(t.i) + '</div><div class="d-ttl">' + esc(t.t) + "</div>" +
      '<div style="margin-top:7px"><span class="pill b-' + t.b + '"><i></i>' + esc(stLabel(t.s)) + "</span>" +
      (CRITSET.has(id) ? ' <span class="pill" style="background:var(--accent-soft);color:var(--accent-ink)">critical path</span>' : "") +
      (FRONTSET.has(id) ? ' <span class="pill b-done"><i></i>ready to start</span>' : "") + "</div></div>" +
      '<button class="d-close" data-act="close-drawer" aria-label="Close">' + ICONS.x + "</button></div>" +
      '<div class="d-body">' +
      '<div class="d-meta">' + meta.map(m => '<div class="dm"><div class="k">' + m[0] + '</div><div class="v">' + esc(m[1]) + "</div></div>").join("") + "</div>" +
      (t.sm ? '<div class="d-sum">' + esc(t.sm) + "</div>" : "") +
      (unmet.length ? '<div class="d-warn">' + ICONS.warn + "<span><b>Blocked.</b> Waiting on " +
        unmet.map(esc).join(", ") + "</span></div>" : "") +
      '<div class="d-sec"><h4>Depends on · ' + (t.d || []).length + '</h4><div class="chips">' + chipsOf(t.d) + "</div></div>" +
      '<div class="d-sec"><h4>Unblocks · ' + (REV[t.i] || []).length + '</h4><div class="chips">' + chipsOf(REV[t.i]) + "</div></div>" +
      ((t.rl || []).length ? '<div class="d-sec"><h4>Related</h4><div class="chips">' + chipsOf(t.rl) + "</div></div>" : "") +
      '<div class="d-sec"><h4>Cited in releases · ' + (RELOF[id] || []).length + "</h4>" +
      ((RELOF[id] || []).length ? (RELOF[id] || []).map(v =>
        '<button class="d-rel" data-act="rel" data-id="' + esc(v) + '" style="width:100%;text-align:left">' +
        '<span class="h">' + esc(relLabel(v)) + "</span><span>" +
        esc((RELBYV[v] || {}).d || "") + "</span></button>").join("") : '<span class="ig-meta">not cited yet — will appear here when a release note or commit references it</span>') + "</div>" +
      '<div class="d-sec"><h4>Commits citing this task · ' + (COMOF[id] || []).length + "</h4>" +
      ((COMOF[id] || []).slice(0, 8).map(c => '<div class="d-rel">' + extA(ghc(c.h), "h", esc(c.h), "open commit on GitHub") +
        "<span>" + esc(c.s) + "</span></div>").join("") || '<span class="ig-meta">none yet</span>') + "</div>" +
      '<div class="d-sec" id="dw-spec"><h4>Spec</h4><div id="dw-spec-body"><p class="ig-meta">Loading…</p></div></div>' +
      '<div class="d-links">' +
      '<a class="btn" target="_blank" rel="noopener" href="' + esc((D.specBase || '') + t.k + "/spec.md") + '">' + ICONS.open + " Open spec</a>" +
      '<button class="btn" data-act="cone" data-id="' + esc(t.i) + '">' + ICONS.graph + " Trace in graph</button>" +
      '<button class="btn" data-act="mod" data-id="' + esc(t.m) + '">' + ICONS.graph + " Focus module " + esc(t.m) + "</button>" +
      "</div></div>";
    drawer.classList.add("open");
    scrim.classList.add("open");
    setHash("t/" + id);
    loadSpec(t);
  }
  function loadSpec(t) {
    const body = document.getElementById("dw-spec-body");
    if (!body) return;
    if (!t.sp) {
      body.innerHTML = '<p class="ig-meta">No spec chunk shipped with this page. Use Open spec for the markdown source.</p>';
      return;
    }
    const W = window.CS_SPEC || (window.CS_SPEC = {});
    function paint() {
      body.innerHTML = '<div class="spec">' + W[t.i] + "</div>";
    }
    if (W[t.i]) { paint(); return; }
    body.innerHTML = '<p class="ig-meta">Loading the full spec …</p>';
    const dir = D.specDir || "data/task";
    const sc = document.createElement("script");
    sc.src = dir + "/" + t.i + ".js";
    sc.onerror = function () {
      body.innerHTML = '<p class="ig-meta">The spec chunk did not load. Use Open spec for the markdown source.</p>';
    };
    sc.onload = function () { if (W[t.i]) paint(); else sc.onerror(); };
    document.head.appendChild(sc);
  }
  function closeDrawer() {
    if (!drawer.classList.contains("open")) return;
    drawer.classList.remove("open");
    scrim.classList.remove("open");
    /* restore the pinned selection's deep link, or clear the task link */
    setHash(S.sel && S.sel.kind !== "task" ? selHash() : "");
  }

  /* ═══ 12 · selection + routing ═════════════════════════════════════════ */
  const selslot = document.getElementById("selslot");
  function renderSelChip() {
    selslot.innerHTML = S.sel
      ? '<span class="selchip"><button data-act="sel-go" title="Jump to it">' +
        esc(S.sel.kind) + ": " + esc(S.sel.kind === "task" ? short(S.sel.id) : S.sel.id) + "</button>" +
        '<button class="x" data-act="clear-sel" aria-label="Clear selection">✕</button></span>'
      : "";
  }
  function setHash(h) {
    const target = h ? "#" + h : "";
    if (target === location.hash) return;
    S.lastSetHash = target || "#";
    location.hash = h;
  }
  function selHash() {
    if (!S.sel) return "";
    const p = { task: "t/", module: "m/", phase: "p/", release: "r/" }[S.sel.kind];
    return p + S.sel.id;
  }
  /* dim map nodes/edges to a set of module ids (phase and release selections) */
  function mapDimSet(set) {
    document.querySelectorAll("#map .mnode").forEach(g => {
      g.classList.remove("sel");
      g.classList.toggle("dim", !!set && !set.has(g.getAttribute("data-id")));
    });
    document.querySelectorAll("#map .medge").forEach(p => {
      const on = set && set.has(p.getAttribute("data-from")) && set.has(p.getAttribute("data-to"));
      p.classList.remove("hot");
      p.classList.toggle("dim", !!set && !on);
    });
  }
  function relCitedIds(v) {
    const r = v === "unreleased" ? { notes: (D.unreleased || {}).notes || {}, cov: (D.unreleased || {}).cov || {} } : RELBYV[v];
    const set = new Set();
    if (!r) return set;
    ["features", "fixes", "improvements"].forEach(k => ((r.notes || {})[k] || []).forEach(it => (it.ids || []).forEach(id => set.add(id))));
    ((r.cov || {}).commits || []).forEach(c => (c.ids || []).forEach(id => set.add(id)));
    return set;
  }
  function clearMarks() {
    document.querySelectorAll(".rel-card.sel, .slimrow.sel, .rv-chip.sel, #roadsvg .rel.sel, #roadsvg g.ph.sel, #roadsvg g.ph.dim")
      .forEach(n => n.classList.remove("sel", "dim"));
  }
  /* jump target for the selection chip and for second clicks — the ONLY places that scroll */
  function selGo() {
    if (!S.sel) return;
    if (S.sel.kind === "module") document.getElementById("sysmap").scrollIntoView({ block: "start" });
    else if (S.sel.kind === "phase" || S.sel.kind === "task") document.getElementById("flowband").scrollIntoView({ block: "start" });
    else if (S.sel.kind === "release") {
      const card = document.getElementById(S.sel.id === "unreleased" ? "rel-unreleased" : "rel-" + S.sel.id.replace(/\./g, "-"));
      if (card) {
        if (card.classList.contains("paged")) revealRels();
        const det = card.closest("details");
        if (det) det.open = true;
        card.scrollIntoView({ block: "center" });
      }
    }
  }
  function revealRels() {
    document.querySelectorAll(".rel-card.paged").forEach(c => c.classList.remove("paged"));
    const b = document.getElementById("relmore");
    if (b) b.remove();
  }
  /* selections update every band IN PLACE — nothing scrolls unless asked (selGo / deep link) */
  function selectModule(id, opts) {
    if (!MOD[id]) return;
    const again = S.sel && S.sel.kind === "module" && S.sel.id === id;
    S.sel = { kind: "module", id };
    S.flow = "sel";
    clearMarks(); mapFocus(id); renderSelChip(); renderFlow(); renderIndex();
    document.querySelectorAll(".idx-group").forEach(g => {
      if (g.getAttribute("data-mod") === id) { g.open = true; fillGroup(g); }
    });
    setHash("m/" + id);
    if ((opts && opts.scroll) || again) selGo();
  }
  function selectPhase(id, opts) {
    const again = S.sel && S.sel.kind === "phase" && S.sel.id === id;
    S.sel = { kind: "phase", id };
    S.flow = "sel";
    clearMarks(); renderSelChip(); renderFlow(); renderIndex();
    document.querySelectorAll("#roadsvg g.ph").forEach(g => {
      const on = g.getAttribute("data-phase") === id;
      g.classList.toggle("sel", on);
      g.classList.toggle("dim", !on);
    });
    mapDimSet(new Set(TASKS.filter(t => t.pg === id).map(t => t.m)));
    setHash("p/" + id);
    if ((opts && opts.scroll) || again) selGo();
  }
  function selectRelease(v, opts) {
    const again = S.sel && S.sel.kind === "release" && S.sel.id === v;
    S.sel = { kind: "release", id: v };
    clearMarks(); renderSelChip();
    document.querySelectorAll(".rv-chip").forEach(c =>
      c.classList.toggle("sel", c.getAttribute("data-id") === v));
    document.querySelectorAll("#roadsvg .rel").forEach(g =>
      g.classList.toggle("sel", (g.getAttribute("data-cluster") || "").split(",").indexOf(v) >= 0));
    const card = document.getElementById(v === "unreleased" ? "rel-unreleased" : "rel-" + v.replace(/\./g, "-"));
    if (card) card.classList.add("sel");
    const cited = relCitedIds(v);
    mapDimSet(cited.size ? new Set([...cited].filter(id => BY[id]).map(id => BY[id].m)) : null);
    if (S.flow === "sel") renderFlow(); else renderFlowTools();
    setHash("r/" + v);
    if ((opts && opts.scroll) || again) selGo();
  }
  function clearSel() {
    S.sel = null;
    if (S.flow === "sel") S.flow = "focus";
    renderSelChip(); mapFocus(null); mapDimSet(null); clearMarks(); renderFlow(); renderIndex();
    setHash("");
  }
  function applyHash() {
    if (S.lastSetHash && (location.hash === S.lastSetHash || (S.lastSetHash === "#" && !location.hash))) {
      S.lastSetHash = null;
      return;
    }
    S.lastSetHash = null;
    const h = decodeURIComponent(location.hash.replace(/^#/, ""));
    if (!h) return;
    /* status-hub@2 bookmarks stay alive */
    const LEGACY_BAND = {
      board: "indexband", table: "indexband", timeline: "ledger",
      roadmap: "roadmap", backlog: "indexband", changelog: "ledger",
    };
    if (LEGACY_BAND[h]) {
      const el = document.getElementById(LEGACY_BAND[h]);
      if (el) el.scrollIntoView({ block: "start" });
      return;
    }
    if (h.indexOf("task/") === 0 && BY[h.slice(5)]) { openTask(h.slice(5)); return; }
    const kind = h.slice(0, 2), id = h.slice(2);
    if (kind === "t/" && BY[id]) openTask(id);
    else if (kind === "m/") selectModule(id, { scroll: true });
    else if (kind === "p/") selectPhase(id, { scroll: true });
    else if (kind === "r/") selectRelease(id, { scroll: true });
  }

  /* ═══ 13 · search ══════════════════════════════════════════════════════ */
  const qEl = document.getElementById("q");
  const drop = document.getElementById("srdrop");
  let srIdx = 0, srItems = [];
  function doSearch(q) {
    q = q.trim().toLowerCase();
    if (!q) { drop.hidden = true; return; }
    const hits = [];
    const push = (kind, id, t, score) => hits.push({ kind, id, t, score });
    TASKS.forEach(t => {
      const idl = t.i.toLowerCase(), tl = t.t.toLowerCase();
      if (idl.includes(q)) push("task", t.i, t.t, idl.indexOf(q) === 0 ? 0 : idl.startsWith("task-" + q) ? 1 : 3);
      else if (tl.includes(q)) push("task", t.i, t.t, 5 + tl.indexOf(q) / 200);
    });
    MODS.forEach(m => { if (m.id.toLowerCase().includes(q)) push("module", m.id, m.total + " tasks", 2); });
    RELS.forEach(r => { if (("v" + r.v).includes(q) || r.v.includes(q)) push("release", r.v, r.d || "", 2); });
    hits.sort((a, b) => a.score - b.score);
    srItems = hits.slice(0, 10);
    srIdx = 0;
    drop.hidden = false;
    drop.innerHTML = srItems.length ? srItems.map((h, i) =>
      '<button class="sr-item' + (i === srIdx ? " on" : "") + '" data-sri="' + i + '">' +
      '<span class="sr-kind">' + h.kind + '</span><span class="sr-id">' +
      esc(h.kind === "release" ? "v" + h.id : h.id) + '</span><span class="sr-t">' + esc(h.t) + "</span></button>").join("")
      : '<div class="sr-empty">No match for "' + esc(q) + '"</div>';
  }
  function srGo(i) {
    const h = srItems[i];
    if (!h) return;
    drop.hidden = true; qEl.blur();
    if (h.kind === "task") openTask(h.id);
    else if (h.kind === "module") selectModule(h.id, { scroll: true });
    else selectRelease(h.id, { scroll: true });
  }
  qEl.addEventListener("input", () => doSearch(qEl.value));
  qEl.addEventListener("keydown", e => {
    if (drop.hidden) return;
    if (e.key === "ArrowDown") { srIdx = Math.min(srItems.length - 1, srIdx + 1); }
    else if (e.key === "ArrowUp") { srIdx = Math.max(0, srIdx - 1); }
    else if (e.key === "Enter") { srGo(srIdx); e.preventDefault(); return; }
    else if (e.key === "Escape") { drop.hidden = true; return; }
    else return;
    e.preventDefault();
    drop.querySelectorAll(".sr-item").forEach((n, i) => n.classList.toggle("on", i === srIdx));
  });
  document.addEventListener("keydown", e => {
    if (e.key === "/" && document.activeElement !== qEl && !/input|textarea/i.test(document.activeElement.tagName)) {
      e.preventDefault(); qEl.focus(); qEl.select();
    }
    if (e.key === "Escape") {
      if (!drop.hidden) { drop.hidden = true; return; }
      if (drawer.classList.contains("open")) { closeDrawer(); return; }
      if (S.sel) clearSel();
    }
  });
  document.addEventListener("click", e => {
    if (!e.target.closest(".search")) drop.hidden = true;
  });

  /* ═══ 14 · theme ═══════════════════════════════════════════════════════ */
  function applyTheme(th) {
    document.body.setAttribute("data-theme", th);
    document.querySelector(".theme-btn").innerHTML = th === "paper" ? ICONS.moon : ICONS.sun;
    try { localStorage.setItem("sv3-theme", th); } catch (err) { /* file:// may deny */ }
  }
  try { const saved = localStorage.getItem("sv3-theme"); if (saved) applyTheme(saved); } catch (err) { /* ignore */ }
  /* shareable override: index.html?theme=night */
  try {
    const th = new URLSearchParams(location.search).get("theme");
    if (th === "night" || th === "paper") applyTheme(th);
  } catch (err) { /* ignore */ }

  /* ═══ 15 · delegated events ════════════════════════════════════════════ */
  document.addEventListener("click", e => {
    const sri = e.target.closest("[data-sri]");
    if (sri) { srGo(+sri.getAttribute("data-sri")); return; }
    const el = e.target.closest("[data-act]");
    if (!el) return;
    const act = el.getAttribute("data-act"), id = el.getAttribute("data-id");
    if (act === "task") openTask(id);
    else if (act === "mod") { closeDrawer(); selectModule(id); }
    else if (act === "rel") selectRelease(id);
    else if (act === "phase") selectPhase(id);
    else if (act === "clear-sel") clearSel();
    else if (act === "close-drawer") closeDrawer();
    else if (act === "theme") applyTheme(document.body.getAttribute("data-theme") === "paper" ? "night" : "paper");
    else if (act === "go") { const b = document.getElementById(id); if (b) b.scrollIntoView({ block: "start" }); }
    else if (act === "flow-mode") {
      /* scope chips inside band 04 — change the graph in place, never move the page */
      S.flow = id === "sel" && !S.sel ? "focus" : id;
      renderFlow();
    }
    else if (act === "go-flow") {
      /* explicit "take me to the graph" affordances (vitals, attention cards) */
      S.flow = id || "focus";
      renderFlow();
      document.getElementById("flowband").scrollIntoView({ block: "start" });
    }
    else if (act === "flow-done") { S.flowDone = !S.flowDone; renderFlow(); }
    else if (act === "sel-go") selGo();
    else if (act === "show-rels") revealRels();
    else if (act === "idx-filter") { IDX.f = id; renderIndex(); }
    else if (act === "idx-more") {
      const det = document.querySelector('.idx-group[data-mod="' + id + '"]');
      if (det) fillGroup(det, true);
    }
    else if (act === "cone") {
      closeDrawer();
      S.sel = { kind: "task", id };
      S.flow = "sel";
      clearMarks(); renderSelChip(); renderFlow();
      mapDimSet(new Set(flowScopeIds().map(x => (BY[x] || {}).m).filter(Boolean)));
      document.getElementById("flowband").scrollIntoView({ block: "start" });
    }
  });

  /* details lazy-fill */
  document.addEventListener("toggle", e => {
    const det = e.target;
    if (det.classList && det.classList.contains("idx-group") && det.open) fillGroup(det);
    if (det.classList && det.classList.contains("commits") && det.open)
      renderCommitRows(det.querySelector(".cbody"), det.getAttribute("data-ck"));
  }, true);

  /* hover: tooltips + cross-highlighting */
  document.addEventListener("mouseover", e => {
    const mn = e.target.closest("#map .mnode");
    if (mn) {
      const m = MOD[mn.getAttribute("data-id")];
      if (m) {
        mapFocus(m.id);
        tipShow("<b>" + esc(m.id) + "</b><div class='m'>" + m.total + " tasks — " + (m.done || 0) + " done · " +
          (m.active || 0) + " active · " + (m.draft || 0) + " draft · " + (m.hold || 0) + " hold" +
          ((modBlocked[m.id] || 0) ? "<br><span style='color:var(--bad)'>" + modBlocked[m.id] + " blocked</span>" : "") +
          (m.kind ? "<br><span class='mono'>" + esc(m.kind) + "</span>" : "") + "</div>", e.clientX, e.clientY);
      }
      return;
    }
    const me = e.target.closest("#map .medge");
    if (me) {
      const ei = +me.getAttribute("data-ei"), ed = (D.medges || [])[ei];
      if (ed) tipShow("<b>" + esc(ed.from) + " → " + esc(ed.to) + "</b><div class='m'>" + ed.w +
        " dependency link" + (ed.w > 1 ? "s" : "") + "<br>" +
        ed.pairs.map(p => "<span class='mono'>" + esc(p) + "</span>").join("<br>") + "</div>", e.clientX, e.clientY);
      return;
    }
    const fn = e.target.closest("#flow .fnode");
    if (fn) {
      const t = BY[fn.getAttribute("data-id")];
      if (t) {
        flowCone(t.i, true);
        tipShow("<b>" + esc(t.i) + "</b><div class='m'>" + esc(t.t) + "<br>" +
          esc(stLabel(t.s)) + " · " + esc(t.m) + (t.p ? " · " + esc(t.p) : "") +
          ((t.d || []).length ? "<br>◂ depends on " + t.d.length : "") +
          ((REV[t.i] || []).length ? " · unblocks " + REV[t.i].length + " ▸" : "") + "</div>", e.clientX, e.clientY);
      }
      return;
    }
    const rl = e.target.closest("#roadsvg .rel");
    if (rl) {
      const vs = (rl.getAttribute("data-cluster") || rl.getAttribute("data-id")).split(",");
      const lines = vs.map(v => {
        const r = RELBYV[v];
        return r ? "<span class='mono'>v" + esc(r.v) + "</span> " + esc(r.d || "") + " — " +
          r.notes.features.length + "F · " + r.notes.fixes.length + "X · " + r.notes.improvements.length + "I" +
          (r.cov.unlinked ? " · <span style='color:var(--bad)'>" + r.cov.unlinked + " unlinked</span>" : "") : "";
      });
      tipShow("<b>" + vs.length + " release" + (vs.length > 1 ? "s" : "") + "</b><div class='m'>" +
        lines.join("<br>") + "</div>", e.clientX, e.clientY);
      return;
    }
    const ph = e.target.closest("#roadsvg g.ph");
    if (ph) {
      const p = (D.phases || []).find(x => x.id === ph.getAttribute("data-phase"));
      if (p) tipShow("<b>" + esc(p.label) + "</b><div class='m'>" + p.done + " of " + p.total + " done" +
        (p.start ? "<br>since " + esc(p.start) : "") + (p.lastShip ? " · last ship " + esc(p.lastShip) : "") +
        "<br>click to scope the flow graph</div>", e.clientX, e.clientY);
    }
  });
  document.addEventListener("mouseout", e => {
    if (e.target.closest("#map .mnode") && !e.relatedTarget?.closest?.("#map .mnode")) {
      tipHide();
      mapFocus(S.sel && S.sel.kind === "module" ? S.sel.id : null);
    }
    if (e.target.closest("#flow .fnode") && !e.relatedTarget?.closest?.("#flow .fnode")) {
      tipHide(); flowCone(null, false);
    }
    if (e.target.closest("#map .medge, #roadsvg .rel, #roadsvg g.ph")) tipHide();
  });
  document.addEventListener("mousemove", e => {
    if (tip.classList.contains("on")) {
      const w = tip.offsetWidth, h = tip.offsetHeight;
      let tx = e.clientX + 14, ty = e.clientY + 14;
      if (tx + w > innerWidth - 8) tx = e.clientX - w - 12;
      if (ty + h > innerHeight - 8) ty = e.clientY - h - 12;
      tip.style.left = Math.max(6, tx) + "px";
      tip.style.top = Math.max(6, ty) + "px";
    }
  });
  addEventListener("hashchange", applyHash);

  /* ═══ 16 · boot ════════════════════════════════════════════════════════ */
  renderFlow();
  renderIndex();
  applyHash();
})();
