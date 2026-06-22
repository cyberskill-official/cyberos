# CyberOS desktop (FR-APP-002)

A Tauri 2 desktop app that triggers CyberOS workflows and skills from your Mac by driving the existing
gateway HTTP surface. The Rust backend makes every network and keychain call, so the webview never hits
CORS and the session token never enters the webview. This first slice covers the chat-trigger path (the
proven gateway `/v1/chat`), keychain-backed token storage, and a health check. The workflow and skill
picker driven by the mcp-gateway `tools/list` surface is the next iteration.

This is authored, not yet compiled - Tauri is a macOS build and there is no Rust or Tauri toolchain in the
authoring sandbox. Build and run it on your Mac with the steps below.

## Prerequisites

- Rust (stable). The repo otherwise pins 1.88; Tauri 2 needs 1.77+.
- Xcode Command Line Tools: `xcode-select --install`.
- The Tauri CLI v2: `cargo install tauri-cli --version "^2"` (gives `cargo tauri`).

## One-time: generate the app icons

`tauri.conf.json` references icon files that must exist before a build. Generate them from the official
CyberSkill logo (do not hand-draw a mark - brand doctrine is to use the exact logo):

```
cd apps/desktop/src-tauri
cargo tauri icon ~/Projects/CyberSkill/design-system/packages/brand-assets/assets/logo-mark.png
```

That writes `src-tauri/icons/` (32x32.png, 128x128.png, 128x128@2x.png, icon.icns, icon.ico).

## Run the gateway it talks to

The app calls the CyberOS gateway, which routes to your local model. Start the gateway as in
`docs/deploy/local-dev-and-testing.md` Step 4, with the local tenant policy and your LM Studio endpoint.
No `AI_GATEWAY_DEV_CORS` is needed here - the desktop backend, not a browser, makes the call:

```
cd services
AI_GATEWAY_BIND=127.0.0.1:8080 \
  AI_GATEWAY_CONFIG_DIR=ai-gateway/config/tenants \
  LMSTUDIO_ENDPOINT=http://127.0.0.1:1234 \
  cargo run -p cyberos-ai-gateway --bin cyberos-gateway
```

(`config/tenants/org-cyberskill.yaml` maps `chat.smart` to your local model. Edit the model id there if
you load a different one.)

## The Tools view (workflows and skills)

The app has two tabs: Chat (above) and Tools. The Tools tab lists the workflows and skills the mcp-gateway
exposes (`tools/list`), shows each tool's read-only / destructive annotations and input schema, and lets
you trigger one with a JSON arguments payload (`tools/call`). It needs the mcp-gateway running:

```
cd services
cargo run -p cyberos-mcp-gateway --bin cyberos-mcp -- --listen 127.0.0.1:8090
```

The default mcp-gateway URL in the app is `http://127.0.0.1:8090`. The tools list is empty until a module
registers its catalogue. Registration and federated `tools/call` dispatch are wired (FR-MCP-002 slice):
a module POSTs its tools to `/v1/mcp/register` (enable that route with `MCP_DEV_REGISTRATION=1`), after
which the tools appear here and triggering one forwards the call over JSON-RPC to that module's endpoint.
`module_unreachable` now means the registered module endpoint is actually down or returned a bad response,
not that dispatch is unimplemented. The heartbeat/health lifecycle (auto-marking a silent module unhealthy)
is the next FR-MCP-002 slice.

To see the tab list AND run a real tool end to end, start the gateway with `MCP_DEV_REGISTRATION=1` and run
the reference module - it serves an MCP endpoint and self-registers, so echo actually returns a result:

```
# terminal 1: the gateway
cd services
MCP_DEV_REGISTRATION=1 cargo run -p cyberos-mcp-gateway --bin cyberos-mcp -- --listen 127.0.0.1:8090

# terminal 2: the reference module (serves /mcp on :8099 and self-registers)
cd services/mcp-gateway/examples
python3 reference_module.py --gateway http://127.0.0.1:8090 --listen 127.0.0.1:8099
```

Refresh the Tools tab: `cyberos.demo.echo` and `cyberos.demo.now` appear. Select echo, set the arguments to
`{"message":"hello"}`, and Run - it forwards through the gateway to the module and returns your arguments.
Full walkthrough and contract tests are in `services/mcp-gateway/examples/README.md`.

## Dev and build

```
cd apps/desktop/src-tauri
cargo tauri dev      # hot-reloading dev window
cargo tauri build    # produces a signed-able .app / .dmg under target/release/bundle/
```

The default gateway URL, tenant, and alias are editable in the app's top bar; the default is
`http://127.0.0.1:8080` / `org:cyberskill` / `chat.smart`.

## If the Tauri 2 scaffold needs fixups

Tauri's config schema and CLI evolve across 2.x point releases. If `cargo tauri dev` complains about
`tauri.conf.json` or capabilities, generate a known-good skeleton and drop these files into it - the
CyberOS-specific logic lives entirely in `src-tauri/src/{lib.rs,gateway_client.rs,keychain.rs}` and
`src/{index.html,main.js}`, which are framework-agnostic:

```
# in a scratch dir
cargo create-tauri-app           # choose: vanilla, TypeScript no, package manager none
# then copy our src-tauri/src/*.rs, src/*, and the Cargo.toml dependencies + tauri.conf.json
# (frontendDist "../src", withGlobalTauri true) into the generated project.
```

## Layout

```
apps/desktop/
  src/                 frontend (static; no bundler)
    index.html
    main.js            invokes the Tauri commands
  src-tauri/
    Cargo.toml
    build.rs
    tauri.conf.json
    capabilities/default.json
    src/
      main.rs          binary entry; calls lib::run()
      lib.rs           Tauri builder + commands (health, chat, save_token, clear_token, has_token)
      gateway_client.rs  reqwest client for /healthz and /v1/chat
      keychain.rs      OS keychain token storage (keyring crate)
    icons/             generated by `cargo tauri icon` (not committed)
```
