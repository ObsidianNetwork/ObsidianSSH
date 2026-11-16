//! Vault access UI state: the lock screen, master-password setup, and the
//! destroy-confirm flag. Grouped off the `Obsidian SSH` god-struct as part of the
//! modules-by-feature direction (field grouping only).
//!
//! Note: the live `VaultStore` handle stays at `ObsidianSsh::vault`; only the
//! transient unlock/setup UI lives here. The struct is named `VaultUi` to
//! avoid colliding with that `vault` field.

use super::VaultState;

/// The boot error screen's repair of a damaged vault file (issue #250).
#[derive(Debug, Clone, Default)]
pub(crate) enum VaultRebuild {
    #[default]
    Idle,
    /// The copy runs on a blocking worker; the button is disabled.
    Running,
    /// The rebuilt file is in place. `restart_failed` is set when
    /// `relaunch_self` could not spawn the replacement process.
    Done { report: obsidianssh_vault::RebuildReport, restart_failed: bool },
    /// The new file could not be written; the damaged one is as it was.
    Failed(String),
}

/// Lock screen + master-password setup + destroy-confirm UI state.
#[derive(Debug, Clone, Default)]
pub(crate) struct VaultUi {
    /// Loading / NeedSetup / Locked / Unlocked.
    pub(crate) state: VaultState,
    /// Why the boot left `state` at `Loading`: the text of the error
    /// `VaultStore::open_boot` returned, shown on the error screen
    /// beside the vault path and the debug-log path. `None` once the
    /// vault opened (the state moved on) and on a boot that never
    /// tried.
    pub(crate) open_error: Option<String>,
    /// The open failed because SQLite could only open the file
    /// read-only (`VaultError::ReadOnly`): the error screen swaps its
    /// generic hint for the one that says what to check. The owner's
    /// call (2026-10-12): a vault this user cannot write to is REFUSED,
    /// with the reason, never opened read-only (nothing changed would
    /// be saved, and the pre-0.20 opens did exactly that in silence).
    pub(crate) open_read_only: bool,
    /// The open failed on a damaged file (`VaultError::Corrupt`): the
    /// error screen offers the rebuild (`obsidianssh_vault::rebuild_vault`).
    pub(crate) open_corrupt: bool,
    /// Where that rebuild stands.
    pub(crate) rebuild: VaultRebuild,
    /// Password typed on the lock / setup screen.
    pub(crate) password_input: String,
    /// Whether the lock-screen password is shown as plain text.
    pub(crate) password_visible: bool,
    /// Error shown on the lock / setup screen.
    pub(crate) error: Option<String>,
    /// Whether a master password is set (vs the empty-password vault).
    pub(crate) has_user_password: bool,
    /// When no master password is set yet, whether the inline set-password
    /// form is revealed. Flipped by the header switch so the toggle has a
    /// visible effect before a password exists; ignored once one is set.
    pub(crate) show_password_form: bool,
    /// Whether the "change master password" form is open (only meaningful
    /// once a password is set). Reuses `new_password` / `confirm_password`
    /// for the new value and adds `current_password` for verification.
    pub(crate) change_password_open: bool,
    /// Two-step confirm latch for the destroy-vault action.
    pub(crate) destroy_confirm: bool,
}
