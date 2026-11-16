//! First-run welcome / onboarding carousel messages, wrapped by
//! [`crate::messages::Message::Onboarding`]. Handled by
//! `ObsidianSsh::handle_onboarding`.

/// Drives the slide index of the onboarding carousel (rendered off
/// `VaultState::NeedSetup`); the final slide creates the vault via the
/// existing `Vault*` messages.
#[derive(Debug, Clone)]
pub enum OnboardingMessage {
    /// Advance one slide (clamped to the last).
    Next,
    /// Step back one slide (clamped to zero).
    Back,
    /// Jump straight to the final (import / Get started) slide.
    SkipToEnd,
    /// "Import my hosts" on the import slide: remembers the intent and
    /// creates the vault, which then opens the importer.
    ImportAfterSetup,
}
