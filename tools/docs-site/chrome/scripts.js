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
  // Post-process: Mermaid emits SVG with width="100%" which forces the SVG to
  // shrink-fit its container. For wide flowchart-LR diagrams (e.g. Stages 1-5
  // with viewBox 4200x190), this compresses the rendered height to ~50px and
  // makes labels microscopic. Fix: replace width="100%" with the viewBox's
  // natural width, so the SVG keeps its true aspect ratio and the .mermaid
  // container's overflow-x:auto provides horizontal scroll on narrow viewports.
  document.querySelectorAll('.mermaid svg').forEach(svg => {
    if (svg.dataset.sizingFixed === '1') return;
    const vb = svg.getAttribute('viewBox');
    if (!vb) return;
    const parts = vb.split(/\s+/);
    if (parts.length !== 4) return;
    const naturalW = parseFloat(parts[2]);
    const naturalH = parseFloat(parts[3]);
    if (!isFinite(naturalW) || !isFinite(naturalH)) return;
    // Cap super-tall diagrams at 80vh; wider-than-container diagrams keep natural width.
    svg.removeAttribute('width');
    svg.removeAttribute('height');
    svg.style.width = naturalW + 'px';
    svg.style.height = naturalH + 'px';
    svg.style.maxWidth = 'none';
    svg.style.maxHeight = 'none';
    svg.style.display = 'block';
    svg.dataset.sizingFixed = '1';
  });

  // Wire each .mermaid container as click-to-zoom: opens a modal with a scaled,
  // pan-and-pinch-zoomable copy of the SVG. Prevents the horizontal-overflow
  // scroll-trap on wide flowchart-LR diagrams (and keeps non-zoom users intact).
  document.querySelectorAll('.mermaid').forEach(box => {
    if (box.dataset.zoomBound === '1') return;
    if (!box.querySelector('svg')) return;
    box.dataset.zoomBound = '1';
    box.classList.add('mermaid-zoomable');
    if (!box.querySelector('.mermaid-zoom-hint')) {
      const hint = document.createElement('span');
      hint.className = 'mermaid-zoom-hint';
      hint.setAttribute('aria-hidden', 'true');
      hint.textContent = '⤢ click to zoom';
      box.appendChild(hint);
    }
    box.addEventListener('click', (e) => {
      // Ignore selection-text clicks
      if (window.getSelection && String(window.getSelection()).length > 0) return;
      openMermaidModal(box);
    });
  });
}

/* ---------- Mermaid zoom modal ---------- */
let __mermaidModalEl = null;
let __mermaidPanState = null;
function ensureMermaidModal() {
  if (__mermaidModalEl) return __mermaidModalEl;
  const el = document.createElement('div');
  el.className = 'mermaid-modal';
  el.setAttribute('role', 'dialog');
  el.setAttribute('aria-modal', 'true');
  el.setAttribute('aria-label', 'Diagram zoom view');
  el.hidden = true;
  el.innerHTML = `
    <div class="mermaid-modal-backdrop" data-close="1"></div>
    <div class="mermaid-modal-shell">
      <div class="mermaid-modal-toolbar">
        <button type="button" class="mermaid-modal-btn" data-action="zoom-out" title="Zoom out">−</button>
        <button type="button" class="mermaid-modal-btn" data-action="zoom-reset" title="Reset zoom">100%</button>
        <button type="button" class="mermaid-modal-btn" data-action="zoom-in" title="Zoom in">+</button>
        <span class="mermaid-modal-spacer"></span>
        <button type="button" class="mermaid-modal-btn mermaid-modal-close" data-close="1" title="Close (Esc)" aria-label="Close">✕</button>
      </div>
      <div class="mermaid-modal-stage" tabindex="0"></div>
    </div>
  `;
  document.body.appendChild(el);
  el.addEventListener('click', (e) => {
    if (e.target && e.target.closest('[data-close]')) closeMermaidModal();
  });
  const stage = el.querySelector('.mermaid-modal-stage');
  stage.addEventListener('wheel', (e) => {
    if (!__mermaidPanState) return;
    e.preventDefault();
    const dir = e.deltaY < 0 ? 1.1 : (1 / 1.1);
    setMermaidZoom(__mermaidPanState.scale * dir, e.clientX, e.clientY);
  }, { passive: false });
  stage.addEventListener('mousedown', startMermaidPan);
  window.addEventListener('mousemove', moveMermaidPan);
  window.addEventListener('mouseup', endMermaidPan);
  el.querySelector('[data-action="zoom-in"]').addEventListener('click', () => setMermaidZoom(__mermaidPanState.scale * 1.25));
  el.querySelector('[data-action="zoom-out"]').addEventListener('click', () => setMermaidZoom(__mermaidPanState.scale / 1.25));
  el.querySelector('[data-action="zoom-reset"]').addEventListener('click', () => { resetMermaidZoom(); });
  document.addEventListener('keydown', (e) => {
    if (!el.classList.contains('open')) return;
    if (e.key === 'Escape') closeMermaidModal();
    if (e.key === '+' || e.key === '=') setMermaidZoom(__mermaidPanState.scale * 1.25);
    if (e.key === '-' || e.key === '_') setMermaidZoom(__mermaidPanState.scale / 1.25);
    if (e.key === '0') { resetMermaidZoom(); }
  });
  __mermaidModalEl = el;
  return el;
}

