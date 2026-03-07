# AGENTS.md — ZeroClaw Gateway Module

## OVERVIEW

Axum-based HTTP gateway with webhook endpoints, WebSocket chat, SSE events, and secure pairing flow.

## WHERE TO LOOK

| File | Purpose |
|------|---------|
| `mod.rs` | Core gateway setup, routes, rate limiting, idempotency, channel webhooks (WhatsApp, Linq, WATI, Nextcloud Talk) |
| `api.rs` | REST API handlers for web dashboard (`/api/*`), config masking/restoration |
| `ws.rs` | WebSocket chat endpoint (`/ws/chat`) with JSON protocol |
| `sse.rs` | Server-Sent Events stream (`/api/events`) with broadcast observer |
| `static_files.rs` | Embedded web dashboard assets via `rust-embed` |

## CONVENTIONS

### Endpoint Security

- `/health` and `/metrics` are public (no auth required)
- All `/api/*` and `/ws/chat` require bearer token when pairing is enabled
- Webhook signature verification uses constant-time comparison (`constant_time_eq`)
- Secrets stored as SHA-256 hashes (never plaintext): `webhook_secret_hash`

### Binding Rules

- Default bind: `127.0.0.1` — localhost only
- Public bind (`0.0.0.0`) requires either:
  - Active tunnel (`tunnel.provider != "none"`), OR
  - Explicit opt-in: `gateway.allow_public_bind = true`
- Check via `is_public_bind(host)` in `src/security/pairing.rs`

### Rate Limiting

- Sliding window per IP/client key
- Separate limits for `/pair` and `/webhook`
- Configurable: `pair_rate_limit_per_minute`, `webhook_rate_limit_per_minute`

### Idempotency

- Optional `X-Idempotency-Key` header on `/webhook`
- TTL-based key retention with LRU eviction under memory pressure

### Config Masking (api.rs)

- Secrets masked as `"***MASKED***"` in GET responses
- PUT `/api/config` restores masked secrets from current config before saving
- Never persist UI placeholders when no restore target exists

### WebSocket Protocol

```text
Client -> Server: {"type":"message","content":"..."}
Server -> Client: {"type":"done","full_response":"..."}
Server -> Client: {"type":"error","message":"..."}
```

## ANTI-PATTERNS

- **Do not** skip signature verification for WhatsApp/Linq/Nextcloud Talk webhooks
- **Do not** log raw secrets, tokens, or webhook payloads — use `sanitize_api_error()` for error messages
- **Do not** change endpoint paths without updating both router and CLI/docs references
- **Do not** add new endpoints without rate limiting when they accept external input
- **Do not** expose `/api/config` without masking sensitive fields
- **Do not** bypass `require_auth()` in new `/api/*` handlers
- **Do not** change `MAX_BODY_SIZE` (64KB) without security review
