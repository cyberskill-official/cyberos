/* status-app.js - client for status-hub@2.
   One corpus, three lenses (board / table / releases) + an FR detail drawer.
   No dependencies and no network calls except the on-demand per-FR spec chunk
   (data/fr/<ID>.js - a classic script, so it works over file:// too).
   The page stays readable without this file: <noscript> carries a static table
   of every FR plus the full release list. */
(function () {
  "use strict";

  var el = document.getElementById("cs-data");
  if (!el) return;
  var D = JSON.parse(el.textContent);
  var FRS = D.frs;
  var BY = {};
  FRS.forEach(function (f) { BY[f.i] = f; });

  var ACTIVE = { ready_to_implement: 1, implementing: 1, ready_to_review: 1,
                 reviewing: 1, ready_to_test: 1, testing: 1 };
  var DONE = { done: 1, closed: 1 };
  function bucket(s) {
    return DONE[s] ? "done" : ACTIVE[s] ? "active" : s === "on_hold" ? "hold" : "todo";
  }

  var LENSES = { board: 1, table: 1, timeline: 1 };
  var LEGACY = { roadmap: "board", backlog: "table", changelog: "timeline" };
  var FKEYS = ["m", "s", "p", "c", "ph", "b"];
  var FFIELD = { m: "m", s: "s", p: "p", c: "c", ph: "ph" };
  var FLABEL = { m: "module", s: "status", p: "priority", c: "class", ph: "phase", b: "group" };

  var S = { lens: "board", q: "", f: {}, group: "m", sort: "i", dir: 1, fr: null };

  /* ---- hash state ------------------------------------------------------- */
  function readHash() {
    var h = decodeURIComponent(location.hash.replace(/^#/, ""));
    if (!h) return;
    var parts = h.split("?");
    var path = parts[0];
    var qs = parts[1] || "";
    if (path.indexOf("fr/") === 0) S.fr = path.slice(3);
    else if (LENSES[path]) S.lens = path;
    else if (LEGACY[path]) S.lens = LEGACY[path];
    qs.split("&").forEach(function (kv) {
      if (!kv) return;
      var i = kv.indexOf("=");
      var k = i < 0 ? kv : kv.slice(0, i);
      var v = i < 0 ? "" : decodeURIComponent(kv.slice(i + 1));
      if (k === "q") S.q = v;
      else if (k === "g") S.group = v;
      else if (FKEYS.indexOf(k) >= 0 && v) S.f[k] = v;
    });
  }
  function writeHash(replace) {
    var q = [];
    if (S.q) q.push("q=" + encodeURIComponent(S.q));
    FKEYS.forEach(function (k) { if (S.f[k]) q.push(k + "=" + encodeURIComponent(S.f[k])); });
    if (S.lens === "board" && S.group !== "m") q.push("g=" + S.group);
    var h = "#" + (S.fr ? "fr/" + S.fr : S.lens) + (q.length ? "?" + q.join("&") : "");
    if (h === location.hash) return;
    if (replace && history.replaceState) history.replaceState(null, "", h);
    else if (history.pushState) history.pushState(null, "", h);
    else location.hash = h;
  }

  /* ---- filtering -------------------------------------------------------- */
  function matches(f) {
    for (var k in FFIELD) if (S.f[k] && f[FFIELD[k]] !== S.f[k]) return false;
    if (S.f.b && bucket(f.s) !== S.f.b) return false;
    if (S.q) {
      var q = S.q.toLowerCase();
      var hay = (f.i + " " + f.t + " " + f.m + " " + (f.o || "") + " " +
                 (f.ph || "") + " " + (f.sm || "")).toLowerCase();
      if (hay.indexOf(q) < 0) return false;
    }
    return true;
  }
  function view() { return FRS.filter(matches); }

  /* ---- helpers ---------------------------------------------------------- */
  function esc(s) {
    return String(s == null ? "" : s).replace(/&/g, "&amp;").replace(/</g, "&lt;")
      .replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }
  function pct(n, d) { return d ? (100 * n / d).toFixed(1) : "0.0"; }
  function chip(f, ghost) {
    if (!f) return "";
    return '<button class="chip ' + bucket(f.s) + (ghost ? " ghost" : "") +
      '" data-fr="' + esc(f.i) + '" title="' + esc(f.i + " — " + f.t + " [" + f.s + "]") +
      '">' + esc(f.i.replace(/^FR-/, "")) + "</button>";
  }
  function chipId(id, ghost) {
    return BY[id] ? chip(BY[id], ghost)
      : '<span class="chip ghost" title="not in this corpus">' + esc(id.replace(/^FR-/, "")) + "</span>";
  }
  function segs(rows) {
    var d = 0, a = 0, h = 0;
    rows.forEach(function (f) {
      var b = bucket(f.s);
      if (b === "done") d++; else if (b === "active") a++; else if (b === "hold") h++;
    });
    var n = rows.length;
    return '<div class="minibar"><i class="seg-done" style="width:' + pct(d, n) + '%"></i>' +
      '<i class="seg-active" style="width:' + pct(a, n) + '%"></i>' +
      '<i class="seg-hold" style="width:' + pct(h, n) + '%"></i></div>';
  }

  /* ---- lens: board ------------------------------------------------------ */
  function board(rows) {
    var g = S.group, keys = [], bag = {};
    rows.forEach(function (f) {
      var k = f[g] || "(unset)";
      if (!bag[k]) { bag[k] = []; keys.push(k); }
      bag[k].push(f);
    });
    if (g === "s") keys.sort(function (a, b) { return D.statuses.indexOf(a) - D.statuses.indexOf(b); });
    else keys.sort(function (a, b) {
      return bag[b].length - bag[a].length || String(a).localeCompare(String(b));
    });
    if (!keys.length) return '<p class="empty">No feature request matches these filters.</p>';
    return '<div class="grid">' + keys.map(function (k) {
      var rs = bag[k];
      var d = rs.filter(function (f) { return bucket(f.s) === "done"; }).length;
      var a = rs.filter(function (f) { return bucket(f.s) === "active"; }).length;
      return '<section class="card' + (a ? " hot" : "") + '">' +
        '<div class="card-h"><h3><span class="k">' + esc(k) + '</span> ' +
        '<span class="muted">· ' + rs.length + " FRs</span></h3>" +
        '<span class="pct">' + pct(d, rs.length) + "% done</span></div>" +
        segs(rs) + '<div class="frs">' +
        rs.map(function (f) { return chip(f); }).join("") + "</div></section>";
    }).join("") + "</div>";
  }

  /* ---- lens: table ------------------------------------------------------ */
  var COLS = [
    { k: "i", h: "id" }, { k: "t", h: "title" }, { k: "m", h: "module" },
    { k: "c", h: "class" }, { k: "p", h: "priority" }, { k: "ph", h: "phase" },
    { k: "e", h: "effort" }, { k: "s", h: "status" }
  ];
  function table(rows) {
    var k = S.sort, dir = S.dir;
    var sorted = rows.slice().sort(function (a, b) {
      var x = a[k], y = b[k];
      if (k === "s") { x = D.statuses.indexOf(x); y = D.statuses.indexOf(y); }
      else if (k === "e") { x = +x || 0; y = +y || 0; }
      else { x = x == null ? "" : x; y = y == null ? "" : y; }
      return (x > y ? 1 : x < y ? -1 : 0) * dir || String(a.i).localeCompare(String(b.i));
    });
    if (!sorted.length) return '<p class="empty">No feature request matches these filters.</p>';
    var head = COLS.map(function (c) {
      var s = S.sort === c.k ? (S.dir > 0 ? "ascending" : "descending") : "none";
      return '<th data-sort="' + c.k + '" aria-sort="' + s + '" scope="col">' + c.h + "</th>";
    }).join("");
    var body = sorted.map(function (f) {
      return '<tr data-fr="' + esc(f.i) + '" tabindex="0">' +
        '<td class="code">' + esc(f.i) + "</td>" +
        '<td class="t">' + esc(f.t) + "</td>" +
        "<td>" + esc(f.m) + "</td><td>" + esc(f.c || "") + "</td>" +
        '<td class="pri ' + esc(f.p || "") + '">' + esc(f.p || "") + "</td>" +
        "<td>" + esc(f.ph || "") + "</td><td>" + esc(f.e || "") + "</td>" +
        '<td><span class="pill ' + bucket(f.s) + '">' + esc(f.s) + "</span></td></tr>";
    }).join("");
    return '<div class="tbl-wrap"><table><thead><tr>' + head +
      "</tr></thead><tbody>" + body + "</tbody></table></div>";
  }

  /* ---- lens: releases --------------------------------------------------- */
  var BOUND = {};
  (D.bound || []).forEach(function (id) { BOUND[id] = 1; });

  function timeline(rows) {
    var seen = {};
    rows.forEach(function (f) { seen[f.i] = 1; });
    // "unreleased" = in flight, plus anything already done that no release accounts for
    var moving = rows.filter(function (f) {
      var b = bucket(f.s);
      return b === "active" || (b === "done" && !BOUND[f.i]);
    });
    var out = [];
    if (moving.length) {
      var shipped = moving.filter(function (f) { return bucket(f.s) === "done"; }).length;
      out.push('<article class="rel now"><span class="tick">★</span><div>' +
        '<div class="rel-h"><b>unreleased</b><span class="muted">' + moving.length + " FRs</span></div>" +
        '<p class="relnote">No release accounts for these yet — ' +
        (moving.length - shipped) + " in flight (<code>ready_to_implement</code> → <code>testing</code>)" +
        (shipped ? ", " + shipped + " already <code>done</code> but not cut into a release" : "") +
        ".</p>" +
        '<div class="frs">' + moving.map(function (f) { return chip(f); }).join("") +
        "</div></div></article>");
    }
    D.releases.forEach(function (r, i) {
      var cited = r.cited.filter(function (id) { return !BY[id] || seen[id]; });
      var dated = r.dated.filter(function (id) { return seen[id]; });
      var secs = r.sec.map(function (s) {
        return '<div class="rel-sec"><h4>' + esc(s.h) + "</h4><ul>" +
          s.items.map(function (x) { return "<li>" + x + "</li>"; }).join("") + "</ul></div>";
      }).join("");
      var chips = cited.map(function (id) { return chipId(id); }).join("") +
        dated.map(function (id) { return chipId(id, true); }).join("");
      out.push('<article class="rel' + (i === 0 && !moving.length ? " now" : "") +
        '"><span class="tick">' + (i === 0 ? "★" : "✓") + "</span><div>" +
        '<div class="rel-h"><b>' + esc(r.vl) + "</b>" +
        (r.d ? '<span class="muted">' + esc(r.d) + "</span>" : "") +
        '<span class="muted">' + (cited.length + dated.length) + " FRs</span></div>" +
        (chips ? '<div class="frs">' + chips + "</div>"
               : '<p class="relnote">No FR id cited in this entry.</p>') +
        (dated.length ? '<p class="relnote">Dashed chips were bound by <code>shipped:</code> date, ' +
          "not cited in the entry text.</p>" : "") +
        (r.intro.length || secs
          ? "<details><summary>Release notes</summary>" +
            r.intro.map(function (p) { return "<p>" + p + "</p>"; }).join("") + secs + "</details>"
          : "") +
        "</div></article>");
    });
    return '<div class="rels">' + out.join("") + "</div>";
  }

  /* ---- drawer ----------------------------------------------------------- */
  var drawer = document.getElementById("drawer");
  var scrim = document.getElementById("scrim");
  var dwTab = "overview";
  var lastFocus = null;

  // the markdown is only reachable when it ships next to the page (docs/status/ does; the
  // website build does not - there the rendered FR page is the way in)
  function specHref(f) { return D.frBase ? D.frBase + f.dm + "/" + f.k + "/spec.md" : ""; }
  function srcLink(f, label) {
    var h = specHref(f);
    return h ? '<a href="' + esc(h) + '">' + label + "</a>"
      : (f.pg ? '<a href="' + esc(f.pg) + '">the FR page</a>' : "");
  }
  function row(dt, dd) { return dd ? "<div><dt>" + esc(dt) + "</dt><dd>" + dd + "</dd></div>" : ""; }

  function drawerHtml(f) {
    var n = f.d.length + f.b.length + f.rl.length;
    var tabs = [["overview", "Overview"], ["spec", "Full spec"], ["links", "Links (" + n + ")"]];
    return '<div class="dw-h"><div class="row">' +
      '<span class="pill ' + bucket(f.s) + '">' + esc(f.s) + "</span>" +
      '<span class="code muted">' + esc(f.i) + "</span>" +
      '<span class="pri ' + esc(f.p || "") + '">' + esc(f.p || "") + "</span>" +
      '<button class="dw-x" type="button" aria-label="Close">×</button></div>' +
      '<h2 id="dw-title">' + esc(f.t) + "</h2></div>" +
      '<div class="dw-tabs" role="tablist">' + tabs.map(function (t) {
        return '<button class="dw-tab" role="tab" type="button" data-dwtab="' + t[0] +
          '" aria-selected="' + (dwTab === t[0]) + '">' + t[1] + "</button>";
      }).join("") + "</div><div class=\"dw-b\" id=\"dw-body\"></div>";
  }

  function overview(f) {
    var m = '<dl class="meta">' +
      row("module", esc(f.m)) + row("class", esc(f.c)) + row("phase", esc(f.ph)) +
      row("milestone", esc(f.ms)) + row("slice", esc(f.sl)) + row("owner", esc(f.o)) +
      row("effort", f.e ? esc(f.e) + " h" : "") + row("verify", esc(f.v)) +
      row("created", esc(f.cr)) + row("shipped", esc(f.sh)) + "</dl>";
    var s = "";
    if (f.sm) s += '<div class="sect"><h3>Summary</h3><p>' + esc(f.sm) + "</p></div>";
    if (f.st && f.st.length) {
      s += '<div class="sect"><h3>Sub-tasks (' + f.st.length + ')</h3><ul class="tasks">' +
        f.st.map(function (t) { return "<li>" + esc(t) + "</li>"; }).join("") + "</ul></div>";
    }
    if (f.r) s += '<div class="sect"><h3>Risk if skipped</h3><p class="risk">' + esc(f.r) + "</p></div>";
    return m + s;
  }

  function links(f) {
    function blk(title, ids, none) {
      return "<div><h3>" + esc(title) + " (" + ids.length + ")</h3>" +
        (ids.length ? '<div class="frs">' + ids.map(function (id) { return chipId(id); }).join("") + "</div>"
                    : '<p class="muted">' + esc(none) + "</p>") + "</div>";
    }
    var srcs = [];
    if (f.pg) srcs.push('<a href="' + esc(f.pg) + '">Rendered FR page</a>');
    if (specHref(f)) srcs.push('<a href="' + esc(specHref(f)) + '"><code>spec.md</code> — the record of truth</a>');
    if (!srcs.length) srcs.push('<span class="muted">Markdown is the record of truth: <code>' +
      esc("docs/feature-requests/" + f.dm + "/" + f.k + "/spec.md") + "</code></span>");
    return '<div class="rel-links">' +
      blk("Depends on", f.d, "Nothing blocks this FR.") +
      blk("Blocks", f.b, "This FR blocks nothing.") +
      blk("Related", f.rl, "No related FRs recorded.") + "</div>" +
      '<div class="sect"><h3>Sources</h3><p>' + srcs.join(" · ") + "</p></div>";
  }

  function specTab(f) {
    var body = document.getElementById("dw-body");
    if (!f.sp) {
      body.innerHTML = '<p class="load">No spec body was rendered into this page. ' +
        srcLink(f, "Open spec.md") + "</p>";
      return;
    }
    var W = window.CS_SPEC || (window.CS_SPEC = {});
    function paintSpec() {
      var tmp = document.createElement("div");
      tmp.innerHTML = W[f.i];
      var toc = [];
      Array.prototype.forEach.call(tmp.querySelectorAll("h2"), function (h, n) {
        var id = "s" + n;
        h.id = id;
        toc.push('<a href="#' + id + '" data-anchor="' + id + '">' + esc(h.textContent) + "</a>");
      });
      body.innerHTML = (toc.length ? '<nav class="toc">' + toc.join("") + "</nav>" : "") +
        '<div class="spec">' + tmp.innerHTML + "</div>";
    }
    if (W[f.i]) { paintSpec(); return; }
    body.innerHTML = '<p class="load">Loading the full spec …</p>';
    var sc = document.createElement("script");
    sc.src = D.specDir + "/" + f.i + ".js";
    sc.onerror = function () {
      body.innerHTML = '<p class="load">The spec chunk did not load. ' + srcLink(f, "Open spec.md") + "</p>";
    };
    sc.onload = function () { if (W[f.i]) paintSpec(); else sc.onerror(); };
    document.head.appendChild(sc);
  }

  function paintDrawer(f) {
    if (dwTab === "spec") specTab(f);
    else document.getElementById("dw-body").innerHTML = dwTab === "links" ? links(f) : overview(f);
  }
  function openFR(id, push) {
    var f = BY[id];
    if (!f) return;
    if (!S.fr) lastFocus = document.activeElement;
    S.fr = id;
    drawer.innerHTML = drawerHtml(f);
    drawer.hidden = false;
    scrim.hidden = false;
    document.body.style.overflow = "hidden";
    paintDrawer(f);
    drawer.querySelector(".dw-x").focus();
    if (push !== false) writeHash();
  }
  function closeFR() {
    if (!S.fr) return;
    S.fr = null;
    dwTab = "overview";
    drawer.hidden = true;
    scrim.hidden = true;
    drawer.innerHTML = "";
    document.body.style.overflow = "";
    writeHash();
    if (lastFocus && lastFocus.focus) lastFocus.focus();
  }

  /* ---- paint ------------------------------------------------------------ */
  var lensEl = document.getElementById("lens");
  var countEl = document.getElementById("count");
  var chipbar = document.getElementById("active-filters");

  function paint() {
    var rows = view();
    countEl.textContent = rows.length === FRS.length
      ? FRS.length + " feature requests"
      : "showing " + rows.length + " of " + FRS.length;
    lensEl.innerHTML = S.lens === "table" ? table(rows)
      : S.lens === "timeline" ? timeline(rows) : board(rows);

    Array.prototype.forEach.call(document.querySelectorAll(".ln"), function (b) {
      b.setAttribute("aria-selected", String(b.dataset.lens === S.lens));
    });
    var g = document.getElementById("f-g");
    if (g) g.parentNode.hidden = S.lens !== "board";

    var on = [];
    FKEYS.forEach(function (k) {
      if (S.f[k]) on.push('<span class="fchip">' + esc(FLABEL[k]) + ": " + esc(S.f[k]) +
        '<button type="button" data-drop="' + k + '" aria-label="Remove filter">×</button></span>');
    });
    if (S.q) on.push('<span class="fchip">search: ' + esc(S.q) +
      '<button type="button" data-drop="q" aria-label="Clear search">×</button></span>');
    chipbar.innerHTML = on.join("");
    chipbar.hidden = !on.length;

    Array.prototype.forEach.call(document.querySelectorAll(".kpi"), function (k) {
      k.dataset.on = S.f.b && S.f.b === k.dataset.bucket ? "1" : "0";
    });
  }

  function syncControls() {
    var q = document.getElementById("q");
    if (q) q.value = S.q;
    FKEYS.forEach(function (k) {
      var s = document.getElementById("f-" + k);
      if (s) s.value = S.f[k] || "";
    });
    var g = document.getElementById("f-g");
    if (g) g.value = S.group;
  }

  /* ---- events ----------------------------------------------------------- */
  document.addEventListener("click", function (e) {
    var t = e.target;
    if (!t.closest) return;

    if (t === scrim || t.closest(".dw-x")) { closeFR(); return; }

    var dt = t.closest("[data-dwtab]");
    if (dt && S.fr) {
      dwTab = dt.dataset.dwtab;
      Array.prototype.forEach.call(drawer.querySelectorAll(".dw-tab"), function (b) {
        b.setAttribute("aria-selected", String(b.dataset.dwtab === dwTab));
      });
      paintDrawer(BY[S.fr]);
      return;
    }

    var anchor = t.closest("[data-anchor]");
    if (anchor) {
      e.preventDefault();
      var target = document.getElementById(anchor.dataset.anchor);
      if (target) target.scrollIntoView({ block: "start" });
      return;
    }

    var fr = t.closest("[data-fr]");
    if (fr) { openFR(fr.dataset.fr); return; }

    var ln = t.closest(".ln");
    if (ln) { S.lens = ln.dataset.lens; writeHash(); paint(); return; }

    var drop = t.closest("[data-drop]");
    if (drop) {
      var dk = drop.dataset.drop;
      if (dk === "q") S.q = ""; else delete S.f[dk];
      syncControls(); writeHash(); paint();
      return;
    }

    var kpi = t.closest(".kpi");
    if (kpi && kpi.dataset.bucket) {
      if (S.f.b === kpi.dataset.bucket) delete S.f.b; else S.f.b = kpi.dataset.bucket;
      writeHash(); paint();
      return;
    }

    var th = t.closest("th[data-sort]");
    if (th) {
      if (S.sort === th.dataset.sort) S.dir = -S.dir;
      else { S.sort = th.dataset.sort; S.dir = 1; }
      paint();
      return;
    }

    if (t.closest("#clear")) { S.q = ""; S.f = {}; syncControls(); writeHash(); paint(); }
  });

  document.addEventListener("keydown", function (e) {
    if (e.key === "Escape" && S.fr) { closeFR(); return; }
    if (e.key === "/" && document.activeElement.tagName !== "INPUT") {
      e.preventDefault();
      document.getElementById("q").focus();
      return;
    }
    if (e.key === "Enter" && e.target.dataset && e.target.dataset.fr) openFR(e.target.dataset.fr);
  });

  var qEl = document.getElementById("q");
  var timer;
  qEl.addEventListener("input", function () {
    clearTimeout(timer);
    timer = setTimeout(function () { S.q = qEl.value.trim(); writeHash(true); paint(); }, 120);
  });

  Array.prototype.forEach.call(document.querySelectorAll("#bar select"), function (s) {
    s.addEventListener("change", function () {
      var k = s.id.slice(2);
      if (k === "g") S.group = s.value;
      else if (s.value) S.f[k] = s.value;
      else delete S.f[k];
      writeHash();
      paint();
    });
  });

  document.getElementById("theme").addEventListener("click", function () {
    var next = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
    document.documentElement.dataset.theme = next;
    try { window.localStorage.setItem("cs-theme", next); } catch (err) { /* private mode */ }
  });

  window.addEventListener("hashchange", function () {
    var was = S.fr;
    S.fr = null; S.q = ""; S.f = {};
    readHash();
    syncControls();
    if (S.fr) { openFR(S.fr, false); return; }
    if (was) closeFR();
    paint();
  });

  /* ---- boot ------------------------------------------------------------- */
  try {
    var saved = window.localStorage.getItem("cs-theme");
    if (saved) document.documentElement.dataset.theme = saved;
  } catch (err) { /* private mode */ }

  readHash();
  syncControls();
  paint();
  if (S.fr) openFR(S.fr, false);
})();
