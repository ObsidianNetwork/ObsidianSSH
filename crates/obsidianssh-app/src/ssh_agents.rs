//! The ssh-agent roster as the app shows it: every live agent endpoint
//! (the Bitwarden desktop agent among them, see
//! `obsidianssh_ssh::agent_roster`) with the keys it serves.
//!
//! Keys that live only in an agent never enter the vault. A host that
//! prefers one stores its OpenSSH public line (`Connection::agent_key`),
//! and the editor's key picker lists the agent's keys next to the saved
//! ones under a label built here, so the picker and the stored pin always
//! agree on the name.
//!
//! Bitwarden is only asked for its keys when the user is reaching for
//! them: a LIST against a locked Bitwarden vault makes the desktop app
//! ask to be unlocked, and nobody should have to unlock Bitwarden just
//! to open this app. Background reads ([`refresh`]) leave it unasked;
//! the editor's Agent key picker uses [`refresh_with_bitwarden`], and a
//! connect that signs in through the agent asks it as part of auth.

use iced::Task;
use obsidianssh_ssh::{AgentEndpoint, AgentKey, AgentKeys};

use crate::app::{Message, ObsidianSsh};

/// Read the roster in the background without dialing Bitwarden; the
/// answer lands as `Message::AgentRoster`. For boot and the slow poll.
pub(crate) fn refresh() -> Task<Message> {
    Task::perform(obsidianssh_ssh::agent_roster(false), Message::AgentRoster)
}

/// Read the roster including Bitwarden's keys, which prompts to unlock
/// a locked Bitwarden vault. Only for the user picking an agent key.
pub(crate) fn refresh_with_bitwarden() -> Task<Message> {
    Task::perform(obsidianssh_ssh::agent_roster(true), Message::AgentRoster)
}

/// Fold a fresh roster over the previous one: an endpoint a background
/// read left unasked keeps what an earlier read learned from it, so
/// Bitwarden's keys stay in the picker and the status bar between the
/// reads that actually ask it.
pub(crate) fn carry_over(previous: &[AgentEndpoint], mut fresh: Vec<AgentEndpoint>) -> Vec<AgentEndpoint> {
    for agent in &mut fresh {
        if agent.keys == AgentKeys::NotAsked
            && let Some(prev) = previous
                .iter()
                .find(|p| p.endpoint == agent.endpoint && matches!(p.keys, AgentKeys::Listed(_)))
        {
            agent.keys = prev.keys.clone();
        }
    }
    fresh
}

/// The base64 key blob of an OpenSSH public line: what two lines with
/// different comments (or none) share when they name the same key.
pub(crate) fn key_blob(public_line: &str) -> Option<&str> {
    public_line.split_whitespace().nth(1)
}

/// The comment field of an OpenSSH public line (everything after the
/// blob), empty when the line carries none.
fn line_comment(public_line: &str) -> String {
    public_line.split_whitespace().skip(2).collect::<Vec<_>>().join(" ")
}

/// Picker label for an agent key: its comment (Bitwarden's item name) and
/// where it comes from. The fingerprint stands in for a key without a
/// comment.
fn label(comment: &str, fingerprint: &str, bitwarden: bool) -> String {
    let name = if comment.trim().is_empty() { fingerprint } else { comment.trim() };
    let source = if bitwarden { "Bitwarden" } else { "ssh-agent" };
    format!("{name} · {source}")
}

