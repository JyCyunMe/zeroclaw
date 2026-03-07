# ZeroClaw Commands Reference

## Overview

ZeroClaw has three types of commands:
1. **CLI Commands** - Terminal commands for managing the runtime
2. **Runtime Chat Commands** - In-chat commands for Telegram/Discord
3. **Slash Commands** (Planned) - Native Discord/Telegram slash commands

---

## 1. CLI Commands

### 1.1 Core Commands

| Command | Description |
|---------|-------------|
| `zeroclaw onboard` | Initialize workspace and configuration |
| `zeroclaw agent` | Start AI agent interactive loop |
| `zeroclaw gateway` | Start gateway server (webhooks, websockets) |
| `zeroclaw daemon` | Start full runtime (gateway + channels + scheduler) |
| `zeroclaw status` | Show system status |
| `zeroclaw providers` | List supported AI providers |

---

### 1.2 Service Subcommands

| Command | Description |
|---------|-------------|
| `zeroclaw service install` | Install OS service (systemd/launchd) |
| `zeroclaw service start` | Start daemon service |
| `zeroclaw service stop` | Stop daemon service |
| `zeroclaw service restart` | Restart daemon service |
| `zeroclaw service status` | Check daemon service status |
| `zeroclaw service uninstall` | Uninstall daemon service unit |

---

### 1.3 Channel Subcommands

| Command | Description |
|---------|-------------|
| `zeroclaw channel list` | List all configured channels |
| `zeroclaw channel start` | Start all configured channels |
| `zeroclaw channel doctor` | Run health checks for channels |
| `zeroclaw channel add <type> <json>` | Add new channel configuration |
| `zeroclaw channel remove <name>` | Remove a channel |
| `zeroclaw channel bind-telegram <identity>` | Bind Telegram user to allowlist |
| `zeroclaw channel bind-discord <user_id>` | Bind Discord user to allowlist |

**Supported channel types**: telegram, discord, slack, whatsapp, matrix, imessage, email

---

### 1.4 Model Subcommands

| Command | Description |
|---------|-------------|
| `zeroclaw models refresh` | Refresh and cache provider models |
| `zeroclaw models list` | List cached models for a provider |
| `zeroclaw models set <model>` | Set default model in config |
| `zeroclaw models status` | Show current model configuration |

---

### 1.5 Auth Subcommands

| Command | Description |
|---------|-------------|
| `zeroclaw auth login --provider <name>` | Login with OAuth (openai-codex, gemini) |
| `zeroclaw auth paste-redirect` | Complete OAuth by pasting redirect URL |
| `zeroclaw auth paste-token` | Paste auth token (for Anthropic) |
| `zeroclaw auth setup-token` | Setup token interactively |
| `zeroclaw auth refresh` | Refresh access token |
| `zeroclaw auth logout` | Remove auth profile |
| `zeroclaw auth list` | List auth profiles |
| `zeroclaw auth status` | Show auth status and token expiry |

---

### 1.6 Doctor Subcommands

| Command | Description |
|---------|-------------|
| `zeroclaw doctor models` | Probe model catalogs across providers |
| `zeroclaw doctor traces` | Query runtime trace events |

---

### 1.7 Cron Subcommands

| Command | Description |
|---------|-------------|
| `zeroclaw cron list` | List all scheduled tasks |
| `zeroclaw cron add <expr> <cmd>` | Add recurring task (cron expression) |
| `zeroclaw cron add-at <time> <cmd>` | Add one-shot task (RFC 3339 timestamp) |
| `zeroclaw cron add-every <ms> <cmd>` | Add interval-based task |
| `zeroclaw cron once <duration> <cmd>` | Execute once after delay |
| `zeroclaw cron pause <id>` | Pause a task |
| `zeroclaw cron resume <id>` | Resume a paused task |
| `zeroclaw cron remove <id>` | Remove a task |
| `zeroclaw cron update <id>` | Update a task |

**Cron format**: `min hour day month weekday` (5-field, UTC by default)

