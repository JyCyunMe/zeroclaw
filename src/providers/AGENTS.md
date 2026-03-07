# AGENTS.md — Provider Subsystem

Factory-based model inference backends for AI providers. Each provider implements the `Provider` trait and is registered in `mod.rs`.

## WHERE TO LOOK

- `traits.rs` — `Provider` trait, `ChatMessage`, `ChatResponse`, `ToolCall`, streaming types
- `mod.rs` — Factory functions (`create_provider`, `create_resilient_provider`), alias resolution, credential handling
- `compatible.rs` — Generic OpenAI-compatible provider for `/v1/chat/completions` APIs
- `reliable.rs` — `ReliableProvider` wrapper with retry, fallback, and error classification
- `router.rs` — `RouterProvider` for model-based routing across providers

## CONVENTIONS

- **Trait + factory architecture**: implement `Provider` in a submodule, register in `create_provider_with_url_and_options`
- **Provider naming**: `<ProviderName>Provider` (e.g., `OpenAiProvider`, `AnthropicProvider`)
- **Factory keys**: lowercase, user-facing (e.g., `"openai"`, `"anthropic"`), aliases resolved via helper functions
- **Custom endpoints**: `custom:https://...` for OpenAI-compatible, `anthropic-custom:https://...` for Anthropic-compatible
- **Credential resolution**: explicit param > provider-specific env var > generic fallback (`ZEROCLAW_API_KEY`)
- **Error sanitization**: use `sanitize_api_error()` to scrub secrets and truncate messages
- **Secrets**: never log raw API keys; store encrypted via `AuthService` for subscription-based providers
- **Tests**: cover factory wiring, alias resolution, and error classification in `reliable.rs`

## ANTI-PATTERNS

- Leaking provider-specific behavior into shared orchestration code (agent loop, tool execution)
- Adding new provider aliases without updating `canonical_china_provider_name()` for i18n routing
- Hardcoding base URLs in multiple places — use constants at module top
- Returning panics from provider methods — use `anyhow::Result` and explicit error propagation
- Bypassing `resolve_provider_credential()` for fallback providers — each must resolve its own key
- Ignoring `supports_native_tools()` when implementing tool-aware providers