/// Every key the live agents serve, as `(label, public line)`, first
/// agent first and each key once (the same key can sit in two agents).
pub(crate) fn roster_choices(agents: &[AgentEndpoint]) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for agent in agents {
        for AgentKey { comment, public_line, fingerprint } in agent.keys.listed().into_iter().flatten() {
            let blob = key_blob(public_line);
            if out.iter().any(|(_, line)| key_blob(line) == blob) {
                continue;
            }
            out.push((label(comment, fingerprint, agent.bitwarden), public_line.clone()));
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AgentSummary {
    None,
    /// The preferred endpoint (Bitwarden when present) and what it holds:
    /// `Some(n)` keys, or `None` when it did not answer.
    Agent { bitwarden: bool, keys: Option<usize> },
    /// Bitwarden is there but has not been asked for its keys yet (see
    /// the module docs), so nothing is known about its vault.
    BitwardenNotAsked,
}

/// Summarize the roster for the status bar: Bitwarden speaks for itself
/// when it is there, otherwise the first endpoint holding keys, otherwise
/// the first endpoint.
pub(crate) fn summary(agents: &[AgentEndpoint]) -> AgentSummary {
    let pick = agents
        .iter()
        .find(|a| a.bitwarden)
        .or_else(|| agents.iter().find(|a| a.keys.listed().is_some_and(|k| !k.is_empty())))
        .or_else(|| agents.first());
    match pick {
        Some(a) if a.keys == AgentKeys::NotAsked => AgentSummary::BitwardenNotAsked,
        Some(a) => AgentSummary::Agent { bitwarden: a.bitwarden, keys: a.keys.listed().map(<[_]>::len) },
        None => AgentSummary::None,
    }
}

impl ObsidianSsh {
    pub(crate) fn agents_have_keys(&self) -> bool {
        self.ssh_agents.iter().any(|a| a.keys.listed().is_some_and(|k| !k.is_empty()))
    }

    /// The agent keys the host editor offers under the Agent method, as
    /// `(label, public line)`: the live roster minus keys already saved
    /// in the vault (their vault label covers them), plus the host's
    /// stored pin when no live agent serves it right now (Bitwarden
    /// closed), so an existing pick never silently disappears.
    pub(crate) fn editor_agent_choices(&self) -> Vec<(String, String)> {
        let vault_blobs: Vec<&str> =
            self.vaults.cur().keys.iter().filter_map(|k| key_blob(&k.public_key)).collect();
        let mut choices: Vec<(String, String)> = roster_choices(&self.ssh_agents)
            .into_iter()
            .filter(|(_, line)| key_blob(line).is_none_or(|b| !vault_blobs.contains(&b)))
            .collect();
        if let Some(stored) = self.vaults.cur().editor_form.agent_key.as_deref()
            && !choices.iter().any(|(_, line)| key_blob(line) == key_blob(stored))
        {
            choices.push((self.agent_key_label(stored), stored.to_string()));
        }
        choices
    }

    /// The picker label for a stored agent pin: the live roster's label
    /// when an agent serves the key, else one built from the line itself.
    pub(crate) fn agent_key_label(&self, public_line: &str) -> String {
        roster_choices(&self.ssh_agents)
            .into_iter()
            .find(|(_, line)| key_blob(line) == key_blob(public_line))
            .map(|(label, _)| label)
            .unwrap_or_else(|| {
                let comment = line_comment(public_line);
                label(&comment, key_blob(public_line).unwrap_or_default(), false)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(comment: &str, blob: &str) -> AgentKey {
        AgentKey {
            comment: comment.into(),
            public_line: format!("ssh-ed25519 {blob} {comment}"),
            fingerprint: format!("SHA256:{blob}"),
        }
    }

    fn endpoint(bitwarden: bool, keys: AgentKeys) -> AgentEndpoint {
        AgentEndpoint { endpoint: "/tmp/agent.sock".into(), bitwarden, keys }
    }

    fn bitwarden_at(path: &str, keys: AgentKeys) -> AgentEndpoint {
        AgentEndpoint { endpoint: path.into(), bitwarden: true, keys }
    }

    #[test]
    fn choices_name_the_source_and_list_each_key_once() {
        let agents = vec![
            endpoint(true, AgentKeys::Listed(vec![key("GitHub SSH Key", "AAA"), key("", "BBB")])),
            endpoint(false, AgentKeys::Listed(vec![key("dup", "AAA"), key("laptop", "CCC")])),
            endpoint(false, AgentKeys::Unanswered),
            endpoint(true, AgentKeys::NotAsked),
        ];
        let labels: Vec<String> = roster_choices(&agents).into_iter().map(|(l, _)| l).collect();
        assert_eq!(
            labels,
            vec!["GitHub SSH Key · Bitwarden", "SHA256:BBB · Bitwarden", "laptop · ssh-agent"]
        );
    }

    #[test]
    fn summary_prefers_bitwarden_then_an_agent_with_keys() {
        assert_eq!(summary(&[]), AgentSummary::None);
        let empty_bitwarden = vec![
            endpoint(false, AgentKeys::Listed(vec![key("k", "AAA")])),
            endpoint(true, AgentKeys::Listed(Vec::new())),
        ];
        assert_eq!(summary(&empty_bitwarden), AgentSummary::Agent { bitwarden: true, keys: Some(0) });
        let plain = vec![
            endpoint(false, AgentKeys::Unanswered),
            endpoint(false, AgentKeys::Listed(vec![key("k", "AAA")])),
        ];
        assert_eq!(summary(&plain), AgentSummary::Agent { bitwarden: false, keys: Some(1) });
        // Unasked Bitwarden is neither empty (a warning) nor unreachable.
        let unasked = vec![endpoint(false, AgentKeys::Listed(Vec::new())), endpoint(true, AgentKeys::NotAsked)];
        assert_eq!(summary(&unasked), AgentSummary::BitwardenNotAsked);
    }

    #[test]
    fn background_read_keeps_what_bitwarden_said_last() {
        let listed = AgentKeys::Listed(vec![key("GitHub SSH Key", "AAA")]);
        let previous = vec![bitwarden_at("/home/u/.bitwarden-ssh-agent.sock", listed.clone())];
        // Same socket, left unasked: the earlier listing stays.
        let fresh = vec![bitwarden_at("/home/u/.bitwarden-ssh-agent.sock", AgentKeys::NotAsked)];
        assert_eq!(carry_over(&previous, fresh)[0].keys, listed);
        // A different socket inherits nothing, and an answer that WAS
        // read is never overwritten by an older one.
        let elsewhere = vec![bitwarden_at("/tmp/bw.sock", AgentKeys::NotAsked)];
        assert_eq!(carry_over(&previous, elsewhere)[0].keys, AgentKeys::NotAsked);
        let answered = vec![bitwarden_at("/home/u/.bitwarden-ssh-agent.sock", AgentKeys::Listed(Vec::new()))];
        assert_eq!(carry_over(&previous, answered)[0].keys, AgentKeys::Listed(Vec::new()));
        // A socket that went away takes its keys with it.
        assert!(carry_over(&previous, Vec::new()).is_empty());
    }
}
