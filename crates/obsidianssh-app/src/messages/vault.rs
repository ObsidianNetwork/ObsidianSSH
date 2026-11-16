//! Vault open / unlock, the forgot-password reset, damaged-file repair
//! and external-write reloads, wrapped by [`crate::messages::Message::Vault`].
//!
//! New vaults are created without a master password. The unlock form
//! only exists for a vault file that already carries one.

#[derive(Debug, Clone)]
pub enum VaultMessage {
    VaultPasswordChanged(super::Redacted),
    VaultTogglePasswordVisibility,
    VaultUnlock,
    /// Create the vault (no master password) and enter the app.
    VaultSkipPassword,
    VaultDestroyConfirm,
    VaultDestroy,
    /// Another process wrote the vault file (the MCP server pinning a
    /// host key, creating or editing a host) and bumped the change
    /// marker; `writer` names it. The in-memory entity lists are
    /// re-read while the vault is unlocked.
    VaultChangedOutside { writer: String },
    /// The boot error screen's repair: rebuild a damaged vault file
    /// (`obsidianssh_vault::rebuild_vault`) on a blocking worker.
    RebuildDamagedVault,
    /// The rebuild ended: what it kept, or why it could not write.
    DamagedVaultRebuilt(Result<obsidianssh_vault::RebuildReport, String>),
    /// Restart into the rebuilt vault (`relaunch_self`).
    RestartAfterVaultRebuild,
}
