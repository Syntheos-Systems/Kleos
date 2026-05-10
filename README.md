<div align="center">

<img src="docs/assets/banner.svg" width="800" alt="Kleos Banner" />

# Kleos

**Give your AI a memory it doesn't lose between sessions. One Rust binary. Runs on your hardware.**

[![License](https://img.shields.io/badge/License-Elastic--2.0-blue.svg)](LICENSE) [![Rust](https://img.shields.io/badge/rust-1.94%2B-orange.svg)](https://www.rust-lang.org) [![Native](https://img.shields.io/badge/Native-No%20Python-blueviolet.svg)](#)

[Quickstart](https://github.com/Ghost-Frame/Kleos/wiki/Quickstart) · [Why Kleos](https://github.com/Ghost-Frame/Kleos/wiki/Why-Kleos) · [What's in the box](#whats-in-the-box) · [Workspace](#workspace) · [Wiki](https://github.com/Ghost-Frame/Kleos/wiki)

</div>

---

Most AI agents are amnesiacs. They re-learn the same facts every conversation, lose track of what they tried yesterday, and forget the user's name the moment the prompt window scrolls. The usual fix is a vector database stapled to a Python memory class: fragile glue, latency tax, still no actual structure to what the agent knows.

Kleos is the persistent brain that sits behind the agent. It remembers what was said, decays old facts the way humans do, builds a graph of who relates to what, surfaces contradictions when the agent learns something new, and exposes itself through one local binary that speaks the Model Context Protocol natively. No Python runtime. No managed cloud. Your data on your disk, optionally sealed to your YubiKey.

![Kleos CLI demo](tools/cli-demo.gif)

![Kleos memory graph visualization](tools/gui-demo-v3.gif)

---

<details>
<summary><strong>I want to use this -- show me the quickstart</strong></summary>

```bash
git clone https://github.com/Ghost-Frame/Kleos.git && cd Kleos
cargo build --release
KLEOS_BOOTSTRAP_SECRET=my-setup-secret ./target/release/kleos-server
```

The server binds to `127.0.0.1:4200`. In another shell, claim the admin key and store your first memory:

```bash
# One-time bootstrap mints the admin key
curl -X POST http://localhost:4200/bootstrap \
  -H "Content-Type: application/json" \
  -d '{"secret": "my-setup-secret"}'

# Store
curl -X POST http://localhost:4200/store \
  -H "Authorization: Bearer eg_YOUR_KEY" \
  -H "Content-Type: application/json" \
  -d '{"content": "Production DB is PostgreSQL 16 on db.example.com:5432", "category": "reference"}'

# Search
curl -X POST http://localhost:4200/search \
  -H "Authorization: Bearer eg_YOUR_KEY" \
  -H "Content-Type: application/json" \
  -d '{"query": "database connection info"}'
```

MCP wiring, encryption setup, sidecar, hooks, and SDKs live in the [Quickstart](https://github.com/Ghost-Frame/Kleos/wiki/Quickstart) and [Integration Guides](https://github.com/Ghost-Frame/Kleos/wiki/Integration-Guides). Day-to-day operations are in the [Operations Manual](docs/KLEOS_OPERATIONS_MANUAL.md).

</details>

<details>
<summary><strong>I'm evaluating this for a team -- prove it's serious</strong></summary>

Kleos is a 16-crate Rust workspace, roughly 204K lines of code with about 6,000 test declarations across 113 test files. It ships as a single statically linked binary with the mimalloc allocator. No Python runtime, no Docker requirement.

**Surface.** The HTTP server exposes 47 route modules behind 8 middleware layers (auth, per-tenant rate limit, audit log, client IP, JSON depth limit, Prometheus metrics, safe-mode, tower-http for compression and timeouts). The MCP server speaks 57 tools across memory, context, graph, intelligence, services, structural analysis, skills, and admin. Client SDKs are first-party: TypeScript, Python (Pydantic v2 + httpx), Go (stdlib only).

**Multi-tenancy.** Every tenant gets its own SQLite database, its own deadpool reader and writer pools, and quota enforcement on bytes and memory count. 44 numbered schema migrations, all forward-compatible. `user_id` is enforced on every query, including parent lookups during versioning.

**Security.** SQLCipher at rest, with the database key resolved from a keyfile, an environment variable, or a YubiKey HMAC-SHA1 challenge-response. Bearer tokens are SHA-256 hashed and peppered. Request signing uses ECDSA-P256 or Ed25519 with a canonical KLEOSv1 envelope and replay-resistant nonces. A separate credential daemon (`kleos-credd`) holds secrets in an AES-256-GCM vault with a two-tier master and agent key model. The pre-action gate system can allow, warn, or block agent commands and routes risky ones into a human-in-the-loop approval queue. Audit log on every mutation, response hardening headers by default, automatic safe-mode write block on crash-loop detection.

**Resilience.** Every external call (LLM, reranker) is wrapped in a `ServiceGuard` combining a three-state circuit breaker (Closed, Open, Half-Open), exponential-backoff retry with jitter and smart error classification, and a SQLite-backed dead-letter queue so failed work can be inspected and replayed.

**Coordination services.** Seven of them, each a real database-backed service, not a stub:

- **Axon** -- event bus with channels, subscriptions, retention windows, cursors
- **Broca** -- structured action log
- **Chiasm** -- task tracker with retries and timeouts
- **Soma** -- agent registry with heartbeats
- **Loom** -- DAG workflow engine (action, decision, parallel, wait, webhook, llm, transform steps)
- **Thymus** -- quality evaluation with memory feedback loops
- **Brain** -- async subprocess orchestration for the cognitive backend

</details>

<details>
<summary><strong>I'm into the cognitive-science angle</strong></summary>

**FSRS-6 spaced repetition.** All 21 trained weights from the open Free Spaced Repetition Scheduler are baked in. Each memory carries a `DualStrength` tuple: storage strength (long-term encoding) accumulates with use; retrieval strength (current accessibility) decays via power-law forgetting. Live retrievability modulates every search candidate's score in real time.

**4-channel hybrid retrieval.** A query fans out across vector similarity (BAAI/bge-m3 ONNX embeddings, 1024-dim, LanceDB ANN when enabled), FTS5 BM25, knowledge-graph traversal (2-hop, weighted by typed edges and PageRank), and personality signals. The four channels merge through Reciprocal Rank Fusion. Question-type detection classifies the query into one of five intents (fact recall, preference, reasoning, generalization, timeline) and reweights the channels per intent before the IBM Granite cross-encoder reranks the top-K. Pattern-based contradiction damping suppresses memories that contain "no longer," "switched from," and similar markers.

**Knowledge graph that earns the name.** Auto-linking by cosine similarity, typed edges (similarity, updates, extends, contradicts, caused_by, prerequisite_for) with link-type weights so a "causes" edge counts more than an "extends" edge. Louvain community detection capped at 10K nodes per pass. Weighted PageRank with deduplicated multi-edges. Sliding-window entity cooccurrence. Structural and impact analysis.

**Cognitive backend (feature-gated `brain`).** A modern continuous Hopfield network (Ramsauer et al. 2020) with softmax attention, default temperature 8.0, and capacity exponential in dimension rather than the classical `0.14N`. Embedding compression via PCA implemented with power iteration and deflation, no LAPACK dependency. A six-stage dream cycle replays, merges, prunes, discovers, decorrelates, and resolves stored patterns while the agent rests.

**Personality engine.** Six signal types extracted from memory text: preference, value, motivation, decision, emotion, identity. Each signal carries a subject, a valence (positive, negative, neutral, mixed), an intensity in `[0, 1]`, and the reasoning that produced it. Recall is shaped by the agent's current personality context.

**Things real engineers will also care about.** SimHash near-duplicate suppression on store. SVO contradiction detection against the structured-fact triple store. Atomic fact decomposition (LLM-first with rule-based and template fallbacks) splitting long memories into self-contained facts linked back via `has_fact`. Async job queue for PageRank refresh, vector sync replay, auto-checkpoint, and auto-backup. Agent-Forge: a structured reasoning CLI (spec-task, log-hypothesis, verify, challenge-code, repo-map, search-code, and 14 more subcommands) with Tree-sitter AST parsing for Rust, TypeScript, JavaScript, Python, Go, C/C++, JSON.

</details>

---

## What's in the Box

| Component | Role | Highlights |
| --- | --- | --- |
| **kleos-server** | The Brain | FSRS-6, 4-channel hybrid search, 7 coordination services, 47 route modules |
| **agent-forge** | The Discipline | Tree-sitter AST repo mapping, structured reasoning, adversarial review |
| **kleos-mcp** | The Interface | 57 MCP tools for direct LLM integration over stdio |
| **kleos-credd** | The Shield | Hardware-backed AES-256-GCM vault with YubiKey HMAC-SHA1 |
| **kleos-sidecar** | The Guardian | Session-scoped memory proxy with batched observation flushing |
| **kleos-ingest** | The Collector | Transcript ingest daemon with PIV and software-key request signing |

---

## Workspace

Sixteen Cargo crates:

| Crate | Role |
| --- | --- |
| `kleos-lib` | Core library. Memory, search, embeddings, graph, intelligence, services, auth, jobs across 51 modules plus the feature-gated `brain` backend. Previously published as `engram-lib` (last `0.3.1`). |
| `kleos-server` | Axum HTTP server. 47 route modules, 8 middleware layers, embedded GUI. |
| `kleos-cli` | Command-line client over the HTTP API. Memory ops, skill management, credential management via credd. |
| `kleos-sidecar` | Session-scoped memory proxy with file watcher, batched flushing, compression, persistent session store. |
| `kleos-mcp` | Model Context Protocol server. 57 tools across memory, context, graph, intelligence, services, structural, skills, admin. Stdio transport with HTTP behind a feature flag. |
| `kleos-cred` | Credential management library. Crypto primitives, YubiKey challenge-response, key derivation, CRED:v3 vault resolution. |
| `kleos-credd` | Credential management daemon. HTTP server with master + agent two-tier auth, AES-256-GCM encryption. |
| `kleos-approval-tui` | Ratatui-based approval queue TUI (WIP). |
| `kleos-migrate` | One-shot ETL: encrypted SQLCipher monolith into per-tenant shards. |
| `kleos-sh` | Shell command gate wrapper. Checks commands through Kleos before execution. |
| `kleos-fs` | Filesystem helper binaries for guarded read and write operations. |
| `eidolon-supervisor` | Local supervisor for Eidolon and Kleos agent-host process coordination. |
| `kleos-ingest` | Transcript ingest daemon with PIV and software-key request signing and real-time observation streaming. |
| `kleos-cleanup` | One-shot cleanup utility for deduplicating growth rows and demoting activity logs out of the main memory table. |
| `agent-forge` | Structured reasoning CLI with 20+ subcommands including spec-task, log-hypothesis, verify, challenge-code, session-diff, repo-map, search-code. Tree-sitter AST for 7 languages. |
| `sdk` | First-party client SDKs: TypeScript, Python, Go. |

```bash
cargo build --release --workspace                                                # build everything
cargo test --workspace --exclude kleos-migrate                                   # CI test suite
cargo clippy --workspace --exclude kleos-migrate --all-targets -- -D warnings    # lint gate
```

---

<div align="center">

[Wiki](https://github.com/Ghost-Frame/Kleos/wiki) · [Issues](https://github.com/Ghost-Frame/Kleos/issues)

Elastic License 2.0

</div>
