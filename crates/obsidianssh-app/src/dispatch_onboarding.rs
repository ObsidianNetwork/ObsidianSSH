//! `ObsidianSsh::handle_onboarding`: dispatch arms for the welcome / onboarding
//! carousel, split out of dispatch.rs. Returns `Err(message)` for anything
//! it doesn't claim so the try_handler! chain falls through.
#![allow(clippy::result_large_err)]

use iced::Task;

use crate::app::{Message, OnboardingMessage, ObsidianSsh};

/// Last slide index. The carousel has five slides (0..=4): welcome,
/// connect, the SSH agent (Bitwarden), the optional-features toggles,
/// and the import offer, whose Get started creates the vault. Kept here
/// as the single source of truth for navigation clamping and the "Skip"
/// jump.
pub(crate) const ONBOARDING_LAST_SLIDE: usize = 4;

/// Index of the optional-features slide (toggles).
pub(crate) const ONBOARDING_FEATURES_SLIDE: usize = 3;

impl ObsidianSsh {
    pub(crate) fn handle_onboarding(
        &mut self,
        message: OnboardingMessage,
    ) -> Task<Message> {
        match message {
            OnboardingMessage::Next => {
                if self.onboarding_slide < ONBOARDING_LAST_SLIDE {
                    self.onboarding_slide += 1;
                }
            }
            OnboardingMessage::Back => {
                if self.onboarding_slide > 0 {
                    self.onboarding_slide -= 1;
                }
            }
            OnboardingMessage::SkipToEnd => {
                self.onboarding_slide = ONBOARDING_LAST_SLIDE;
            }
            OnboardingMessage::ImportAfterSetup => {
                // The import needs the vault, which does not exist yet on
                // this screen: remember the intent and create it; the
                // creation consumes the intent (`take_onboarding_import_task`).
                self.onboarding_import_pending = true;
                return Task::done(Message::Vault(crate::app::VaultMessage::VaultSkipPassword));
            }
        }
        Task::none()
    }

    /// Open the Import hub right after the vault is created, when the
    /// onboarding import slide asked for it. One-shot.
    pub(crate) fn take_onboarding_import_task(&mut self) -> Task<Message> {
        if !std::mem::take(&mut self.onboarding_import_pending) {
            return Task::none();
        }
        Task::done(Message::Share(crate::app::ShareMessage::ShowImportHub))
    }
}
