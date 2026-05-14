/* ============================================================
   CyberOS Documentation — Shared Scripts
   Loaded as type="module" so we can pull Mermaid via ES modules.
   No build step required.
   ============================================================ */

import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@11.4.1/dist/mermaid.esm.min.mjs';

/* ---------- Mermaid theme — CyberSkill Design System (Umber + Ochre) ---------- */
mermaid.initialize({
  startOnLoad: false,
  theme: 'base',
  themeVariables: {
    fontFamily: '"Be Vietnam Pro", Inter, system-ui, sans-serif',
    primaryColor:        '#f5ede6',   // umber-50
    primaryTextColor:    '#45210e',   // umber-500 (anchor)
    primaryBorderColor:  '#45210e',
    lineColor:           '#6f665d',   // neutral-500
    secondaryColor:      '#fef6e0',   // ochre-50
    tertiaryColor:       '#f0eee9',   // neutral-100
    background:          '#ffffff',
    mainBkg:             '#ffffff',
    secondBkg:           '#f5ede6',
    tertiaryBkg:         '#fef6e0',
    altBackground:       '#f0eee9',
    nodeBkg:             '#f5ede6',
    nodeBorder:          '#45210e',
    edgeLabelBackground: '#ffffff',
    actorBkg:            '#f5ede6',
    actorBorder:         '#45210e',
    signalColor:         '#6f665d',
    signalTextColor:     '#1d0c05',
    labelBoxBkgColor:    '#fef6e0',
    labelBoxBorderColor: '#f4ba17',
  },
  flowchart: { htmlLabels: true, curve: 'basis' },
  sequence:  { actorMargin: 50, boxMargin: 10 },
});

window.__mermaid = mermaid;

async function renderMermaid() {
  const nodes = document.querySelectorAll('.mermaid:not([data-processed="true"])');
  if (nodes.length) {
    try { await mermaid.run({ nodes }); }
    catch (e) { console.warn('mermaid render error', e); }
  }
}

/* ---------- Path helper — figure out where /assets/ lives from current page ----------
   Pages at root reference `assets/...`; pages in subdirs reference `../assets/...`.
   We detect that and use it for the shared nav.html fetch. */
function assetsBase() {
  // The currently-executing script's src lets us find /assets/
  const me = document.currentScript || Array.from(document.getElementsByTagName('script'))
    .find(s => s.src && s.src.endsWith('scripts.js'));
  if (me && me.src) {
    return me.src.replace(/scripts\.js(\?.*)?$/, '');
  }
  // Fallback: detect from current path
  const p = location.pathname;
  if (p.includes('/modules/') || p.includes('/architecture/') || p.includes('/reference/')) {
    return '../assets/';
  }
  return 'assets/';
}

function rootBase() {
  // The directory where index.html lives, relative to current page.
  const p = location.pathname;
  if (p.includes('/modules/') || p.includes('/architecture/') || p.includes('/reference/')) {
    return '../';
  }
  return './';
}

/* ---------- Shared nav loader ----------
   Every page has <div id="shared-nav"></div>; we inject assets/nav.html into it. */
async function loadSharedNav() {
  const slot = document.getElementById('shared-nav');
  if (!slot) return;
  const base = assetsBase();
  const root = rootBase();
  try {
    const res = await fetch(base + 'nav.html', { cache: 'no-cache' });
    if (!res.ok) throw new Error('nav fetch failed: ' + res.status);
    let html = await res.text();
    // Rewrite {{ROOT}} placeholder so links resolve from the current page location.
    html = html.replace(/\{\{ROOT\}\}/g, root);
    slot.innerHTML = html;
    wireNavInteractions();
    highlightCurrentPage();
  } catch (e) {
    console.warn('Shared nav failed to load:', e);
    // Minimal fallback so the page is still navigable.
    slot.innerHTML = `<nav class="sticky-nav no-print"><div class="nav-inner">
      <a class="nav-brand" href="${root}index.html"><div class="nav-logo">C</div>
      <div class="nav-brand-text"><div class="title">CyberOS</div><div class="subtitle">docs</div></div></a></div></nav>`;
  }
}

function wireNavInteractions() {
  // Pagefind site-wide search (replaces the legacy in-page filter).
  setupPagefindSearch();
  // Print button
  const printBtn = document.getElementById('nav-print');
  if (printBtn) printBtn.addEventListener('click', () => window.print());
  // Dark mode toggle
  const darkBtn = document.getElementById('nav-dark');
  if (darkBtn) darkBtn.addEventListener('click', toggleDarkMode);
  // Mobile menu
  const menuBtn = document.getElementById('nav-menu');
  const mobile = document.getElementById('nav-mobile');
  if (menuBtn && mobile) {
    menuBtn.addEventListener('click', () => mobile.classList.toggle('hidden'));
  }
}

