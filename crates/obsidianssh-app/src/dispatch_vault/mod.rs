//! `ObsidianSsh::handle_vault`, the vault-domain router.
//!
//! `open` gets you in (create, unlock, reset); `repair` rebuilds a
//! damaged file.

#![allow(clippy::result_large_err)]

// Dispatch sub-handlers, one file per arm family.
mod open;
mod repair;

use iced::Task;

use crate::app::{SshMessage, VaultMessage, Message, ObsidianSsh};
use crate::state::{VaultState};
use obsidianssh_vault::VaultError;

impl ObsidianSsh {
    pub(crate) fn handle_vault(
        &mut self,
        message: VaultMessage,
    ) -> Task<Message> {
        match message {
            m @ (
                VaultMessage::VaultPasswordChanged(..)
                | VaultMessage::VaultTogglePasswordVisibility
                | VaultMessage::VaultSkipPassword
                | VaultMessage::VaultDestroyConfirm
                | VaultMessage::VaultDestroy
                | VaultMessage::VaultUnlock
            ) => self.handle_vault_open(m),
            m @ (
                VaultMessage::RebuildDamagedVault
                | VaultMessage::DamagedVaultRebuilt(..)
                | VaultMessage::RestartAfterVaultRebuild
            ) => self.handle_vault_repair(m),
            VaultMessage::VaultChangedOutside { writer } => {
                tracing::info!(writer = %writer, "vault written by another process, reloading");
                // The marker names no vault (one file for the device), so
                // every open vault re-reads its rows; the reload is a
                // no-op for a locked one.
                self.on_each_open_vault(|app| {
                    app.reload_after_external_write();
                    None::<()>
                });
                Task::none()
            }
        }
    }
}
