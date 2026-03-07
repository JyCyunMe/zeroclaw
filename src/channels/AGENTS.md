# AGENTS.md — src/channels

## OVERVIEW

Multi-channel messaging subsystem connecting ZeroClaw to 17+ platforms via the `Channel` trait.

## STRUCTURE

```
src/channels/
├── traits.rs        # Channel trait + ChannelMessage/SendMessage types
├── mod.rs           # Factory exports, start_channels orchestration, runtime loop
├── telegram.rs      # Telegram Bot API (polling + webhook)
├── discord.rs       # Discord Gateway + REST
├── slack.rs         # Slack RTM/Events API
├── mattermost.rs    # Mattermost API v4
├── imessage.rs      # macOS-only AppleScript bridge
├── matrix.rs        # Matrix homeserver (feature-gated)
├── signal.rs        # Signal CLI bridge
├── whatsapp.rs      # WhatsApp Business Cloud API
├── whatsapp_web.rs  # WhatsApp Web QR/pair (feature-gated)
├── nostr.rs         # Nostr NIP-04/17 DMs
├── irc.rs           # IRC protocol
├── lark.rs          # Lark/Feishu (feature-gated)
├── dingtalk.rs      # DingTalk
├── qq.rs            # Tencent QQ
├── email_channel.rs # SMTP/IMAP
├── linq.rs          # Linq protocol
├── cli.rs           # Stdin/stdout loop
└── wati.rs          # WATI WhatsApp API
```

## WHERE TO LOOK

- **Adding a channel:** Start with `traits.rs` for trait contract, then an existing impl like `telegram.rs` or `cli.rs` as reference.
- **Allowlist/auth logic:** `telegram.rs` (`is_user_allowed`), `imessage.rs` (`is_contact_allowed`), `discord.rs` (authorizer checks).
- **Factory wiring:** `mod.rs` exports via `pub use` and `start_channels` instantiation.
- **Typing indicators:** `traits.rs` default stubs; see `telegram.rs` and `discord.rs` for real impls.
- **Thread handling:** `ChannelMessage.thread_ts` and `SendMessage::in_thread()` for Slack/Discord thread replies.
- **Media attachments:** `telegram.rs` parses `[IMAGE:...]`, `[DOCUMENT:...]` markers.

## CONVENTIONS

- **Channel trait:** Must implement `name()`, `send()`, `listen()`. Default impls exist for `health_check()`, `start_typing()`, `stop_typing()`, draft methods, reactions.
- **Naming:** `<Platform>Channel` (e.g., `TelegramChannel`, `DiscordChannel`).
- **Allowlist semantics:** Empty list = deny all; `"*"` = allow all; otherwise exact match. Check with `is_user_allowed()` / `is_contact_allowed()`.
- **Deny-by-default:** Validate sender identity before processing any inbound message. Log rejections with sender ID.
- **Tests required:** Auth/allowlist validation, health check, send/listen happy path. See `traits.rs` test block for minimal pattern.
- **Feature gates:** Some channels behind `channel-<name>` features (e.g., `channel-lark`, `channel-matrix`, `whatsapp-web`).
- **Platform constraints:** `iMessageChannel` only works on macOS (requires Messages database + AppleScript). Document platform requirements in module docs.

## ANTI-PATTERNS

- Do not bypass allowlist checks in `listen()` — always validate sender before emitting to channel.
- Do not log raw API tokens, chat IDs, or sender identities at INFO level; use DEBUG or redact.
- Do not panic in `listen()` — return `Err` and let the supervisor handle reconnection.
- Do not hardcode platform-specific limits (e.g., Telegram 4096 chars); define constants.
- Do not introduce new channel config keys without updating `docs/channels-reference.md`.
- Do not add new channels without allowlist support and deny-by-default semantics.
