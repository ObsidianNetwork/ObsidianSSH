# Architecture

Obsidian SSH is a Cargo workspace of 30 crates. The UI layer is an
[iced](https://iced.rs) application on the wgpu backend; everything below it
is a set of focused engines (SSH, Telnet, serial, vault, sync, terminal)
that the app composes.

```
+--------------------------------------------------------------------+
| Iced Application (wgpu, GPU-accelerated)      obsidianssh-app           |
| Sidebar / Tab bar / Card grid / Terminal / SFTP / AI               |
| Slide-in editors . Split panes . Modals & overlays                 |
+------------------------------+-------------------------------------+
| Connection engines           | Encrypted vault                     |
| obsidianssh-ssh     russh, auto-  | obsidianssh-vault                        |
|   auth, jump hosts, proxies, | SQLite, Argon2id,                   |
|   -L/-R/-D, SFTP, TOFU       | ChaCha20-Poly1305 per-field,        |
| obsidianssh-telnet  RFC 854/1143, | session logs / recordings,          |
|   TLS, raw TCP               | .obsidianssh export / import             |
| obsidianssh-serial  COM / tty     |                                     |
| obsidianssh-mosh    mosh handover |                                     |
|   + UDP session over SSH     |                                     |
| obsidianssh-zmodem  sz/rz engine  |                                     |
| obsidianssh-archive tar/zip over  |                                     |
|   SFTP + local codecs        |                                     |
| obsidianssh-fido2   FIDO2 tokens: |                                     |
|   CTAP2 / HID, Windows Hello |                                     |
+--------------------------------------------------------------------+
| Cloud providers + plugin subsystem                                 |
| obsidianssh-cloud              provider trait (discovery + transport)   |
| obsidianssh-cloud-aws/-gcp/-azure/-k8s     providers                    |
| obsidianssh-cloud-aliyun/-tencent          providers                    |
| *-plugin                  subprocess binaries (JSON-RPC 2.0)       |
| obsidianssh-plugin-protocol    stdio wire contract                      |
| obsidianssh-plugin-signer      Ed25519 sign + SHA-256                   |
+------------------------------+-------------------------------------+
| P2P sync                     | AI / automation                     |
| obsidianssh-sync                  | obsidianssh-mcp                          |
| quinn QUIC, mDNS, STUN,      | JSON-RPC 2.0 over stdio,            |
| signaling + relay,           | list / get / exec SSH hosts         |
| Ed25519/X25519, LWW          | for AI assistants                   |
| obsidianssh-relay (self-host)     |                                     |
+------------------------------+-------------------------------------+
| Terminal                     | Core model types                    |
| obsidianssh-terminal              | obsidianssh-core                         |
| alacritty_terminal,          | Connection, Key, Identity,          |
| custom widget + PTY,         | ProxyIdentity, Group, Snippet,      |
| themes + custom themes       | KnownHost, PortForwardRule,         |
|                              | SessionGroup, CloudAccount, ...     |
+--------------------------------------------------------------------+
```

## Crates

| Crate | Purpose |
|-------|---------|
| `obsidianssh-app` | Iced app: views, themes, i18n, AI chat, SFTP browser, cloud UI, split panes, overlays, keyboard navigation |
| `obsidianssh-core` | Shared model types: Connection, SshKey, Identity, ProxyIdentity, Group, Snippet, KnownHost, PortForwardRule, SessionGroup, CloudAccount, custom themes, LogEntry |
| `obsidianssh-terminal` | Terminal widget: alacritty_terminal + custom canvas widget + PTY + themes + URL/IP/path detection |
| `obsidianssh-ssh` | SSH engine: auto-auth, jump hosts, SOCKS/HTTP/Command proxy, Local/Remote/Dynamic forwarding, SFTP, TOFU, RSA-SHA2 |
| `obsidianssh-fido2` | FIDO2 token transport with nothing SSH in it: CTAPHID framing, CTAP2 `authenticatorGetAssertion` with PIN/UV auth protocols 1 and 2, and the per-platform ways to reach a token (Windows Hello, raw HID on Windows, Linux and macOS), cancellable from another thread. `obsidianssh-ssh::sk` frames what it returns as an OpenSSH `sk-*` signature |
| `obsidianssh-mosh` | mosh carried over an SSH host: the `mosh-server` handover (remote command synthesis with shell quoting, announcement parsing) and the UDP session that follows, driving the `mosh-rs` crate and publishing escape bytes so the pane reads it like any other transport |
| `obsidianssh-telnet` | Native Telnet engine: RFC 854/855 option negotiation (RFC 1143 state machine), NAWS, terminal-type, charset transcoding, TLS (`telnets`), plus the raw-TCP mode console servers expose serial lines on |
| `obsidianssh-serial` | Serial console sessions: COM / `/dev/tty*`, configurable baud, framing, flow control, line endings, local echo |
| `obsidianssh-zmodem` | ZMODEM transfer engine: auto-detects `sz` / `rz` on the byte stream over SSH, Telnet and serial |
| `obsidianssh-archive` | SFTP archive operations: remote `tar` / `unzip` / `zip` command synthesis with safe quoting (POSIX + Windows), local zip / tar.gz codecs, zip central-directory browsing over ranged reads |
| `obsidianssh-vault` | Encrypted vault: SQLite + Argon2id + ChaCha20-Poly1305 per-field + session logs / recordings + `.obsidianssh` export/import; the device store (`device.json`: device-level settings and the registry of vaults) |
| `obsidianssh-biometric` | Biometric / OS-keyring app unlock: Windows Hello, macOS Touch ID (Keychain user presence), Linux Secret Service |
| `obsidianssh-sync` | P2P sync engine: QUIC (quinn) + mDNS + STUN + signaling + HTTP relay fallback + Ed25519/X25519 + LWW conflict resolution |
| `obsidianssh-relay` | Self-hostable signaling + relay HTTP server (axum + in-memory queues) |
| `obsidianssh-mcp` | MCP server binary: JSON-RPC 2.0 over stdio, exposes SSH hosts to AI assistants. Distributed as a plugin, not bundled in the OS installers |
| `obsidianssh-cloud` | Cloud provider abstraction: a `CloudProvider` trait split into discovery (list resources) and transport (open a channel: SSH / SSM / ECS Exec / kubectl exec) |
| `obsidianssh-cloud-aws` | AWS provider: named profiles, static keys, IAM Identity Center (SSO), EC2 + ECS discovery |
| `obsidianssh-cloud-gcp` | Google Cloud provider: Compute Engine + GKE discovery driven through the `gcloud` CLI |
| `obsidianssh-cloud-azure` | Azure provider: VMs + AKS discovery driven through the `az` CLI |
| `obsidianssh-cloud-aliyun` | Alibaba Cloud provider: ECS + ACK discovery driven through the `aliyun` CLI; ACK kubeconfigs are returned by the API and stored per cluster |
| `obsidianssh-cloud-tencent` | Tencent Cloud provider: CVM + TKE discovery driven through `tccli`; TKE kubeconfigs are returned by the API and stored per cluster |
| `obsidianssh-cloud-k8s` | Kubernetes provider: kubeconfig auth, workload discovery and pod shells driven through the `kubectl` CLI |
| `obsidianssh-cloud-aws-plugin` | AWS provider packaged as a standalone subprocess (JSON-RPC 2.0 over stdio) |
| `obsidianssh-cloud-gcp-plugin` | Google Cloud provider packaged as a standalone subprocess |
| `obsidianssh-cloud-azure-plugin` | Azure provider packaged as a standalone subprocess |
| `obsidianssh-cloud-aliyun-plugin` | Alibaba Cloud provider packaged as a standalone subprocess |
| `obsidianssh-cloud-tencent-plugin` | Tencent Cloud provider packaged as a standalone subprocess |
| `obsidianssh-cloud-k8s-plugin` | Kubernetes provider packaged as a standalone subprocess |
| `obsidianssh-plugin-protocol` | Wire protocol for cloud-provider plugins: line-delimited JSON-RPC 2.0 over stdio |
| `obsidianssh-plugin-signer` | CLI that signs a plugin binary with the Ed25519 key and computes the SHA-256 the manifest needs |

## Tech stack

| Layer | Technology |
|-------|-----------|
| UI | Iced (wilsonglasser fork, branch `obsidianssh`, wgpu GPU-accelerated, software-renderer fallback) |
| Icons | Lucide + Codicon (iced_fonts) + brand SVG icons |
| Fonts | Noto Sans (UI, CJK / Hebrew / Thai / Devanagari coverage) + SauceCodePro / Symbols Nerd Font (terminal) |
| Terminal | alacritty_terminal |
| SSH | russh (async, pure Rust, RSA-SHA2) |
| Telnet / Serial / ZMODEM | native Rust engines (`obsidianssh-telnet`, `obsidianssh-serial`, `obsidianssh-zmodem`) |
| Cloud | AWS (EC2/ECS, SSM, EC2 Instance Connect), GCP (`gcloud`), Azure (`az`), Alibaba Cloud (`aliyun`), Tencent Cloud (`tccli`), Kubernetes (`kubectl`), as Ed25519-signed subprocess plugins |
| AI | reqwest + Anthropic / OpenAI-compatible / Gemini APIs |
| MCP | JSON-RPC 2.0 over stdio |
| P2P Sync | quinn (QUIC), mDNS, STUN, HTTP relay fallback, Ed25519/X25519, XChaCha20-Poly1305 |
| Encryption at rest | Argon2id + ChaCha20-Poly1305 |
| Storage | SQLite (rusqlite) |
| Async | Tokio |

## Design notes

- **No Electron, no webview.** The whole UI is a single native binary;
  optional capabilities (cloud providers, MCP server) are downloaded on
  demand as signed subprocess plugins so the core stays small.
- **Secrets are per-field encrypted.** Credentials never live in plaintext
  columns; structural tests enforce it.
- **The vault is the source of truth.** Sync, export and the MCP server all
  read through the same store; nothing bypasses it.
- **Plugins are sandboxed by process.** Cloud providers speak line-delimited
  JSON-RPC 2.0 over stdio and are Ed25519-signature-verified before every
  execution.
