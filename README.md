<div align="center">

<img src="docs/assets/banner.svg" width="800" alt="Kleos Banner" />

# Kleos

**The operating system for AI agents. Memory, coordination, skills, security, and supervision in one binary.**

[![License](https://img.shields.io/badge/License-Elastic--2.0-blue.svg)](LICENSE) [![Rust](https://img.shields.io/badge/rust-1.94%2B-orange.svg)](https://www.rust-lang.org) [![Native](https://img.shields.io/badge/Native-No%20Python-blueviolet.svg)](#)

[Quickstart](https://github.com/Ghost-Frame/Kleos/wiki/Getting-Started) · [Why Kleos](https://github.com/Ghost-Frame/Kleos/wiki/Why-Kleos) · [Wiki](https://github.com/Ghost-Frame/Kleos/wiki)

</div>

---

AI agents today can write code, search the web, and hold a conversation. But they can't remember what they did yesterday. They can't coordinate with other agents. They can't learn from their own mistakes, and nobody is watching whether they're drifting off-task. You end up gluing together a vector database, a task queue, a credential store, and a supervision layer, all separate services, all separately configured, all separately failing.

Kleos replaces that stack. It gives your AI agent long-term memory that decays like a human's, a knowledge graph that tracks how ideas connect, coordination services for tasks and workflows, a skill system that evolves over time, a credential vault sealed to a hardware security key, and a supervisor that watches agent sessions for drift. You run it on your own machine as a single binary. Your data never leaves your hardware.

![Kleos CLI demo](tools/cli-demo.gif)

![Kleos memory graph visualization](tools/gui-demo-v3.gif)

---

## What Kleos does

Kleos has six major systems. Most agent infrastructure gives you one of these and tells you to build the rest yourself.

**Memory.** Store observations. Search by meaning across four channels (vector similarity, full-text, graph traversal, personality signals). Memories strengthen with use and fade when ignored, using the same spaced-repetition algorithm behind Anki. Near-duplicates are caught on store. Contradictions are flagged. Long memories are broken into atomic facts automatically.

**Knowledge graph.** Memories link to each other with typed edges (updates, contradicts, caused_by, prerequisite_for, and more). Community detection finds clusters. PageRank surfaces the most connected ideas. Your agent doesn't just recall. It can traverse.

**Coordination.** Seven built-in services handle the work that usually requires a separate message broker: an event bus, an action log, a task tracker, an agent registry, a DAG workflow engine, a quality evaluation system, and a cognitive backend. One API call fans out to all of them.

**Skills and growth.** Agents store skills with version history, execution tracking, and trust scores. Skills can be evolved, fixed, or derived by an LLM. A growth system lets agents reflect on their own patterns and materialize insights into new memories. A cloud library lets skills be shared.

**Security.** Databases are encrypted with SQLCipher. The encryption key can come from a file, an environment variable, or a hardware security key. A separate credential daemon holds secrets in an AES-256-GCM vault with a two-tier key model. A policy gate checks every agent command before it runs and routes risky operations into a human approval queue. Request signing supports PIV certificates. Every mutation is audit-logged.

**Supervision.** The Eidolon supervisor watches agent sessions in real time, detects behavioral drift against configurable rules, and alerts the server. The server can inject corrections or block the agent's next action. Shell commands pass through a validation layer that blocks destructive patterns and SSRF attempts before execution reaches the host.

---

<details>
<summary><strong>Quick start</strong></summary>

```bash
git clone https://github.com/Ghost-Frame/Kleos.git && cd Kleos
cargo build --release
KLEOS_BOOTSTRAP_SECRET=pick-a-secret ./target/release/kleos-server
```

Server starts on `127.0.0.1:4200`. In another terminal:

```bash
# Mint your admin key (one-time setup)
curl -X POST http://localhost:4200/bootstrap \
  -H "Content-Type: application/json" \
  -d '{"secret": "pick-a-secret"}'

# Store a memory
curl -X POST http://localhost:4200/store \
  -H "Authorization: Bearer YOUR_KEY" \
  -H "Content-Type: application/json" \
  -d '{"content": "Production DB is PostgreSQL 16 on db.example.com:5432", "category": "reference"}'

# Search by meaning
curl -X POST http://localhost:4200/search \
  -H "Authorization: Bearer YOUR_KEY" \
  -H "Content-Type: application/json" \
  -d '{"query": "database connection info"}'
```

For Claude Code hook integration, encryption setup, the session sidecar, and client SDKs, see the [Getting Started](https://github.com/Ghost-Frame/Kleos/wiki/Getting-Started) guide.

</details>

<details>
<summary><strong>For developers: integration and architecture</strong></summary>

### Connecting your agent

Kleos speaks three protocols:

1. **HTTP API.** 56 route modules covering memory, search, graph, coordination, skills, growth, ingestion, approvals, and admin. Bearer token or signed-request auth.
2. **Model Context Protocol (MCP).** 59 tools over stdio. Drop it into Claude Code, Cursor, or any MCP-compatible client.
3. **Client SDKs.** TypeScript, Python (Pydantic v2 + httpx), Go (stdlib only). First-party, typed, tested.

### Claude Code integration

Kleos ships ready-to-use hooks in `hooks/`. Two versions:

- **Simple.** Session start/end, per-turn memory recall, tool observation. Bash + curl.
- **Full.** Adds Eidolon brain-aware context, automatic sidecar startup, Chiasm task tracking, growth materialization, Agent-Forge protocol enforcement, and session quality scoring.

Copy the hooks, configure `settings.json`, and your Claude Code agent has persistent memory and coordination across sessions.

### What runs inside

16-crate Rust workspace. The server handles:

- **Multi-tenancy.** Each tenant gets its own encrypted SQLite database, connection pools, and quota limits.
- **8 middleware layers.** Auth, per-tenant rate limiting, audit log, IP extraction, JSON depth limits, Prometheus metrics, safe-mode, compression/timeouts.
- **7 coordination services.** Event bus (Axon), action log (Broca), task tracker (Chiasm), agent registry (Soma), DAG workflow engine (Loom), quality evaluation (Thymus), cognitive backend (Brain).
- **Skills platform.** CRUD, versioned evolution, execution tracking, trust scoring, judgments, cloud library, LLM-backed fix/derive/capture.
- **Growth engine.** Self-reflection, pattern observation, insight materialization.
- **Ingestion pipeline.** Markdown, JSON/JSONL, CSV, PDF, ZIP. Semantic chunking, format-aware parsing, Mem0/SuperMemory import support.
- **Resilience.** Circuit breakers, exponential backoff with jitter, dead-letter queue for failed operations.

### Session sidecar

`kleos-sidecar` sits between your agent and the server. Instead of blocking on every memory write, the agent emits observations to the sidecar, which buffers them in memory, optionally compresses via a local Ollama instance, and flushes in batches. File-watching and persistent session support included.

### Security model

- SQLCipher encryption at rest. Key from a file, an env var, or a hardware security key (YubiKey HMAC-SHA1 challenge-response).
- Bearer tokens hashed with SHA-256 and peppered.
- Request signing via ECDSA-P256 or Ed25519 with replay-resistant nonces.
- Separate credential daemon (`kleos-credd`) with AES-256-GCM vault and two-tier key model (master + agent keys).
- Zero-knowledge agent bootstrapping via ECDH key agreement with PIV.
- Pre-action gate system: allow, warn, or block agent commands, with a human approval queue for risky operations.
- Shell command gating with static validation, SSRF guard, and AST-aware filesystem operations.
- Session grants with configurable TTL for managed environments.
- Audit log on every mutation.

### Building from source

```bash
cargo build --release --workspace                                             # everything
cargo test --workspace --exclude kleos-migrate                                # test suite
cargo clippy --workspace --exclude kleos-migrate --all-targets -- -D warnings # lint gate
```

Requires Rust 1.94+, `protoc`, and `libpcsclite-dev` (Linux) for smartcard support.

</details>

<details>
<summary><strong>For researchers: the cognitive science</strong></summary>

### Spaced repetition (FSRS-6)

All 21 trained weights from the Free Spaced Repetition Scheduler, the same algorithm behind Anki's scheduling. Each memory carries a dual-strength tuple: storage strength accumulates with use, retrieval strength decays via power-law forgetting. Live retrievability modulates search scores in real time. Memories the agent uses stay sharp. Ones it ignores drift out of the way without being deleted.

### Hybrid retrieval

Four channels run per query:

1. **Vector similarity.** BAAI/bge-m3 ONNX embeddings (1024-dim), LanceDB ANN index.
2. **Full-text.** FTS5 BM25 scoring.
3. **Graph traversal.** 2-hop expansion weighted by typed edges and PageRank.
4. **Personality signals.** Preference, value, motivation, decision, emotion, identity.

Results merge through Reciprocal Rank Fusion. Question-type detection classifies the query (fact recall, preference, reasoning, generalization, timeline) and reweights channels before an IBM Granite cross-encoder reranks the top-K. Contradiction damping suppresses memories containing "no longer," "switched from," and similar markers.

### Knowledge graph

Auto-linking by cosine similarity. Six typed edges: similarity, updates, extends, contradicts, caused_by, prerequisite_for. Link-type weights so a "causes" edge outranks an "extends" edge. Louvain community detection capped at 10K nodes per pass. Weighted PageRank with deduplicated multi-edges. Sliding-window entity cooccurrence. Structural and impact analysis.

### Hopfield network (feature-gated)

A continuous Hopfield network (Ramsauer et al. 2020) with softmax attention and capacity exponential in dimension. Embedding compression via PCA using power iteration and deflation, no LAPACK dependency. A six-stage dream cycle runs during idle: replay, merge, prune, discover, decorrelate, resolve.

### Personality engine

Six signal types extracted from stored text. Each carries a subject, valence (positive/negative/neutral/mixed), intensity in [0,1], and the reasoning that produced it. The agent's recall is shaped by its own personality context. Two agents querying the same memories get different results.

### Growth and self-reflection

Agents generate observations about their own patterns by reflecting on recent activity. A materialization step converts raw observations into structured insight memories. The growth system avoids duplicating existing observations and integrates with the spaced-repetition loop so useful insights stay accessible.

### On-store processing

SimHash near-duplicate suppression. SVO contradiction detection against a structured-fact triple store. Atomic fact decomposition (LLM-first with rule-based fallbacks) that splits long memories into self-contained facts linked via `has_fact` edges.

### Skill evolution

Skills are versioned, tracked, and judged. When a skill fails, the LLM can analyze execution history and propose a fix. New skills can be derived from existing ones or captured from natural language descriptions. Trust scores aggregate execution success rates and human/agent judgments. A lineage graph tracks which skills evolved from which.

</details>

<details>
<summary><strong>For those evaluating the engineering</strong></summary>

### Scope

16 Rust crates. ~204K lines of code. ~6,000 test declarations across 113 test files. Single statically linked binary with the mimalloc allocator. No Python runtime. No external service dependencies at rest.

### Workspace

| Crate | What it does |
| --- | --- |
| `kleos-lib` | Core library: memory, search, embeddings, graph, intelligence, services, skills, growth, auth, gate, jobs. Feature-gated `brain` backend. |
| `kleos-server` | Axum HTTP server. 56 route modules, 8 middleware layers, embedded web GUI. |
| `kleos-cli` | Command-line client. Memory ops, skill management, handoffs, credential management. |
| `kleos-mcp` | MCP server. 59 tools across 8 domains. Stdio transport, HTTP behind a feature flag. |
| `kleos-sidecar` | Session-scoped memory proxy. File watcher, batched flushing, Ollama compression, persistent sessions. |
| `kleos-cred` | Credential library. YubiKey challenge-response, Argon2id KDF, ECDH agreement, CRED:v3 vault resolution. |
| `kleos-credd` | Credential daemon. Two-tier auth (master + agent keys), AES-256-GCM encryption, zero-knowledge agent bootstrap. |
| `kleos-ingest` | Transcript ingest daemon. PIV/software-key request signing, file watching, LLM summarization, real-time observation streaming. |
| `agent-forge` | Structured reasoning CLI. 20+ subcommands (spec-task, log-hypothesis, verify, challenge-code, session-diff, repo-map, search-code). Tree-sitter AST parsing for 7 languages. |
| `eidolon-supervisor` | Session drift detection daemon. Watches agent transcripts in real time, fires alerts on rule violations. |
| `kleos-sh` | Shell command gate. Static validation, SSRF guard, Claude Code hook integration, human approval queue. |
| `kleos-fs` | AST-aware filesystem operations. Guarded read/write with agent-forge integration. |
| `kleos-migrate` | One-shot ETL from encrypted monolith to per-tenant shards. |
| `kleos-cleanup` | Deduplication and log demotion utility. |
| `kleos-approval-tui` | Ratatui terminal UI for human-in-the-loop approval workflows. |
| `sdk` | Client SDKs: TypeScript, Python, Go. |

### Design decisions worth noting

- **No external services.** Embeddings run locally via ONNX. Vector storage is embedded LanceDB. Coordination is in-process. No message broker. No Python runtime.
- **Database-per-tenant isolation.** Not schema-per-tenant, not row-level. Each tenant gets its own SQLite file with its own encryption key and connection pool.
- **All external calls wrapped in ServiceGuard.** Three-state circuit breaker + exponential backoff + dead-letter queue. Failed work is inspectable and replayable.
- **Hardware-anchored trust.** The credential daemon and the server both support YubiKey-derived keys. Request signing supports PIV certificates. Agent bootstrapping uses ECDH key agreement so API keys never hit disk in plaintext.
- **Skills as a first-class system.** Versioned, trust-scored, LLM-evolvable, with execution analytics, judgment history, and lineage tracking. Cloud library for sharing.
- **Built-in supervision.** The Eidolon supervisor is not an afterthought. It watches agent sessions in real time and can intervene before damage is done.

### Install profiles

The `dist/install.sh` script supports three profiles:

- `server` includes kleos-server, kleos-cli, kleos-mcp
- `agent-host` includes kleos-cli, kleos-sh, kr, kw, ke, agent-forge, eidolon-supervisor, kleos-cred/credd
- `full` includes every binary

</details>

---

<div align="center">

[Wiki](https://github.com/Ghost-Frame/Kleos/wiki) · [Issues](https://github.com/Ghost-Frame/Kleos/issues)

Elastic License 2.0

</div>
