# CyberOS CHAT patch series

The active patch series is currently empty. Patches placed here are applied
lexicographically at Docker build time (see `../Dockerfile`, stage 1) over the
pinned upstream Mattermost commit. An empty series is a valid state.

Authentication is no longer done by a build patch. As of FR-CHAT-013, CHAT
federates to the CyberOS OIDC provider (FR-AUTH-110) through Mattermost's own
native connector - server configuration, not a patch and not a plugin. See
`../deploy/oidc-sso-config.md`.

`superseded/` holds the two FR-CHAT-002 patches (`010-disable-builtin-auth`,
`011-load-authbridge-plugin`) that disabled builtin auth and loaded the closed
AuthBridge plugin. They are kept for history and are not applied: the build
globs only top-level `*.patch`, so files under `superseded/` are ignored.
