# AGENTS.md — ZeroClaw Tunnel Module

## OVERVIEW

Secure remote access layer: wraps external tunnel binaries (cloudflared, tailscale, ngrok, or custom) to expose the gateway without public network binding.

## WHERE TO LOOK

| Concern | File | Key |
|---------|------|-----|
| `Tunnel` trait definition | `mod.rs` | `pub trait Tunnel` |
| Factory wiring | `mod.rs` | `create_tunnel()` |
| Shared process utilities | `mod.rs` | `SharedProcess`, `kill_shared()` |
| Cloudflare implementation | `cloudflare.rs` | `CloudflareTunnel` |
| Tailscale implementation | `tailscale.rs` | `TailscaleTunnel` |
| ngrok implementation | `ngrok.rs` | `NgrokTunnel` |
| Custom command wrapper | `custom.rs` | `CustomTunnel` |
| No-op fallback | `none.rs` | `NoneTunnel` |
| Gateway integration | `../gateway/mod.rs` | public bind guard |

## CONVENTIONS

### Trait contract

Implement `Tunnel` for each provider:

```rust
#[async_trait::async_trait]
impl Tunnel for MyTunnel {
    fn name(&self) -> &str { "my_tunnel" }
    async fn start(&self, local_host: &str, local_port: u16) -> Result<String>;
    async fn stop(&self) -> Result<()>;
    async fn health_check(&self) -> bool;
    fn public_url(&self) -> Option<String>;
}
```

### Factory pattern

Register new providers in `create_tunnel()` with:

1. Config validation (bail if required section missing)
2. Stable lowercase provider key (e.g., `"cloudflare"`, `"tailscale"`)
3. Return `Ok(None)` for `"none"` or empty provider

### Shared process handling

- Use `new_shared_process()` to create the handle
- Store spawned child in `TunnelProcess { child, public_url }`
- Call `kill_shared(&self.proc)` in `stop()`
- Use `try_lock()` in sync `public_url()` to avoid blocking

### Gateway security contract

Gateway refuses `0.0.0.0` bind when `tunnel.provider = "none"` and `allow_public_bind = false`. This is enforced in `src/gateway/mod.rs` before the server starts.

### Configuration schema

Add tunnel-specific config to `src/config/schema.rs`:

- Cloudflare: `token: String`
- Tailscale: `funnel: bool`, `hostname: Option<String>`
- ngrok: `auth_token: String`, `domain: Option<String>`
- Custom: `start_command: String`, `health_url: Option<String>`, `url_pattern: Option<String>`

## TESTING REQUIREMENTS

- Unit tests for factory: missing config errors, valid config succeeds
- Unit tests per implementation: `name()`, `public_url()` before start, `health_check()` before start
- Integration marker: tests that spawn real binaries should be ignored by default
- Use `tokio::test` for async trait method tests
- Cover `kill_shared()` with and without an active child process

## ANTI-PATTERNS

- Do not log tokens, auth keys, or tunnel URLs with secrets
- Do not block on async locks in sync methods; use `try_lock()`
- Do not spawn tunnels without `kill_on_drop(true)` to prevent orphan processes
- Do not add tunnel providers without corresponding config schema and factory wiring
- Do not bypass the gateway public-bind guard; it is a security boundary
