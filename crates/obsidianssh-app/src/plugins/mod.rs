//! Plugin subsystem.
//!
//! Optional helpers (`gif`, `mcp`) ship as standalone binaries the
//! app downloads on demand, verifies and caches (the manifest and
//! signing contract lives in `obsidianssh-plugin-protocol`).
//!
//! Module map:
//!
//! - [`manifest`]: parse the hosted manifest JSON, filter by
//!   protocol + `min_app`, pick the best version.
//! - [`verify`]: Ed25519 verify-only over downloaded binaries.
//! - [`download`]: reqwest GET, SHA-256 + signature gate, atomic
//!   write into the version cache.
//! - [`cache`]: on-disk layout under `~/.obsidianssh/plugins/`, keep the
//!   last two versions per provider.

pub mod cache;
pub mod download;
pub mod manifest;
pub mod seed;
pub mod verify;

/// True when a freshly-built plugin binary sits next to the app
/// executable, which the dev loop treats as installed so a
/// `cargo build` + restart picks up plugin edits without going
/// through the cache.
///
/// The single authority for that question. `detect_status` (does the
/// UI call it installed) and `resolve_binary` (what gets spawned) both
/// ask here rather than probing separately: two answers to one
/// question means a provider the panel calls a DevBuild while the host
/// spawns something else.
///
/// A harness run never sees a dev binary. The sandbox exists so a run
/// starts from first-run state whatever the machine looks like, and
/// `target/debug` is machine state that the `$HOME` redirect cannot
/// reach: with a plugin built, a harness test of the plugins panel
/// would fail on a developer's box and pass in CI.
pub fn dev_binary_present(provider_id: &str) -> bool {
    #[cfg(debug_assertions)]
    {
        #[cfg(feature = "harness")]
        if crate::harness::is_sandboxed() {
            return false;
        }
        if let Ok(exe) = std::env::current_exe()
            && let Some(dir) = exe.parent()
        {
            return dir.join(cache::binary_name(provider_id)).exists();
        }
    }
    let _ = provider_id;
    false
}

pub use manifest::{ManifestEntry, PlatformBinary, PluginManifest};

use std::path::PathBuf;

/// GitHub repo where plugin release artifacts and their `*.json`
/// manifests are published. `fetch_manifest` lists releases here,
/// filters by `<provider>-v*` tags, and downloads the highest
/// version's manifest from its release assets, no separate manifest
/// host involved.
pub const RELEASE_REPO: &str = "ObsidianNetwork/ObsidianSSH";

/// Unified error for every step of the plugin lifecycle, spawn,
/// JSON-RPC, manifest parsing, download, integrity. Kept as one enum
/// (rather than one per submodule) so call sites, and the eventual
/// `PluginProvider`, match on a single type.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// The plugin binary isn't present at the expected cache path.
    #[error("plugin binary not found: {0}")]
    BinaryNotFound(PathBuf),

    /// The manifest JSON was missing, unreachable, or malformed, or
    /// carried no version compatible with this app build.
    #[error("manifest error: {0}")]
    Manifest(String),

    /// The binary download failed (HTTP error, connection dropped).
    #[error("download failed: {0}")]
    Download(String),

    /// SHA-256 mismatch or Ed25519 signature rejection on a
    /// downloaded binary.
    #[error("integrity check failed: {0}")]
    Integrity(String),

    /// Filesystem error working with the cache directory.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Offline mode is on (`crate::offline`): nothing was dialled.
    /// Distinct from `Download` so the panel and the install modal can
    /// point at the switch instead of at a firewall.
    #[error("offline mode is on; plugin downloads are paused")]
    Offline,
}

impl PluginError {
    /// Stable i18n key for showing this error in the UI. The raw
    /// `Display` text is detailed (file paths, byte counts, HTTP
    /// codes, signature bytes) and goes through `tracing` for the
    /// log file; what the user sees is a short translated phrase
    /// per variant. Keep these aligned with the keys defined in
    /// `crate::i18n` for every language.
    pub fn i18n_key(&self) -> &'static str {
        match self {
            Self::BinaryNotFound(_) => "plugin_err_binary_not_found",
            Self::Manifest(_) => "plugin_err_manifest",
            Self::Download(_) => "plugin_err_download",
            Self::Integrity(_) => "plugin_err_integrity",
            Self::Io(_) => "plugin_err_io",
            Self::Offline => "plugin_err_offline",
        }
    }
}
