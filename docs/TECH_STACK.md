# SPRX Protocol — Technology Decision Matrix & Stack

## Technology Stack Summary

### 1. Core Blockchain Layer (`crates/*`)
- **Language**: Rust (2021 Edition, MSRV 1.80.0)
- **Async Runtime**: Tokio
- **Storage**: `redb` (pure-Rust embedded ACID KV store)
- **Cryptography**: `ed25519-dalek`, `k256`, `blake3`, `sha2`, `bip39`
- **RPC & Networking**: `axum`, `tower-http`, `tokio::net::TcpListener`

### 2. Backend & Ingestion Layer (`backend/*`)
- **Language**: Python 3.11+
- **API Framework**: FastAPI, Pydantic v2, Uvicorn
- **Database**: PostgreSQL 16 with SQLAlchemy 2.0 (asyncpg) & Alembic
- **Caching & Message Broker**: Redis 7 & Celery
- **Authentication**: JWT, bcrypt, Ed25519 signature validation

### 3. Frontend & Presentation Layer (`apps/*`, `packages/*`)
- **SDK**: `@sprax/wallet-core` (TypeScript, WebCrypto, BIP-39)
- **Web Wallet**: React 18, Vite, TypeScript, Lucide Icons
- **Explorer UI**: React 18, Vite, Tailwind CSS
- **Admin Panel**: React 18, Vite, Tailwind CSS, Recharts
- **Mobile Wallet**: Flutter / Dart (multi-platform iOS & Android)

### 4. Infrastructure & DevOps (`deploy/*`, `.github/*`)
- **Containers**: Docker, Docker Compose
- **Monitoring**: Prometheus (`:26660`), Grafana
- **CI/CD**: GitHub Actions (multi-stack test matrix for Rust, Python, TypeScript, Docker)
