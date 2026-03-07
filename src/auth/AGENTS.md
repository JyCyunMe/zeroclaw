# AGENTS.md — src/auth

## OVERVIEW

Multi-provider authentication with encrypted profile storage, OAuth flows, and token lifecycle management.

## WHERE TO LOOK

- `mod.rs` — `AuthService` entry point, profile selection, token refresh with backoff
- `profiles.rs` — `AuthProfilesStore`, `AuthProfile`, `TokenSet`, encrypted persistence
- `oauth_common.rs` — Shared PKCE generation, URL encoding, query parsing
- `openai_oauth.rs` — OpenAI Codex OAuth (device-code, loopback callback, JWT parsing)
- `gemini_oauth.rs` — Google/Gemini OAuth (requires `GEMINI_OAUTH_CLIENT_ID`/`SECRET`)
- `anthropic_token.rs` — Anthropic auth-kind detection (API key vs Authorization header)

## CONVENTIONS

**Profile ID format**: `<provider>:<profile_name>` (e.g., `openai-codex:default`, `anthropic:work`).

**Storage files**:
- `~/.zeroclaw/auth-profiles.json` — encrypted profile data
- `~/.zeroclaw/.secret_key` — encryption key (managed by `SecretStore`)
- `enc2:` prefix in JSON indicates encrypted fields

**Encryption**: All tokens encrypted at rest via `SecretStore` in `src/security/`. Migration from plaintext happens automatically on load.

**OAuth flows**:
- Device-code flow recommended for headless/server environments
- Loopback callback (`localhost:1455`/`1456`) for browser-based auth
- PKCE required for all authorization code exchanges
- State parameter mandatory for CSRF protection

**Token refresh**:
- 90-second skew before expiry triggers refresh
- Backoff (10s) after failed refresh attempts
- Per-profile mutex prevents concurrent refresh races

**Profile selection priority**: explicit override > active profile > default profile > first available.

**Auth kinds (Anthropic)**:
- `api-key` — standard `x-api-key` header
- `authorization` — `Authorization: Bearer` for subscription/setup tokens
- Auto-detection based on token shape (JWT-like vs `sk-ant-api` prefix)

**Testing**:
- Use `tempfile::TempDir` for isolated storage tests
- Mock token responses, never hit real OAuth endpoints
- Verify encrypted output contains `enc2:` prefix, not raw secrets

## ANTI-PATTERNS

- Never log raw access tokens, refresh tokens, or secrets
- Never bypass encryption when persisting profiles
- Never skip OAuth state validation
- Never hardcode client IDs/secrets (OpenAI is exception; Gemini requires env vars)
- Never store tokens in plaintext JSON
- Never reuse PKCE code verifiers across sessions
- Never skip lock acquisition when modifying profiles
- Never assume profile exists without checking `data.profiles.get()`
