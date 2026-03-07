# AGENTS.md — Tool Execution Surface

Tool execution layer exposing capabilities to the LLM agent loop. Security-critical: tools can execute actions with real-world side effects.

## Where to Look

| Category | Files | Security Surface |
|----------|-------|------------------|
| **Core trait** | `traits.rs` | `Tool`, `ToolResult`, `ToolSpec` |
| **File ops** | `file_read.rs`, `file_write.rs`, `file_edit.rs` | Path sandboxing, symlink escape detection |
| **Shell** | `shell.rs` | Command allowlist, rate limiting, env sanitization |
| **HTTP/Web** | `http_request.rs`, `web_fetch.rs`, `web_search_tool.rs` | Domain allowlists, private host blocking |
| **Browser** | `browser.rs`, `browser_open.rs` | Domain allowlists, coordinate guardrails |
| **Memory** | `memory_store.rs`, `memory_recall.rs`, `memory_forget.rs` | Workspace scoping |
| **Scheduling** | `cron_*.rs`, `schedule.rs` | Command validation passthrough |
| **Hardware** | `hardware_*.rs` (feature-gated) | Peripheral access via `src/peripherals/` |
| **Registry** | `mod.rs` | `default_tools()`, `all_tools_with_runtime()` |

## Conventions

**Tool implementation pattern:**
```rust
pub struct MyTool {
    security: Arc<SecurityPolicy>,  // Required for all side-effecting tools
    // ... config fields
}

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "One-line summary for LLM" }
    fn parameters_schema(&self) -> serde_json::Value { json!({...}) }
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> { ... }
}
```

**Security checks (required for side-effecting tools):**
1. Rate limit: `security.is_rate_limited()` before any work
2. Input validation: sanitize all string/path inputs before use
3. Path/URL scoping: `security.is_path_allowed()`, domain allowlists
4. Record action: `security.record_action()` after validation passes
5. Canonicalize paths before filesystem access to block symlink escapes

**Registration:**
- Add module in `mod.rs`
- Add to `default_tools_with_runtime()` or `all_tools_with_runtime()`
- Gate with config flag if optional (browser, http, composio)

**Input validation requirements:**
- Reject early with clear error message (never silent fallback)
- Use typed structs with serde for complex args
- Clamp timeouts, sizes, offsets to safe bounds
- Null-byte injection blocked at path validation layer

**Output requirements:**
- Return `ToolResult { success, output, error }`
- Never panic in execute path — always return structured error
- Truncate large outputs with explicit indication

## Anti-Patterns (Tool-Specific)

- **Do not** bypass `SecurityPolicy` checks for convenience
- **Do not** log secrets, tokens, or raw credentials (redact headers)
- **Do not** accept unvalidated URLs or paths — always canonicalize and scope
- **Do not** silently broaden permissions when errors occur
- **Do not** spawn untracked child processes without timeout guardrails
- **Do not** skip rate-limit checks even for "safe" operations
- **Do not** expose file paths outside workspace without explicit `allowed_roots` config
- **Do not** allow private/local network hosts in HTTP tools unless explicitly required
