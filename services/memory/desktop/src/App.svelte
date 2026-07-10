<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  type Tab = "dashboard" | "search" | "sync" | "ops";

  type SearchHit = {
    id: string;
    kind: string;
    ts_ns: number;
    preview: string;
    score: number;
  };

  type SyncState = {
    chain_head: string | null;
    last_sync_at: string | null;
    last_sync_duration_ms: number | null;
    cloud_state: string;
  };

  // Svelte 5 runes.
  let active: Tab = $state("dashboard");

  let syncState: SyncState | null = $state(null);
  let syncError: string | null = $state(null);

  let searchQuery: string = $state("");
  let searchHits: SearchHit[] = $state([]);
  let searchError: string | null = $state(null);

  let captureText: string = $state("");
  let captureTags: string = $state("quick_note");
  let captureResult: string | null = $state(null);
  let captureError: string | null = $state(null);

  async function refreshSyncState() {
    syncError = null;
    try {
      syncState = await invoke<SyncState>("get_sync_state");
    } catch (e) {
      syncError = String(e);
    }
  }

  async function runSearch() {
    searchError = null;
    if (!searchQuery.trim()) {
      searchHits = [];
      return;
    }
    try {
      searchHits = await invoke<SearchHit[]>("search_memory", {
        query: searchQuery,
        limit: 25,
      });
    } catch (e) {
      searchError = String(e);
      searchHits = [];
    }
  }

  async function writeNote() {
    captureError = null;
    captureResult = null;
    try {
      const tags = captureTags
        .split(",")
        .map((t) => t.trim())
        .filter(Boolean);
      const path = await invoke<string>("write_quick_note", {
        text: captureText,
        tags,
      });
      captureResult = `Wrote ${path}`;
      captureText = "";
    } catch (e) {
      captureError = String(e);
    }
  }

  // ── CyberOS operations (FR-APP-001) ──────────────────────────────────────
  type OpResult = { ok: boolean; output: string };
  type ProjectInfo = { path: string; name: string; installed_version: string | null };

  let opsCheckout: string = $state("");
  let opsProjects: ProjectInfo[] = $state([]);
  let opsProject: string = $state("");
  let opsBusy: string | null = $state(null); // which operation is running
  let opsOutput: string = $state("");
  let opsOk: boolean | null = $state(null);
  let opsError: string | null = $state(null);
  let opsLoaded = false;

  async function opsLoad() {
    if (opsLoaded) return;
    opsLoaded = true;
    try {
      const s = await invoke<{ checkout: string }>("ops_get_settings");
      opsCheckout = s.checkout;
      opsProjects = await invoke<ProjectInfo[]>("ops_list_projects");
    } catch (e) {
      opsError = String(e);
    }
  }

  async function opsRefreshProjects() {
    opsError = null;
    try {
      opsProjects = await invoke<ProjectInfo[]>("ops_list_projects");
    } catch (e) {
      opsError = String(e);
    }
  }

  async function opsRun(kind: "build" | "check" | "init") {
    opsError = null;
    opsOutput = "";
    opsOk = null;
    opsBusy = kind;
    try {
      await invoke("ops_set_settings", { settings: { checkout: opsCheckout } });
      const cmd = kind === "build" ? "ops_build" : kind === "check" ? "ops_check" : "ops_init";
      const args: Record<string, string> = { checkout: opsCheckout };
      if (kind !== "build") args.project = opsProject;
      const res = await invoke<OpResult>(cmd, args);
      opsOutput = res.output;
      opsOk = res.ok;
      if (kind === "init" && res.ok) await opsRefreshProjects();
    } catch (e) {
      opsError = String(e);
      opsOk = false;
    } finally {
      opsBusy = null;
    }
  }

  // Kick off an initial sync-state fetch on mount.
  refreshSyncState();
</script>

