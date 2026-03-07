# src/agent — Orchestration Loop

Orchestration hub coordinating providers, tools, memory, and observability for autonomous agent turns.

## WHERE TO LOOK

- `agent.rs` — `Agent` struct, `AgentBuilder`, `turn()` lifecycle, `from_config()` factory
- `loop_.rs` — Production agent loop (`run_tool_call_loop`), tool-call parsing, credential scrubbing
- `dispatcher.rs` — `ToolDispatcher` trait, `NativeToolDispatcher`, `XmlToolDispatcher`
- `classifier.rs` — Query classification routing by hint (model selection hints)
- `prompt.rs` — System prompt assembly with identity, skills, tool instructions
- `memory_loader.rs` — Context loading from memory for conversation enrichment
- `tests.rs` — Comprehensive 1338-line test suite covering 20+ edge cases

## CONVENTIONS

- **Builder pattern**: `Agent::builder().provider(...).tools(...).memory(...).observer(...).build()` — all required fields validated at build time
- **Dispatcher selection**: Config key `agent.tool_dispatcher` chooses `native` (OpenAI function-calling) vs `xml` (tag-based); fallback uses `provider.supports_native_tools()`
- **Turn lifecycle**: System prompt → memory context → provider chat → tool calls → tool results → trim history → return text or iterate
- **Parallel tools**: `config.agent.parallel_tools` gates sequential vs concurrent tool execution
- **Observer events**: Emit `ToolCall`, `AgentStart`, `AgentEnd` via `Observer` trait for observability
- **Error handling**: Provider errors propagate; tool failures return structured `ToolResult` (not panics); max-iteration bailout with explicit error
- **History trimming**: `config.agent.max_history_messages` preserves system prompt, drops oldest non-system messages
- **Auto-compaction**: When history exceeds threshold, older messages summarized via LLM to compressed context

## TEST COVERAGE

`tests.rs` exercises full `Agent.turn()` cycles with mock providers:

- Simple text response, single/multi-tool chains, max-iteration bailout
- Unknown tool name recovery, tool execution failure, parallel dispatch
- History trimming, memory auto-save round-trip, native vs XML dispatcher
- Empty responses, mixed text+tools, multi-tool batches, system prompt generation
- Context enrichment, serialization round-trips, builder validation

Add new tests to `tests.rs` following the `ScriptedProvider` pattern; use `agent_e2e` test target for integration coverage.

## ANTI-PATTERNS

- **Do not** add new tool dispatch logic outside `dispatcher.rs` — all parsing/formatting lives there
- **Do not** bypass `AgentBuilder` validation — use `from_config()` or explicit builder calls
- **Do not** panic in tool execution paths — return `ToolResult { success: false, ... }`
- **Do not** log raw tool outputs containing secrets — use `scrub_credentials()` from `loop_.rs`
- **Do not** add provider-specific behavior to orchestration code — dispatchers abstract format differences
- **Do not** extend `ConversationMessage` enum without updating all dispatchers
