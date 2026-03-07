# AGENTS.md — SOP System (Standard Operating Procedures)

## OVERVIEW

SOP module implements trigger-driven, multi-step procedures with cooldown, concurrency control, and approval flows.

## WHERE TO LOOK

| Concern | File | Key Exports |
|---------|------|-------------|
| Core types | `types.rs` | `Sop`, `SopTrigger`, `SopStep`, `SopRun`, `SopRunAction` |
| Run lifecycle | `engine.rs` | `SopEngine` — trigger matching, run state, cooldown/concurrency |
| Event dispatch | `dispatch.rs` | `dispatch_sop_event()` — unified entry point for all trigger sources |
| Condition eval | `condition.rs` | `evaluate_condition()` — JSON path and direct numeric comparisons |
| Audit logging | `audit.rs` | `SopAuditLogger` — persists runs/steps to Memory backend |
| Metrics | `metrics.rs` | `SopMetricsCollector` — windowed and all-time counters |
| Trust-phase gates | `gates.rs` | `GateEvalState` — feature-gated (`ampersona-gates`) |

## CONVENTIONS

**SOP file format:**
- Each SOP lives in `<sops_dir>/<name>/` with `SOP.toml` (metadata + triggers) and `SOP.md` (procedure steps).
- `SOP.toml` defines: `name`, `description`, `priority`, `execution_mode`, `cooldown_secs`, `max_concurrent`, `[[triggers]]`.
- `SOP.md` uses `## Steps` section with numbered items: `1. **Title** — Body`. Sub-bullets: `- tools: tool1, tool2`, `- requires_confirmation: true`.

**Trigger types:**
- `Mqtt { topic, condition }` — MQTT topic subscription with optional JSON path condition.
- `Webhook { path }` — HTTP POST to `/webhook/<path>`.
- `Cron { expression }` — Standard cron expression, integrates with scheduler.
- `Peripheral { board, signal, condition }` — Hardware signal trigger with comparison.
- `Manual` — Explicitly triggered via CLI or API.

**Execution modes:**
- `Auto` — No approval required.
- `Supervised` — Approval before first step only.
- `StepByStep` — Approval before each step.
- `PriorityBased` — Auto for Critical/High, Supervised for Normal/Low.

**Dispatch flow:**
1. Event arrives (MQTT, webhook, cron, peripheral).
2. `dispatch_sop_event()` locks engine, matches triggers, starts runs.
3. Each run yields `SopRunAction`: `ExecuteStep`, `WaitApproval`, `Completed`, or `Failed`.
4. Audit logger persists run/step state to Memory backend.

**HEARTBEAT.md format:**
- Located at `<workspace>/HEARTBEAT.md`.
- Lines starting with `- ` are parsed as periodic tasks.
- Heartbeat engine (`src/heartbeat/`) reads and collects tasks on each tick.
- Config: `[heartbeat] enabled = true, interval_minutes = 30, target = "telegram"`.

**Testing requirements:**
- Unit tests for `parse_steps()` in `mod.rs` cover markdown edge cases.
- Unit tests for `evaluate_condition()` cover JSON path and numeric comparisons.
- Integration tests use `tempfile` for SOP directories.
- All trigger types must round-trip through TOML serialization.

## ANTI-PATTERNS

- Do NOT bypass `dispatch_sop_event()` — it handles locking, audit, and health bookkeeping.
- Do NOT skip cooldown/concurrency checks — use `engine.can_start()` before `start_run()`.
- Do NOT ignore `SopRunAction` — callers must execute or audit the returned action.
- Do NOT add new trigger types without updating `SopTriggerSource` and `dispatch.rs`.
- Do NOT store sensitive data in SOP definitions — use secrets store for credentials.
- Do NOT block the engine mutex for long operations — dispatch uses two-phase locking pattern.
