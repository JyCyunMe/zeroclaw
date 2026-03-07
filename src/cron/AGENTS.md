# AGENTS.md — Cron Scheduling Subsystem

Task execution engine for scheduled and recurring jobs with SQLite persistence.

## WHERE TO LOOK

- `mod.rs` — CLI command routing (`list/add/add-at/add-every/once/remove/update/pause/resume`)
- `types.rs` — `CronJob`, `Schedule` enum (`Cron`/`At`/`Every`), `JobType`, `DeliveryConfig`
- `schedule.rs` — Cron expression parsing, timezone handling, next-run calculation
- `store.rs` — SQLite persistence, schema migrations, run history pruning
- `scheduler.rs` — Execution loop, shell/agent job runners, delivery integration

## CONVENTIONS

**Schedule types:**
- `Schedule::Cron { expr, tz }` — standard cron with optional IANA timezone
- `Schedule::At { at }` — one-shot at specific datetime (auto `delete_after_run`)
- `Schedule::Every { every_ms }` — fixed interval

**Cron expressions:**
- Accepts 5-field (`minute hour day month weekday`) normalized to 6-field with seconds=0
- 6/7-field expressions passed through unchanged
- Timezone via `chrono_tz` for localized scheduling

**Persistence:**
- SQLite at `<workspace>/cron/jobs.db` with auto-migration for new columns
- Run history capped by `config.cron.max_run_history`, pruned on each insert
- Output truncated to 16KB with `...[truncated]` marker

**Execution:**
- Shell jobs: `sh -lc` with 120s timeout, workspace-scoped, security policy checked
- Agent jobs: invoke `agent::run` with prefixed prompt `[cron:<id> <name>]`
- Retry with exponential backoff (configurable `scheduler_retries`)
- One-shot jobs: delete on success, disable on failure

**Delivery:**
- `delivery.mode = "announce"` sends output to configured channel (telegram/discord/slack/mattermost)
- `best_effort = true` keeps job success even if delivery fails

**Testing:**
- Use `tempfile::TempDir` with `Config::default()` overrides
- Mock time with `Utc::now()` deltas for schedule testing
- Security policy tests cover: read-only mode, rate limits, forbidden paths, command allowlists

## ANTI-PATTERNS

- Do not skip `validate_schedule()` before persisting — invalid expressions cause runtime errors
- Do not bypass `SecurityPolicy::is_command_allowed()` for shell jobs
- Do not run agent jobs at sub-5-minute intervals without explicit warning (resource exhaustion)
- Do not store unbounded output — use `truncate_cron_output()` before DB insert
- Do not add new schedule types without updating `next_run_for_schedule()` and `validate_schedule()`
- Do not create direct DB connections — use `with_connection()` for schema consistency
