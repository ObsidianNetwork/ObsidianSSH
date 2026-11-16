//! Download-mirror routing for GitHub-bound traffic.
//!
//! Fonts, plugin installs and the auto-updater download from GitHub.
//! Where GitHub is blocked or slow (`raw.githubusercontent.com` is
//! hard-blocked on mainland-China networks, `api.github.com` and
//! release assets are intermittently unreachable), the user can set a
//! prefix proxy (the ghproxy convention: `<base>/<full-original-url>`)
//! in the one setting this module owns (`download_mirror`).
//!
//! Only the four GitHub download hosts are ever rewritten
//! (`api.github.com`, `github.com`, `raw.githubusercontent.com`,
//! `objects.githubusercontent.com`); any other URL is returned
//! untouched, so AI-provider, cloud or user-relay traffic can never
//! leak through a configured proxy (unit-tested guarantee).
//!
//! Trust model: content integrity never depends on the mirror. Fonts
//! are SHA-256-pinned; plugin binaries and app updates are SHA-256 +
//! Ed25519 gated against baked-in anchors. A hostile mirror can
//! withhold updates or replay an older release listing (stale-version
//! pin), but cannot cause execution of unsigned code.
//!
//! Choices: `GitHubDirect` (the default) dials GitHub only;
//! `Custom(base)` tries the user's prefix proxy first and falls back
//! to the direct URL (public proxies often cover only the download
//! hosts and not `api.github.com`, so the fallback keeps metadata
//! working through whichever leg answers, and a dead proxy degrades to
//! direct GitHub instead of a hard failure).

use std::sync::RwLock;

/// The GitHub hosts a mirror may rewrite. `objects.githubusercontent`
/// is where `releases/download` URLs 302-redirect to, so it is part
/// of the blocked surface even though the app never dials it
/// directly.
const GITHUB_HOSTS: &[&str] = &[
    "api.github.com",
    "github.com",
    "raw.githubusercontent.com",
    "objects.githubusercontent.com",
];

/// The user's mirror preference, one settings key (`download_mirror`
/// = `"github"` / an https base URL, the token-or-value precedent of
/// the `language` setting).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) enum MirrorChoice {
    /// GitHub only, no rewrite.
    #[default]
    GitHubDirect,
    /// Always try the given prefix-proxy base first.
    Custom(String),
}

impl MirrorChoice {
    pub(crate) fn from_setting(raw: &str) -> Self {
        // The `url` arm is a catch-all, so every named token has to be
        // listed above it or a vault holding it boots as a Custom base.
        // `auto` and `project` named the removed upstream asset host
        // and now resolve to direct GitHub.
        match raw.trim() {
            "" | "github" | "auto" | "project" => Self::GitHubDirect,
            url => Self::Custom(url.to_string()),
        }
    }

    pub(crate) fn as_setting(&self) -> String {
        match self {
            Self::GitHubDirect => "github".into(),
            Self::Custom(url) => url.clone(),
        }
    }

    /// The settings-picker token this choice renders as. `Custom`
    /// answers `"custom"` for every base, so the picker never has to
    /// know the URL.
    pub(crate) fn token(&self) -> &'static str {
        match self {
            Self::GitHubDirect => "github",
            Self::Custom(_) => "custom",
        }
    }
}

/// Process-wide choice, seeded from the vault settings at boot and
/// updated by the Settings dispatcher. A plain std RwLock: readers
/// are download tasks that touch it once per request.
static CHOICE: RwLock<MirrorChoice> = RwLock::new(MirrorChoice::GitHubDirect);

pub(crate) fn set_choice(choice: MirrorChoice) {
    if let Ok(mut guard) = CHOICE.write() {
        *guard = choice;
    }
}

pub(crate) fn choice() -> MirrorChoice {
    CHOICE.read().map(|g| g.clone()).unwrap_or_default()
}

/// The hosts a GitHub-bound download may talk to under the CURRENT
/// mirror choice, for error surfaces that want to say exactly what a
/// blocked network needs to allow: the four GitHub hosts, led by a
/// custom proxy's own host when one is set. Order matches dial order.
pub(crate) fn consulted_hosts() -> Vec<String> {
    let mut hosts: Vec<String> = GITHUB_HOSTS.iter().map(|h| (*h).to_string()).collect();
    if let MirrorChoice::Custom(base) = choice() {
        let host = host_of(&base);
        if !host.is_empty() && !hosts.contains(&host) {
            hosts.insert(0, host);
        }
    }
    hosts
}

