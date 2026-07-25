# Payload supply-chain posture (TASK-IMP-043)

CyberOS 1.x treats the **versioned payload tarball** as the primary distributable.

## Integrity today

- **SHA256SUMS** — published beside release assets; bootstrap/rollout refuse unverified installs.
- **SBOM** — `tools/install/emit-payload-sbom.sh` emits CycloneDX JSON; release uploads `cyberos-payload.cdx.json`.
- **GitHub Actions pins** — suite-gate, payload-gate, release.yml pin official `actions/*` to full commit SHAs.

## Deferred

- **cosign / Sigstore signing of GHCR images** — platform-scoped; SHA256SUMS remain the consumer verify control.
- Pinning every third-party Action (tauri, rust-toolchain, Play upload) — follow-on.