<div class="min-h-screen flex flex-col">
  <header class="border-b border-slate-800 px-6 py-4 flex items-center justify-between">
    <h1 class="text-lg font-semibold tracking-tight">
      CyberOS BRAIN <span class="text-slate-500 font-normal">— desktop</span>
    </h1>
    <span class="text-xs text-slate-500 font-mono">v0.1.0 · FR-BRAIN-104</span>
  </header>

  <nav class="border-b border-slate-800 px-6 flex gap-2">
    {#each ["dashboard", "search", "sync", "ops"] as const as tab}
      <button
        class={`py-3 px-3 text-sm border-b-2 transition-colors ${
          active === tab
            ? "border-emerald-400 text-emerald-300"
            : "border-transparent text-slate-400 hover:text-slate-200"
        }`}
        onclick={() => {
          active = tab;
          if (tab === "ops") opsLoad();
        }}
      >
        {tab === "ops" ? "CyberOS Ops" : tab[0].toUpperCase() + tab.slice(1)}
      </button>
    {/each}
  </nav>

  <main class="flex-1 px-6 py-6">
    {#if active === "dashboard"}
      <section class="space-y-4">
        <h2 class="text-base font-medium">Quick capture</h2>
        <p class="text-sm text-slate-400">
          Writes a markdown file under <code class="font-mono text-xs">~/.cyberos/memory/store/default/captures/</code>
          with <code class="font-mono text-xs">sync_class: shareable</code>.
        </p>
        <textarea
          class="w-full bg-slate-900 border border-slate-700 rounded p-3 text-sm font-mono"
          rows="4"
          placeholder="Type a note…"
          bind:value={captureText}
        ></textarea>
        <input
          class="w-full bg-slate-900 border border-slate-700 rounded p-2 text-sm"
          placeholder="tags, comma separated"
          bind:value={captureTags}
        />
        <button
          class="bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-medium px-4 py-2 rounded disabled:opacity-40"
          disabled={!captureText.trim()}
          onclick={writeNote}
        >
          Write quick note
        </button>
        {#if captureResult}
          <div class="text-sm text-emerald-400 font-mono">{captureResult}</div>
        {/if}
        {#if captureError}
          <div class="text-sm text-red-400 font-mono">{captureError}</div>
        {/if}
      </section>
    {:else if active === "search"}
      <section class="space-y-4">
        <h2 class="text-base font-medium">Local search</h2>
        <p class="text-sm text-slate-400">
          Calls <code class="font-mono text-xs">POST 127.0.0.1:7901/v1/brain/search</code>
          via the Rust BRAIN service (FR-BRAIN-108).
        </p>
        <form
          class="flex gap-2"
          onsubmit={(e) => {
            e.preventDefault();
            runSearch();
          }}
        >
          <input
            class="flex-1 bg-slate-900 border border-slate-700 rounded p-2 text-sm"
            placeholder="search query…"
            bind:value={searchQuery}
          />
          <button class="bg-slate-700 hover:bg-slate-600 text-sm px-4 py-2 rounded" type="submit">
            Search
          </button>
        </form>
        {#if searchError}
          <div class="text-sm text-red-400 font-mono">{searchError}</div>
        {/if}
        {#if searchHits.length > 0}
          <ul class="divide-y divide-slate-800 border border-slate-800 rounded">
            {#each searchHits as hit}
              <li class="p-3 hover:bg-slate-900">
                <div class="flex justify-between items-baseline">
                  <span class="font-mono text-xs text-slate-500">{hit.kind}</span>
                  <span class="font-mono text-xs text-slate-600">score {hit.score.toFixed(3)}</span>
                </div>
                <div class="text-sm mt-1">{hit.preview}</div>
              </li>
            {/each}
          </ul>
        {:else if searchQuery && !searchError}
          <div class="text-sm text-slate-500">No hits.</div>
        {/if}
      </section>
    {:else if active === "ops"}
      <section class="space-y-4">
        <h2 class="text-base font-medium">CyberOS operations</h2>
        <p class="text-sm text-slate-400">
          Build the distributable payload, then check or init/update any project. Every button
          runs the canonical <code class="font-mono text-xs">tools/cyberos-init</code> scripts (FR-APP-001).
        </p>

        <label class="block text-sm text-slate-400">
          CyberOS checkout
          <input
            class="mt-1 w-full bg-slate-900 border border-slate-700 rounded p-2 text-sm font-mono"
            bind:value={opsCheckout}
          />
        </label>

        <div class="flex gap-2">
          <button
            class="bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-medium px-4 py-2 rounded disabled:opacity-40"
            disabled={opsBusy !== null || !opsCheckout.trim()}
            onclick={() => opsRun("build")}
          >
            {opsBusy === "build" ? "Building…" : "Build payload"}
          </button>
          <button
            class="bg-slate-700 hover:bg-slate-600 text-sm px-4 py-2 rounded"
            onclick={opsRefreshProjects}
          >
            Refresh projects
          </button>
        </div>

        <label class="block text-sm text-slate-400">
          Project
          <select
            class="mt-1 w-full bg-slate-900 border border-slate-700 rounded p-2 text-sm font-mono"
            bind:value={opsProject}
          >
            <option value="">— choose a project —</option>
            {#each opsProjects as p}
              <option value={p.path}>
                {p.name} {p.installed_version ? `(CyberOS ${p.installed_version})` : "(not initialised)"} — {p.path}
              </option>
            {/each}
          </select>
        </label>
        <input
          class="w-full bg-slate-900 border border-slate-700 rounded p-2 text-sm font-mono"
          placeholder="…or type a project path"
          bind:value={opsProject}
        />

        <div class="flex gap-2">
          <button
            class="bg-slate-700 hover:bg-slate-600 text-sm px-4 py-2 rounded disabled:opacity-40"
            disabled={opsBusy !== null || !opsProject.trim()}
            onclick={() => opsRun("check")}
          >
            {opsBusy === "check" ? "Checking…" : "Check version"}
          </button>
          <button
            class="bg-emerald-600 hover:bg-emerald-500 text-white text-sm font-medium px-4 py-2 rounded disabled:opacity-40"
            disabled={opsBusy !== null || !opsProject.trim()}
            onclick={() => opsRun("init")}
          >
            {opsBusy === "init" ? "Running…" : "Init / Update"}
          </button>
        </div>

        {#if opsError}
          <div class="text-sm text-red-400 font-mono whitespace-pre-wrap">{opsError}</div>
        {/if}
        {#if opsOutput}
          <div class={`text-xs font-mono rounded border p-3 whitespace-pre-wrap max-h-80 overflow-auto ${
            opsOk ? "border-slate-700 bg-slate-900 text-slate-300" : "border-red-800 bg-red-950 text-red-300"
          }`}>{opsOutput}</div>
          {#if opsOk === false}
            <div class="text-sm text-red-400">Operation failed — see output above.</div>
          {/if}
        {/if}
      </section>
    {:else if active === "sync"}
      <section class="space-y-4">
        <h2 class="text-base font-medium">Sync state</h2>
        <p class="text-sm text-slate-400">
          Reads <code class="font-mono text-xs">~/.cyberos/memory/store/default/sync/last-status.json</code>
          (per <code class="font-mono text-xs">brain_sync.py::LAST_STATUS_REL</code>).
        </p>
        <button
          class="bg-slate-700 hover:bg-slate-600 text-sm px-4 py-2 rounded"
          onclick={refreshSyncState}
        >
          Refresh
        </button>
        {#if syncError}
          <div class="text-sm text-red-400 font-mono">{syncError}</div>
        {/if}
        {#if syncState}
          <dl class="grid grid-cols-2 gap-x-4 gap-y-2 text-sm font-mono">
            <dt class="text-slate-500">chain_head</dt>
            <dd class="break-all">{syncState.chain_head ?? "—"}</dd>
            <dt class="text-slate-500">last_sync_at</dt>
            <dd>{syncState.last_sync_at ?? "—"}</dd>
            <dt class="text-slate-500">last_sync_duration_ms</dt>
            <dd>{syncState.last_sync_duration_ms ?? "—"}</dd>
            <dt class="text-slate-500">cloud_state</dt>
            <dd>{syncState.cloud_state}</dd>
          </dl>
        {/if}
      </section>
    {/if}
  </main>
</div>