---

### 1.8 Memory Subcommands

| Command | Description |
|---------|-------------|
| `zeroclaw memory list` | List memory entries with filters |
| `zeroclaw memory get <key>` | Get specific memory entry |
| `zeroclaw memory stats` | Show memory backend statistics |
| `zeroclaw memory clear` | Clear memories by category or key |

---

### 1.9 Hardware Subcommands

| Command | Description |
|---------|-------------|
| `zeroclaw hardware discover` | Enumerate USB devices and known boards |
| `zeroclaw hardware introspect <path>` | Introspect device by path |
| `zeroclaw hardware info` | Get chip info via USB (probe-rs) |

---

### 1.10 Peripheral Subcommands

| Command | Description |
|---------|-------------|
| `zeroclaw peripheral list` | List configured peripherals |
| `zeroclaw peripheral add <board> <path>` | Add a peripheral |
| `zeroclaw peripheral remove <name>` | Remove a peripheral |
| `zeroclaw peripheral flash` | Flash firmware to board |

**Supported boards**: nucleo-f401re, rpi-gpio, esp32, arduino-uno

---

### 1.11 Skills Subcommands

| Command | Description |
|---------|-------------|
| `zeroclaw skills list` | List installed skills |
| `zeroclaw skills audit <source>` | Audit a skill |
| `zeroclaw skills install <source>` | Install a skill |
| `zeroclaw skills remove <name>` | Remove a skill |

---

### 1.12 Integrations Subcommands

| Command | Description |
|---------|-------------|
| `zeroclaw integrations info <name>` | Show integration details |

---

### 1.13 Migrate Subcommands

| Command | Description |
|---------|-------------|
| `zeroclaw migrate openclaw` | Import memory from OpenClaw workspace |

---

### 1.14 Estop Subcommands

| Command | Description |
|---------|-------------|
| `zeroclaw estop` | Trigger emergency stop |
| `zeroclaw estop status` | Show estop status |
| `zeroclaw estop resume` | Resume from estop state |

**Estop levels**:
- `network-kill` - Block all network access
- `domain-block` - Block specific domains
- `tool-freeze` - Freeze specific tools

---

## 2. Runtime Chat Commands (Telegram / Discord)

These commands work in Telegram and Discord chats when channels are running.

| Command | Description |
|---------|-------------|
| `/models` | Show available providers and current selection |
| `/models <provider>` | Switch provider for current sender session |
| `/model` | Show current model and cached model IDs |
| `/model <model-id>` | Switch model for current sender session |
| `/new` | Clear conversation history, start fresh session |
| `/bind <code>` | Bind account with pairing code (pairing mode only) |

**Notes**:
- Provider/model switches are sender-scoped (per-user)
- Switching provider or model clears that sender's conversation history
- `/bind` is only available when pairing mode is active (empty allowlist)

---

## 3. Slash Commands (Planned)

Native platform slash commands for Discord and Telegram.

### Discord Slash Command Mapping

| Slash Command | Options | Maps to |
|---------------|---------|---------|
| `/models` | none | `ShowProviders` |
| `/model` | `id: string?` | `ShowModel` / `SetModel` |
| `/new` | none | `NewSession` |
| `/bind` | `code: string (required)` | Pairing bind |

### Implementation Status

- [ ] CommandDefinition data structure
- [ ] Discord Application Command registration API
- [ ] INTERACTION_CREATE event handling
- [ ] Interaction callback response
- [ ] Command mapping
- [ ] Configuration toggle
- [ ] Documentation update
- [ ] Testing

---

## Summary

| Category | Count |
|----------|-------|
| CLI main commands | 15 |
| CLI subcommands | 60+ |
| Runtime chat commands | 6 |
| Slash commands (planned) | 4 |

---

## Reference

- CLI definitions: `src/main.rs`, `src/lib.rs`
- Runtime commands: `src/channels/mod.rs`
- Channel implementations: `src/channels/*.rs`
