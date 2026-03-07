# AGENTS.md — Memory Module

Custom hybrid search engine with zero external vector/keyword dependencies.

## WHERE TO LOOK

| Concern | File | Notes |
|---------|------|-------|
| Core trait | `traits.rs` | `Memory` trait, `MemoryEntry`, `MemoryCategory` |
| SQLite backend | `sqlite.rs` | FTS5 + vector + embedding cache, WAL PRAGMAs |
| Vector ops | `vector.rs` | Cosine similarity, hybrid merge weights |
| Embeddings | `embeddings.rs` | `EmbeddingProvider` trait, OpenAI/custom/noop |
| Chunking | `chunker.rs` | Markdown heading-aware, paragraph fallback |
| Backend types | `backend.rs` | `MemoryBackendKind`, profile metadata |
| Factory | `mod.rs` | `create_memory()`, embedding route resolution |
| Response cache | `response_cache.rs` | LLM response dedup, TTL eviction |

## CONVENTIONS

- **Implement `Memory` trait** for new backends: `store`, `recall`, `get`, `list`, `forget`, `count`, `health_check`.
- **Backend keys**: lowercase, stable (`sqlite`, `lucid`, `postgres`, `qdrant`, `markdown`, `none`).
- **Embedding providers**: implement `EmbeddingProvider::embed()`, register in `create_embedding_provider()`.
- **Hybrid search**: default `vector_weight=0.7`, `keyword_weight=0.3`; normalize before merging.
- **Embedding cache**: `embedding_cache` table with LRU eviction (default 10k entries).
- **Chunking**: line-based markdown, ~4 chars per token estimate, preserve heading context.
- **Postgres**: gated behind `memory-postgres` feature flag.
- **Tests**: cover factory wiring, backend CRUD, embedding fallbacks, hybrid merge edge cases.

## ANTI-PATTERNS

- Do not leak embedding API keys into logs or `Debug` output.
- Do not skip FTS5 trigger sync on insert/update/delete.
- Do not hardcode vector dimensions; respect `EmbeddingProvider::dimensions()`.
- Do not store embeddings without checking `EmbeddingProvider::name() != "none"`.
- Do not use postgres backend without `--features memory-postgres`.
- Do not call embedding APIs synchronously in hot paths; use cache first.
