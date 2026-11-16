# Deferred work

Things deliberately parked after the fork from Oryxis. Each entry says what
exists today, why it is parked, and what reviving it takes.

## Plugins (cloud providers, MCP server, GIF export, relay)

**State:** the plugin system is intact in code (`crates/obsidianssh-app/src/plugins/`,
the `obsidianssh-cloud-*-plugin`, `obsidianssh-mcp`, `obsidianssh-gif` and
`obsidianssh-relay` crates, the catalog in `plugins/*.json`), but installing a
plugin from the app does not work: the catalog URLs point at
`ObsidianNetwork/ObsidianSSH` releases that have not been published, and the
`sha256` / `signature` fields in the catalog still describe upstream Oryxis
builds.

**Why parked:** no current need for plugins.

**To revive:**

1. Generate a production signing keypair (`scripts/keygen-dev.sh`), put the
   public half in `PROD_PUBKEY_FALLBACK` in
   `crates/obsidianssh-app/src/plugins/verify.rs` (it still holds the upstream
   key) and the private half in the `OBSIDIANSSH_SIGNING_KEY` repository secret.
2. Run the `release-*.yml` workflows to publish signed plugin binaries to this
   repo's releases.
3. Regenerate `plugins/*.json` from those releases (new URLs, hashes,
   signatures).

Debug builds trust a dev key derived from a constant seed, so local plugin
development works without step 1 (`cargo run -p obsidianssh-plugin-signer --
sign <path> --dev`).

## Release and packaging infrastructure

The workflows under `.github/workflows/` were renamed but never run for this
fork. Each target needs an account, listing or secret of our own before the
first release:

- AUR package `obsidianssh-bin` (`AUR_SSH_PRIVATE_KEY` secret, AUR account)
- winget identifier `ObsidianNetwork.ObsidianSSH` (`WINGET_TOKEN` secret)
- Flatpak / Flathub app ID `io.github.obsidiannetwork.ObsidianSSH`
- Scoop manifest `packaging/scoop/obsidianssh.json`
- MSIX / Microsoft Store listing (`resources/msix`)
- Relay container image `ghcr.io/obsidiannetwork/obsidianssh-relay`
- Theme gallery publishing to `ObsidianNetwork/obsidianssh-site` (repo does
  not exist yet)
- App-update signing: the auto-updater verifies release signatures against a
  baked-in key, which needs the same treatment as the plugin key above.

## Upstream dependency forks

`iced`, `wgpu`, `russh`, `alacritty_terminal`, `winit`, `serialport` and
`iced_fonts` are built from Wilson Glasser's GitHub forks (several on a branch
named `oryxis`). Decision (2026-11-16): keep depending on his repos as they
are; `Cargo.lock` pins exact commits. The risk is that a deleted fork or a
force-pushed branch breaks fresh builds; if that happens, or before the first
release, fork the seven repos into ObsidianNetwork (all branches) and point
`Cargo.toml` at the copies on the same commits.