/* ---------- Pagefind — site-wide static search ----------
   Pagefind ships a Rust-built index under `<docs>/pagefind/`. We load
   pagefind-ui.js + pagefind-ui.css once and mount the widget into the
   nav's #pagefind-search slot. Path is resolved via rootBase() so this
   works from /index.html, /modules/*.html, /architecture/*.html, etc. */
let _pagefindUILoaded = null;
async function setupPagefindSearch() {
  const slot = document.getElementById('pagefind-search');
  if (!slot) return;
  if (slot.dataset.bound === '1') return;          // already mounted
  slot.dataset.bound = '1';

  const root = rootBase();                          // '' from root, '../' from subdirs
  const base = root + 'pagefind/';

  // Lazy-load pagefind-ui.{js,css} exactly once per page.
  if (!_pagefindUILoaded) {
    _pagefindUILoaded = (async () => {
      const css = document.createElement('link');
      css.rel = 'stylesheet';
      css.href = base + 'pagefind-ui.css';
      document.head.appendChild(css);

      await new Promise((resolve, reject) => {
        const s = document.createElement('script');
        s.src = base + 'pagefind-ui.js';
        s.onload = resolve;
        s.onerror = reject;
        document.head.appendChild(s);
      });
    })();
  }

  try {
    await _pagefindUILoaded;
  } catch (e) {
    console.warn('Pagefind UI failed to load:', e);
    slot.innerHTML = '<input class="nav-search" type="search" placeholder="Search unavailable" disabled aria-label="Search unavailable">';
    return;
  }

  // PagefindUI is registered as a global by pagefind-ui.js.
  if (typeof window.PagefindUI !== 'function') {
    console.warn('Pagefind UI script loaded but PagefindUI not defined.');
    return;
  }

  new window.PagefindUI({
    element: '#pagefind-search',
    bundlePath: base,
    showSubResults: true,
    showImages: false,
    excerptLength: 30,
    resetStyles: false,           // let our CSS variables theme it
    autofocus: false,
    placeholder: 'Search docs…',
    translations: {
      placeholder: 'Search docs…',
      clear_search: 'Clear',
      load_more: 'Load more results',
      search_label: 'Search this site',
      filters_label: 'Filters',
      zero_results: 'No matches for [SEARCH_TERM]',
      many_results: '[COUNT] matches for [SEARCH_TERM]',
      one_result: '[COUNT] match for [SEARCH_TERM]',
      alt_search: 'No matches for [SEARCH_TERM] — showing results for [DIFFERENT_TERM]',
      search_suggestion: 'No matches for [SEARCH_TERM] — try a different term',
      searching: 'Searching for [SEARCH_TERM]…',
    },
  });
}

function highlightCurrentPage() {
  // Mark the current top-level nav link as active.
  const path = location.pathname;
  const file = path.substring(path.lastIndexOf('/') + 1) || 'index.html';
  document.querySelectorAll('#shared-nav a[data-nav-key]').forEach(a => {
    const key = a.getAttribute('data-nav-key');
    if (path.endsWith(key) || (key === 'index.html' && (file === '' || file === 'index.html'))) {
      a.classList.add('active');
    }
  });
}

/* ---------- Intersection observer for sticky nav section highlighting ---------- */
function setupSectionObserver() {
  const sections = document.querySelectorAll('section[id]');
  if (!sections.length) return;
  const links = Array.from(document.querySelectorAll('a[href^="#"]'));
  const linkFor = id => links.find(a => a.getAttribute('href') === '#' + id);

  const io = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        const id = entry.target.id;
        document.querySelectorAll('a[href^="#"]').forEach(a => a.classList.remove('active'));
        const link = linkFor(id);
        if (link) link.classList.add('active');
      }
    });
  }, { rootMargin: '-30% 0px -60% 0px', threshold: 0 });

  sections.forEach(s => io.observe(s));
}