function openMermaidModal(sourceBox) {
  const modal = ensureMermaidModal();
  const stage = modal.querySelector('.mermaid-modal-stage');
  const svg = sourceBox.querySelector('svg');
  if (!svg) return;
  const clone = svg.cloneNode(true);

  // Determine the SVG's intrinsic pixel size from its viewBox so it renders
  // at full natural size inside the modal canvas. Without explicit width/height,
  // browsers can render an SVG at 0×0 when its parent has `max-width: none`.
  let naturalW = 0;
  let naturalH = 0;
  const vb = clone.getAttribute('viewBox');
  if (vb) {
    const parts = vb.trim().split(/\s+/);
    if (parts.length === 4) {
      naturalW = parseFloat(parts[2]) || 0;
      naturalH = parseFloat(parts[3]) || 0;
    }
  }
  // Fallback to whatever was set on the source SVG (post-renderMermaid sizing).
  if (!naturalW) naturalW = parseFloat(svg.getAttribute('width')) || parseFloat(svg.style.width) || 800;
  if (!naturalH) naturalH = parseFloat(svg.getAttribute('height')) || parseFloat(svg.style.height) || 600;

  // Set explicit pixel size on the clone — overrides the css `max-width: none`
  // and gives the canvas a deterministic intrinsic size to scale + pan around.
  clone.setAttribute('width', String(naturalW));
  clone.setAttribute('height', String(naturalH));
  clone.style.width = naturalW + 'px';
  clone.style.height = naturalH + 'px';
  clone.style.maxWidth = 'none';
  clone.style.maxHeight = 'none';
  clone.style.display = 'block';

  stage.innerHTML = '';
  const inner = document.createElement('div');
  inner.className = 'mermaid-modal-canvas';
  // Compute an initial fit-to-stage scale so the diagram is fully visible on open.
  // We don't have layout yet (stage isn't visible), so we approximate using window size
  // minus toolbar height and the 2vh/2vw margins from the shell.
  const availW = Math.max(320, (window.innerWidth || 1280) * 0.92 - 32);
  const availH = Math.max(240, (window.innerHeight || 800) * 0.92 - 80);
  const fit = Math.min(1, availW / naturalW, availH / naturalH);
  inner.appendChild(clone);
  stage.appendChild(inner);

  __mermaidPanState = {
    scale: fit > 0 ? fit : 1,
    x: 0,
    y: 0,
    drag: false,
    sx: 0,
    sy: 0,
    naturalW,
    naturalH,
  };
  applyMermaidTransform();
  modal.hidden = false;
  // next tick for transition
  requestAnimationFrame(() => modal.classList.add('open'));
  document.body.style.overflow = 'hidden';
}

