# AGENTS.md — Observability Subsystem

Extends: [`AGENTS.md`](../../AGENTS.md) (parent protocol)

## OVERVIEW

Pluggable observability via `Observer` trait: Noop, Log, Multi, Prometheus, OTel, Verbose, and runtime trace backends.

## WHERE TO LOOK

- `traits.rs` — `Observer` trait, `ObserverEvent`, `ObserverMetric` enums
- `mod.rs` — factory `create_observer()`, config-to-backend mapping
- `noop.rs` — zero-overhead passthrough (default fallback)
- `log.rs` — tracing-based structured logging
- `multi.rs` — fan-out to multiple observers
- `prometheus.rs` — Prometheus metrics exposition
- `otel.rs` — OpenTelemetry/OTLP (feature-gated: `observability-otel`)
- `verbose.rs` — human-readable CLI progress
- `runtime_trace.rs` — JSONL file-based diagnostics

## CONVENTIONS

**Observer trait implementation:**
- Must implement `Send + Sync + 'static` (shared via `Arc` across async tasks)
- `record_event` and `record_metric` run on hot paths: avoid blocking I/O
- Buffer internally and flush asynchronously when possible
- Name must be lowercase and stable (e.g., `"log"`, `"prometheus"`)
- Use `#[inline(always)]` for Noop-style pass-through methods

**What to LOG:**
- Event type, provider/model names, durations, success flags
- Token counts (aggregated), message counts, channel names
- Component identifiers for error routing

**What to NEVER LOG:**
- API keys, access tokens, secrets, credentials
- Raw prompt content, response bodies, user messages
- Session IDs that could be correlated to sensitive data
- Internal state that could reveal configuration secrets

**Error events:**
- `ObserverEvent::Error { message }` must contain only sanitized descriptions
- Never embed raw error strings that might contain tokens or paths

**Test coverage:**
- All observer types must have unit tests for every event variant
- Test empty inputs, zero durations, and missing optional fields
- Verify factory fallback to Noop for unknown backends
- OTel tests must handle unreachable endpoints gracefully

**Performance:**
- NoopObserver compiles to nothing via `#[inline(always)]`
- Observers should not allocate on the hot path
- Prometheus observer uses pre-allocated metric handles
- Avoid string formatting in tight loops inside record methods

## ANTI-PATTERNS

- Do NOT log prompt/response content or user data
- Do NOT block on I/O inside `record_event` or `record_metric`
- Do NOT add new `ObserverEvent` variants that carry sensitive fields
- Do NOT bypass the factory — observers are config-driven
- Do NOT create cross-subsystem coupling (observer should not import provider/channel internals)
- Do NOT panic in observer methods — errors should be logged and swallowed
