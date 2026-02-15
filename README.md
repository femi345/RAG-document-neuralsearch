# Cortex

A personal search engine that indexes your documents (PDFs, Notion, Slack, Gmail) and provides semantic search + conversational Q&A with citations.

## Architecture

```
┌─────────────┐     ┌─────────────────┐     ┌──────────────┐
│   Next.js    │────▶│   Rust API      │────▶│  Weaviate    │
│   Frontend   │     │   (axum)        │     │  (hybrid     │
└─────────────┘     │                 │     │   search)    │
                    │  8 crates:      │     └──────────────┘
                    │  api, ingestion,│
                    │  connectors,    │     ┌──────────────┐
                    │  chunker,       │────▶│  PostgreSQL   │
                    │  scheduler,     │     │  (metadata)   │
                    │  store,         │     └──────────────┘
                    │  ml-client,     │
                    │  common         │     ┌──────────────┐
                    └────────┬────────┘     │  Python ML   │
                             │ gRPC         │  Service     │
                             └─────────────▶│  (embed,     │
                                            │   rerank,    │
                                            │   generate)  │
                                            └──────────────┘
```

**Rust** handles the hot path: HTTP API, document ingestion, chunking, and job scheduling.
**Python** handles ML: embeddings (sentence-transformers), reranking (cross-encoder), and LLM orchestration.
**gRPC** connects them with a strongly-typed protobuf contract and streaming support.

## Key Features

- **Hybrid search** — BM25 keyword + vector similarity via Weaviate, with configurable alpha weighting
- **RAG chat** — retrieve relevant chunks, rerank with cross-encoder, stream LLM response with inline citations via SSE
- **Multi-provider LLM** — Claude, GPT-4, and Ollama (local) with a unified abstraction layer
- **Semantic chunking** — source-type aware strategy selection with overlap for context continuity
- **Incremental sync** — content-hash deduplication, only re-index changed documents
- **Async pipeline** — concurrent worker pool for non-blocking document ingestion

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Core engine | Rust (axum, tonic, sqlx, tokio) |
| ML service | Python (sentence-transformers, grpcio) |
| Vector DB | Weaviate (HNSW, hybrid search) |
| Relational DB | PostgreSQL 16 |
| IPC | gRPC with Protocol Buffers |
| Frontend | Next.js (planned) |

## Project Structure

```
cortex/
├── proto/                    # Shared gRPC contract
│   └── ml_service.proto
├── rust/                     # Rust workspace (8 crates)
│   └── crates/
│       ├── cortex-api/       # HTTP server + routes
│       ├── cortex-ingestion/ # Parse → chunk → embed → store
│       ├── cortex-connectors/# Data source abstraction
│       ├── cortex-chunker/   # Text splitting strategies
│       ├── cortex-scheduler/ # Async job worker pool
│       ├── cortex-store/     # Postgres + Weaviate clients
│       ├── cortex-ml-client/ # gRPC client to ML service
│       └── cortex-common/    # Shared types and config
├── ml/                       # Python ML service
│   └── cortex_ml/
│       ├── embeddings/       # Sentence-transformers
│       ├── reranker/         # Cross-encoder
│       ├── llm/              # Claude, GPT-4, Ollama
│       └── server.py         # gRPC server
└── docker-compose.yml
```

## API

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/v1/search` | Hybrid search with reranking |
| `POST` | `/api/v1/chat` | RAG chat with SSE streaming |
| `POST` | `/api/v1/ingest/upload` | Async document ingestion |
| `GET` | `/api/v1/ingest/jobs/:id` | Job status polling |
| `GET` | `/api/v1/health` | Service health checks |

## Getting Started

```bash
# Clone and configure
git clone https://github.com/femi345/cortex.git
cd cortex
cp .env.example .env
# Add your API keys to .env

# Start the full stack
docker compose up --build

# Or start infrastructure only and run services locally
make infra
make run-api   # In one terminal
make run-ml    # In another terminal
```

## Development

```bash
make build-rust    # Build Rust workspace
make test          # Run all tests
make fmt           # Format code
make lint          # Lint code
make proto         # Regenerate Python proto stubs
```

## Data Flow

**Ingestion:**
```
Document → Connector → Content Hash Dedup → Parser → Chunker → ML Embeddings → Weaviate + Postgres
```

**Search:**
```
Query → Embed → Weaviate Hybrid Search (BM25 + Vector) → Cross-Encoder Rerank → Results
```

**Chat (RAG):**
```
Query → Search → Rerank → Build Context → Stream LLM Response → Citations
```

## Roadmap

- [x] Core ingestion pipeline (parse → chunk → embed → store)
- [x] Hybrid search with reranking
- [x] RAG chat with SSE streaming and citations
- [x] Multi-provider LLM abstraction (Claude, GPT-4, Ollama)
- [x] Async job scheduling with worker pool
- [x] Docker Compose deployment
- [ ] Semantic chunking (embedding-similarity breakpoints)
- [ ] Notion, Slack, Gmail OAuth connectors
- [ ] Incremental sync with change detection
- [ ] Next.js frontend (search + chat + settings)
- [ ] Rate limiting and authentication middleware
- [ ] OpenAPI documentation generation

## License

MIT