function closeMermaidModal() {
  if (!__mermaidModalEl) return;
  __mermaidModalEl.classList.remove('open');
  document.body.style.overflow = '';
  setTimeout(() => {
    if (!__mermaidModalEl.classList.contains('open')) {
      __mermaidModalEl.hidden = true;
      const stage = __mermaidModalEl.querySelector('.mermaid-modal-stage');
      if (stage) stage.innerHTML = '';
    }
  }, 200);
}

function setMermaidZoom(next, cx, cy) {
  if (!__mermaidPanState) return;
  const clamped = Math.max(0.1, Math.min(8, next));
  __mermaidPanState.scale = clamped;
  applyMermaidTransform();
}

/// Reset to the initial fit-to-stage scale + center pan.
function resetMermaidZoom() {
  if (!__mermaidPanState) return;
  const { naturalW, naturalH } = __mermaidPanState;
  const stage = __mermaidModalEl && __mermaidModalEl.querySelector('.mermaid-modal-stage');
  let fit = 1;
  if (stage && naturalW > 0 && naturalH > 0) {
    const r = stage.getBoundingClientRect();
    if (r.width > 0 && r.height > 0) {
      fit = Math.min(1, r.width / naturalW, r.height / naturalH);
    }
  }
  __mermaidPanState.scale = fit > 0 ? fit : 1;
  __mermaidPanState.x = 0;
  __mermaidPanState.y = 0;
  applyMermaidTransform();
}

function applyMermaidTransform() {
  if (!__mermaidModalEl || !__mermaidPanState) return;
  const canvas = __mermaidModalEl.querySelector('.mermaid-modal-canvas');
  if (!canvas) return;
  // The canvas is positioned at left:50%/top:50% in CSS, so we have to
  // re-apply the -50%/-50% centering offsets in EVERY transform — otherwise
  // they get overwritten and the canvas's top-left lands at the stage's
  // center, leaving most of the diagram off-screen / invisible.
  const { x, y, scale } = __mermaidPanState;
  canvas.style.transform =
    `translate(calc(-50% + ${x}px), calc(-50% + ${y}px)) scale(${scale})`;
}

function startMermaidPan(e) {
  if (!__mermaidPanState) return;
  if (e.button !== 0) return;
  __mermaidPanState.drag = true;
  __mermaidPanState.sx = e.clientX - __mermaidPanState.x;
  __mermaidPanState.sy = e.clientY - __mermaidPanState.y;
  e.preventDefault();
}
function moveMermaidPan(e) {
  if (!__mermaidPanState || !__mermaidPanState.drag) return;
  __mermaidPanState.x = e.clientX - __mermaidPanState.sx;
  __mermaidPanState.y = e.clientY - __mermaidPanState.sy;
  applyMermaidTransform();
}
function endMermaidPan() {
  if (__mermaidPanState) __mermaidPanState.drag = false;
}

/* ---------- Path helpers — derive the site root from THIS script's own URL ----------
   Every generated page loads scripts.js via `<up>assets/scripts.js`, where `<up>` is the
   correct number of `../` for the page's depth (0 for the site root, 2 for a module page
   like /modules/memory/). So the script's resolved src always ends with `assets/scripts.js`
   at the true depth. Stripping that suffix yields the assets dir and the site root at ANY
   depth - deployment-agnostic, and it fixes the previous heuristic that assumed everything
   was exactly one level deep (which broke the logo, every nav link, and sent the Architecture
   links to /modules/architecture/*.html 404s on two-level pages). NOTE: type="module" scripts
   have `document.currentScript === null`, so find the tag by its src. */
function assetsBase() {
  const me = Array.from(document.getElementsByTagName('script'))
    .find(s => s.src && /\/assets\/scripts\.js(\?.*)?$/.test(s.src));
  if (me) return me.src.replace(/scripts\.js(\?.*)?$/, '');
  return 'assets/'; // last resort: same directory
}

function rootBase() {
  // The assets dir sits directly under the site root, so the root is assetsBase without `assets/`.
  const base = assetsBase();
  return /assets\/$/.test(base) ? base.replace(/assets\/$/, '') : './';
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
window.openMermaidModal = openMermaidModal;
window.closeMermaidModal = closeMermaidModal;
window.cyberosDocs = { renderMermaid, runSearch, toggleDarkMode, openMermaidModal, closeMermaidModal };