/* ---------- Search — filters .bbg-card / .module-cell, highlights matches ---------- */
let _searchTimer = null;
function onSearch(raw) {
  clearTimeout(_searchTimer);
  _searchTimer = setTimeout(() => runSearch(raw), 120);
}
function runSearch(raw) {
  const q = (raw || '').trim().toLowerCase();
  // Clear previous highlights
  document.querySelectorAll('mark.search-hit').forEach(m => {
    const text = document.createTextNode(m.textContent);
    m.parentNode.replaceChild(text, m);
  });
  const targets = document.querySelectorAll('.bbg-card, .module-cell, .searchable');
  if (!q) {
    targets.forEach(el => {
      el.classList.remove('search-dim', 'search-hidden');
    });
    return;
  }
  targets.forEach(el => {
    const haystack = el.textContent.toLowerCase();
    if (haystack.includes(q)) {
      el.classList.remove('search-dim', 'search-hidden');
      highlightWithin(el, q);
    } else {
      el.classList.add('search-dim');
    }
  });
}
function highlightWithin(root, query) {
  // Walk text nodes and wrap matches with <mark>.
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
    acceptNode: (node) =>
      (node.parentElement && /^(SCRIPT|STYLE|MARK|CODE|PRE)$/i.test(node.parentElement.tagName))
        ? NodeFilter.FILTER_REJECT
        : NodeFilter.FILTER_ACCEPT
  });
  const matches = [];
  let n;
  while ((n = walker.nextNode())) {
    const lc = n.nodeValue.toLowerCase();
    if (lc.includes(query)) matches.push(n);
  }
  matches.forEach(node => {
    const lc = node.nodeValue.toLowerCase();
    let i = lc.indexOf(query);
    if (i === -1) return;
    const frag = document.createDocumentFragment();
    let cursor = 0;
    while (i !== -1) {
      if (i > cursor) frag.appendChild(document.createTextNode(node.nodeValue.slice(cursor, i)));
      const mark = document.createElement('mark');
      mark.className = 'search-hit';
      mark.textContent = node.nodeValue.slice(i, i + query.length);
      frag.appendChild(mark);
      cursor = i + query.length;
      i = lc.indexOf(query, cursor);
    }
    if (cursor < node.nodeValue.length) frag.appendChild(document.createTextNode(node.nodeValue.slice(cursor)));
    node.parentNode.replaceChild(frag, node);
  });
}

/* ---------- Copy-to-clipboard buttons on <pre><code> ---------- */
function setupCodeCopyButtons() {
  document.querySelectorAll('pre').forEach(pre => {
    if (pre.parentElement.classList.contains('codeblock')) return;
    // Wrap in a .codeblock and add a copy button
    const wrap = document.createElement('div');
    wrap.className = 'codeblock';
    pre.parentNode.insertBefore(wrap, pre);
    wrap.appendChild(pre);

    const btn = document.createElement('button');
    btn.className = 'copy-btn';
    btn.type = 'button';
    btn.textContent = 'Copy';
    btn.addEventListener('click', async () => {
      try {
        await navigator.clipboard.writeText(pre.textContent);
        btn.textContent = 'Copied!';
        btn.classList.add('copied');
        setTimeout(() => {
          btn.textContent = 'Copy';
          btn.classList.remove('copied');
        }, 1400);
      } catch {
        btn.textContent = 'Failed';
        setTimeout(() => (btn.textContent = 'Copy'), 1400);
      }
    });
    wrap.appendChild(btn);
  });
}

/* ---------- Dark mode toggle ---------- */
function toggleDarkMode() {
  const cur = document.documentElement.getAttribute('data-theme');
  const next = cur === 'dark' ? 'light' : 'dark';
  document.documentElement.setAttribute('data-theme', next);
  try { localStorage.setItem('cyberos-docs-theme', next); } catch {}
}
function applyStoredTheme() {
  try {
    const saved = localStorage.getItem('cyberos-docs-theme');
    if (saved === 'dark') document.documentElement.setAttribute('data-theme', 'dark');
  } catch {}
}

/* ---------- Smooth scroll to # anchors with sticky-nav offset ---------- */
function setupSmoothAnchorScroll() {
  document.addEventListener('click', (e) => {
    const a = e.target.closest('a[href^="#"]');
    if (!a) return;
    const id = a.getAttribute('href').slice(1);
    if (!id) return;
    const tgt = document.getElementById(id);
    if (!tgt) return;
    e.preventDefault();
    const top = tgt.getBoundingClientRect().top + window.pageYOffset - 72;
    window.scrollTo({ top, behavior: 'smooth' });
    history.replaceState(null, '', '#' + id);
  });
}

/* ---------- Liquid Glass — subtle scroll-driven parallax for layered surfaces ----------
   Publishes --scroll-y on :root so CSS can reference it for depth effects.
   GPU-accelerated via requestAnimationFrame; bails out under reduced-motion. */
(function setupGlassParallax() {
  if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;

  let ticking = false;
  function updateScrollY() {
    document.documentElement.style.setProperty('--scroll-y', `${window.scrollY}px`);
    ticking = false;
  }
  window.addEventListener('scroll', () => {
    if (!ticking) {
      requestAnimationFrame(updateScrollY);
      ticking = true;
    }
  }, { passive: true });
})();

/* ---------- BOOT ---------- */
function boot() {
  applyStoredTheme();
  loadSharedNav();
  setupSmoothAnchorScroll();
  setupSectionObserver();
  setupCodeCopyButtons();
  // Render mermaid after first paint so charts get sized correctly.
  requestAnimationFrame(() => renderMermaid());
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', boot);
} else {
  boot();
}

// Expose for ad-hoc re-render after Alpine inserts content
window.rerenderMermaid = renderMermaid;
window.cyberosDocs = { renderMermaid, runSearch, toggleDarkMode, setupPagefindSearch };
