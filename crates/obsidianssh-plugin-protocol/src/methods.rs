//! Typed request/response schema for every plugin operation.
//!
//! The shared contract provides the initialization handshake.
//!
//! Each operation is a zero-sized type implementing [`Method`], which
//! ties the method-name string to its `Params` and `Result` types so
//! the host can issue calls without stringly-typed plumbing.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};


/// JSON-RPC method-name strings shared by all plugins.
pub mod method {
    pub const INITIALIZE: &str = "initialize";
    /// Host -> plugin notification (no id, no response) asking the plugin
    /// to flush logs / close SDK clients before its stdin is closed. Purely
    /// additive: plugins that don't recognize it still exit cleanly on the
    /// stdin EOF that follows, so no protocol-version bump is needed.
    pub const SHUTDOWN: &str = "shutdown";
}

/// Links a JSON-RPC method name to its typed params and result.
///
/// Implemented by a zero-sized marker type per operation so the host
/// can write `host.call::<Initialize>(params)` and get an
/// `InitializeResult` back without naming the method string or the
/// result type twice.
pub trait Method {
    const NAME: &'static str;
    type Params: Serialize + DeserializeOwned;
    type Result: Serialize + DeserializeOwned;
}

// ---------------------------------------------------------------------------
// initialize, capability negotiation
// ---------------------------------------------------------------------------

/// Sent by the host as the first frame on every plugin connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Protocol versions the host understands. Order is irrelevant,
    /// the plugin picks the max of the intersection.
    pub supported_versions: Vec<u32>,
}

/// The plugin's reply to `initialize`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Protocol version the plugin selected: the highest value
    /// present in both the host's `supported_versions` and the
    /// plugin's own set. The host aborts the connection if this
    /// isn't a version it offered.
    pub protocol_version: u32,
    /// Stable plugin id (`"gif"`, `"mcp"`, `"relay"`, ...).
    pub provider_id: String,
    /// Human-readable plugin build version (`"0.4.2"`), shown in the
    /// plugins UI. Never used for protocol decisions.
    pub plugin_version: String,
    /// Method names this plugin implements, advertised to the host.
    pub capabilities: Vec<String>,
}

/// `initialize`, protocol + capability handshake.
pub struct Initialize;
impl Method for Initialize {
    const NAME: &'static str = method::INITIALIZE;
    type Params = InitializeParams;
    type Result = InitializeResult;
}