/// The bare host of an `https://host[/path]` base, for display.
fn host_of(base: &str) -> String {
    base.split("://")
        .nth(1)
        .unwrap_or(base)
        .split('/')
        .next()
        .unwrap_or("")
        .to_string()
}

/// Whether `url` points at one of the GitHub download hosts (exact
/// host match on a parsed URL, not a substring test, so
/// `github.com.evil.tld` never qualifies).
fn is_github_bound(url: &str) -> bool {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_owned))
        .is_some_and(|host| GITHUB_HOSTS.contains(&host.as_str()))
}

/// Prefix-proxy rewrite: `<base>/<full-original-url>`.
fn rewrite(url: &str, base: &str) -> String {
    format!("{}/{}", base.trim_end_matches('/'), url)
}

/// The URLs to try, in order, for a download. Non-GitHub URLs and
/// `GitHubDirect` always come back as a single identity entry;
/// `Custom(base)` puts the prefix proxy first and the direct URL
/// second.
pub(crate) fn candidates(url: &str) -> Vec<String> {
    match choice() {
        MirrorChoice::Custom(base) if is_github_bound(url) => vec![rewrite(url, &base), url.to_string()],
        _ => vec![url.to_string()],
    }
}

/// Validate a user-entered mirror base: https and a parseable URL
/// with a host. Returns the normalized base (trailing slash trimmed).
pub(crate) fn validate_base(raw: &str) -> Result<String, ()> {
    let raw = raw.trim().trim_end_matches('/');
    let parsed = reqwest::Url::parse(raw).map_err(|_| ())?;
    if parsed.scheme() != "https" || parsed.host_str().is_none() {
        return Err(());
    }
    Ok(raw.to_string())
}

/// A real GitHub URL to route through a mirror when testing it: the
/// pinned KR font, content-addressed so the bytes never move.
const PROBE_URL: &str = "https://raw.githubusercontent.com/google/fonts/c89741abbf4eeabce432c3ed2fd7dc28b022701e/ofl/notosanskr/NotoSansKR%5Bwght%5D.ttf";

/// What the Settings "Test" button should fetch for a choice: the
/// probe URL routed through a custom prefix proxy, or `None` for
/// direct GitHub, which has no mirror to test.
pub(crate) fn probe_target(choice: &MirrorChoice) -> Option<String> {
    match choice {
        MirrorChoice::GitHubDirect => None,
        MirrorChoice::Custom(base) => Some(rewrite(PROBE_URL, base)),
    }
}

