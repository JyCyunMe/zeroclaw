# Security Subsystem

Enforces policy, sandboxing, pairing, and encrypted secrets for the ZeroClaw runtime.

## Where to Look

| Concern | File | Key Types |
|---------|------|-----------|
| Policy enforcement | `policy.rs` | `SecurityPolicy`, `AutonomyLevel`, `CommandRiskLevel` |
| Sandbox trait | `traits.rs` | `Sandbox`, `NoopSandbox` |
| Sandbox factory | `detect.rs` | `create_sandbox()` |
| Encrypted secrets | `secrets.rs` | `SecretStore` (ChaCha20-Poly1305 AEAD) |
| Gateway pairing | `pairing.rs` | `PairingGuard` |
| Audit logging | `audit.rs` | `AuditLogger`, `AuditEvent` |
| E-stop | `estop.rs` | `EstopManager`, `EstopState` |
| Prompt injection defense | `prompt_guard.rs`, `leak_detector.rs` | `PromptGuard`, `LeakDetector` |
| Sandbox backends | `docker.rs`, `firejail.rs`, `bubblewrap.rs`, `landlock.rs` | `*Sandbox` implementations |

## Conventions

**Deny-by-default enforcement:**
- Empty allowlist = block all; `"*"` = allow all (explicit opt-in)
- `workspace_only = true` by default; absolute paths rejected
- Forbidden paths apply even when `workspace_only = false`

**Forbidden paths (non-negotiable):**
```
/etc, /root, /home, /usr, /bin, /sbin, /lib, /opt, /boot, /dev, /proc, /sys, /var, /tmp
~/.ssh, ~/.gnupg, ~/.aws, ~/.config
```

**Command allowlist validation:**
- Per-segment validation (splits on `|`, `&&`, `||`, `;`, newlines)
- Blocks subshell operators: backticks, `$(`, `<(`, `>(`
- Blocks unquoted shell redirections (`<`, `>`)
- Blocks unquoted single `&` (background chaining)
- Blocks dangerous args: `find -exec`, `git config`, `git alias`

**Secret encryption at rest:**
- `enc2:` prefix = ChaCha20-Poly1305 (current, secure)
- `enc:` prefix = legacy XOR (supported for migration only)
- Key file: `~/.zeroclaw/.secret_key` with mode `0600` (Unix) or icacls restricted (Windows)

**Pairing flow:**
- 6-digit one-time code printed on startup
- `POST /pair` with `X-Pairing-Code` → bearer token
- Tokens stored as SHA-256 hashes (never plaintext)
- Brute-force lockout: 5 failed attempts → 5 min lockout

**Rate limiting:**
- Sliding 1-hour window via `ActionTracker`
- Default: 20 actions/hour
- Read operations bypass rate limit; Act operations consume budget

**Testing requirements:**
- All new validation logic must have bypass-attempt tests
- Test quote-aware parsing (single/double quotes, escapes)
- Test path traversal, symlink escape, null-byte injection
- Verify tamper detection for encrypted secrets

## Anti-Patterns

- Never log raw secrets, tokens, or pairing codes (use `redact()`)
- Never silently broaden permissions (explicit error is safer)
- Never add new forbidden paths without checking existing configs
- Never trust user input for path/command validation without parsing
- Never skip the quote-aware shell lexer when validating commands
- Never add `*` to allowlists in production without explicit approval
- Never rely on prefix matching alone for path validation (use component-aware checks)
