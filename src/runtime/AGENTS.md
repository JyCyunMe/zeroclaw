# AGENTS.md — Runtime Adapters

Runtime adapters abstract execution environments for the agent orchestration layer.

## WHERE TO LOOK

- `traits.rs` — `RuntimeAdapter` trait definition + capability contracts
- `mod.rs` — factory function `create_runtime()` + error handling for unsupported kinds
- `native.rs` — default runtime with full shell/filesystem/long-running support
- `docker.rs` — sandboxed runtime with container isolation
- `wasm.rs` — (feature-gated) WASM sandbox with fuel limits and memory caps

## CONVENTIONS

### RuntimeAdapter Implementation

- Must be `Send + Sync` (shared across async tasks).
- Return accurate capability flags: `has_shell_access`, `has_filesystem_access`, `supports_long_running`.
- `memory_budget()` returns 0 for unlimited; constrained runtimes must return actual ceiling.
- `build_shell_command()` must error if shell access is unavailable.

### Supported Runtimes

- `"native"` — full access, default for local development
- `"docker"` — containerized with configurable isolation (network, memory, read-only rootfs)
- `"wasm"` — feature-gated (`--features runtime-wasm`), no shell, pure computation

### Docker Sandbox Configuration

- `image`, `network`, `memory_limit_mb`, `cpu_limit`, `read_only_rootfs`, `mount_workspace`
- `allowed_workspace_roots` — optional allowlist; blocks mounts outside specified paths
- Refuses to mount `/` (filesystem root) — explicit security guard

### Error Handling

- Unsupported `runtime.kind` (e.g., `"cloudflare"`) must `bail!` with clear message — never silently fall back.
- Empty `runtime.kind` errors explicitly.
- WASM runtime returns feature-gated stub when `runtime-wasm` not enabled.

### Testing Requirements

- Factory tests must cover: native, docker, unknown kind, empty kind.
- Docker tests: network flag, read-only flag, memory/CPU limits, root mount refusal.
- WASM tests: fuel limits, memory caps, config validation, path traversal rejection.

## ANTI-PATTERNS

- Do NOT silently fall back to native on unknown runtime kind.
- Do NOT bypass `allowed_workspace_roots` validation for Docker mounts.
- Do NOT enable shell access on WASM runtime — it exists specifically to deny shell.
- Do NOT skip capability checks in orchestration code — use trait methods instead of hardcoding.