/// Probe a mirror endpoint for the Settings "Test" button: fetch its
/// first bytes, requiring TLS + HTTP 2xx/206. Returns the round-trip
/// latency in milliseconds. Takes the fully-resolved URL from
/// [`probe_target`], since the two mirror kinds address differently.
pub(crate) async fn probe(url: String) -> Result<u64, String> {
    // The mirror only routes what offline mode silences, so the Test
    // button is hidden while the switch is on; the gate here is what
    // keeps a keyboard row or a stale message from dialling anyway.
    if crate::offline::is_on() {
        return Err(crate::i18n::t("offline_mode").to_string());
    }
    let client = reqwest::Client::builder()
        .user_agent(concat!("ObsidianSSH/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(std::time::Duration::from_secs(4))
        .timeout(std::time::Duration::from_secs(8))
        .https_only(true)
        .build()
        .map_err(|e| e.to_string())?;
    let started = std::time::Instant::now();
    let resp = client
        .get(&url)
        .header(reqwest::header::RANGE, "bytes=0-127")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !(resp.status().is_success() || resp.status() == reqwest::StatusCode::PARTIAL_CONTENT) {
        return Err(format!("HTTP {}", resp.status()));
    }
    // Pull the (range-bounded) body so the probe measures a full
    // round trip, not just response headers.
    let _ = resp.bytes().await.map_err(|e| e.to_string())?;
    Ok(started.elapsed().as_millis() as u64)
}

/// Settings > Advanced UI state for the mirror block, one field on
/// `Obsidian SSH` (`download_mirror`).
#[derive(Default)]
pub(crate) struct MirrorUi {
    /// The persisted choice (mirrors the `download_mirror` setting).
    pub(crate) choice: MirrorChoice,
    /// Live contents of the custom-URL field.
    pub(crate) url_input: String,
    /// The picker sits on "Custom" while the URL is still being
    /// typed/invalid (choice keeps the last persisted value).
    pub(crate) custom_pending: bool,
    /// The last committed URL failed https/URL validation.
    pub(crate) url_error: bool,
    /// A probe is in flight (Test button disabled).
    pub(crate) testing: bool,
    /// Outcome of the last probe: latency in ms or the failure.
    pub(crate) test_result: Option<Result<u64, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One test for everything that touches the process-wide CHOICE:
    /// cargo runs tests concurrently and two tests flipping the same
    /// global would race.
    #[test]
    fn mirror_routing() {
        let base = "https://mirror.example.com";
        let cases = [
            ("https://api.github.com/repos/x/releases", true),
            ("https://github.com/x/releases/download/v1/a.bin", true),
            ("https://raw.githubusercontent.com/google/fonts/c/f.ttf", true),
            ("https://objects.githubusercontent.com/blob/x", true),
            ("https://api.anthropic.com/v1/messages", false),
            ("https://github.com.evil.tld/payload", false),
            ("https://mygithub.com/x", false),
            ("https://relay.user.example/healthz", false),
        ];
        set_choice(MirrorChoice::Custom(base.into()));
        for (url, github) in cases {
            assert_eq!(is_github_bound(url), github, "{url}");
            if github {
                assert_eq!(
                    candidates(url),
                    vec![format!("{base}/{url}"), url.to_string()],
                    "{url}"
                );
            } else {
                assert_eq!(candidates(url), vec![url.to_string()], "{url}");
            }
        }

        let url = "https://api.github.com/repos/x/releases";
        set_choice(MirrorChoice::GitHubDirect);
        assert_eq!(candidates(url), vec![url.to_string()]);

        // The allowlist an error surface shows follows the same
        // choice, in dial order. Asserted inside this test because it
        // shares the process-wide CHOICE with the cases above.
        assert_eq!(consulted_hosts().len(), 4);
        set_choice(MirrorChoice::Custom("https://proxy.corp.example/p".into()));
        assert_eq!(consulted_hosts().first().unwrap(), "proxy.corp.example");
        assert_eq!(consulted_hosts().len(), 5);
        set_choice(MirrorChoice::GitHubDirect);
    }

    #[test]
    fn probe_targets_only_custom_mirrors() {
        assert_eq!(probe_target(&MirrorChoice::GitHubDirect), None);
        assert_eq!(
            probe_target(&MirrorChoice::Custom("https://p.example/gh/".into())),
            Some(format!("https://p.example/gh/{PROBE_URL}"))
        );
    }

    #[test]
    fn setting_round_trip() {
        for (raw, choice) in [
            ("github", MirrorChoice::GitHubDirect),
            ("", MirrorChoice::GitHubDirect),
            // Tokens of the removed asset-host modes must resolve
            // BEFORE the url catch-all, or a vault holding one boots
            // as a Custom base pointed at a string that is not a URL.
            ("auto", MirrorChoice::GitHubDirect),
            ("project", MirrorChoice::GitHubDirect),
            (
                "https://mirror.example.com",
                MirrorChoice::Custom("https://mirror.example.com".into()),
            ),
        ] {
            assert_eq!(MirrorChoice::from_setting(raw), choice);
            assert_eq!(MirrorChoice::from_setting(raw).token(), choice.token());
        }
        assert_eq!(MirrorChoice::GitHubDirect.as_setting(), "github");
        // The picker's token round-trips through the setting, so a
        // selected row can never render as a different row.
        assert_eq!(MirrorChoice::GitHubDirect.token(), MirrorChoice::GitHubDirect.as_setting());
        assert_eq!(MirrorChoice::Custom("https://m.cn".into()).token(), "custom");
        assert_eq!(
            MirrorChoice::from_setting(&MirrorChoice::Custom("https://m.cn".into()).as_setting()),
            MirrorChoice::Custom("https://m.cn".into())
        );
    }

    #[test]
    fn base_validation_requires_https_url() {
        assert!(validate_base("https://mirror.example.com/").is_ok());
        assert_eq!(
            validate_base("https://mirror.example.com/").unwrap(),
            "https://mirror.example.com"
        );
        assert!(validate_base("http://mirror.example.com").is_err());
        assert!(validate_base("not a url").is_err());
        assert!(validate_base("").is_err());
        assert!(validate_base("ftp://x").is_err());
    }
}
