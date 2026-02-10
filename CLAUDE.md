# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

zkBring TLSNotary Infrastructure — a privacy-preserving credential verification system. Users prove aspects of their online identity (e.g., Twitter followers, Uber rides, Apple devices) without revealing sensitive data. Proofs are verified and signed with Ethereum-compatible keys for on-chain submission.

Three independent services:
- **verifier/** (Rust/Axum) — main service that verifies TLSNotary proofs and OAuth credentials, signs results for smart contracts
- **notary/** (Docker wrapper) — TLSNotary notarization server (upstream `ghcr.io/tlsnotary/tlsn/notary-server`)
- **proxy/** (Python/websockify) — WebSocket-to-TCP reverse proxy routing credential requests to target domains

## Build & Run Commands

### Verifier (Rust)
```bash
cd verifier
cargo build              # build
cargo run                # run (reads .env.dev in dev mode)
cargo check              # type-check without building
cargo clippy             # lint
cargo fmt                # format
```
The verifier loads `.env.dev` (dev) or `.env` (production) based on the `ENV` variable. It runs on the port specified by `PORT` (default 3000).

### Notary (Docker)
```bash
cd notary
make build               # build Docker image
make setup-dev           # generate dev secp256k1 key
make dev                 # run dev container on :7047
make run                 # run production container on :7047
make release             # push to ghcr.io/bringid/tlsn/notary:latest
```

### Proxy (Docker)
```bash
cd proxy
make build               # build Docker image
make dev                 # run dev container on :3003
make run                 # run production container on :3003
make release             # push to ghcr.io/bringid/tlsn/proxy:latest
```

## Verifier Architecture

### API Endpoints
| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/` | GET | Service info (version, verifier address) |
| `/verify` | POST | TLSNotary proof verification |
| `/verify/oauth` | POST | OAuth credential verification |

### Source Layout (`verifier/src/`)
- **main.rs** — entry point; registers custom handlers, loads verification configs, starts server
- **config.rs** — singleton config via `OnceLock`, loads env vars (notary key, private key, salt, etc.)
- **tlsn.rs** — TLSNotary proof deserialization and verification
- **signer.rs** — Ethereum-compatible ECDSA signing
- **core/** — domain models: `Verification`, `OAuthVerification`, check types (`LenGte`, `Gte`, `Lte`, `Eq`, `Contains`, `Custom`, `Any`), presentation windows
- **services/** — business logic:
  - `server/` — Axum router and HTTP handlers (`root`, `verify_tlsn`, `verify_oauth`)
  - `verification_manager/` — loads and caches TLSN verification configs from `verifications.json`
  - `handlers_manager/` — dynamic custom handler registry
- **custom_handlers/** — domain-specific logic (Apple device ID extraction, Uber ride counting)
- **helpers/** — user ID hashing (keccak256 with salt), response construction and signing, registry address parsing

### Verification Configs
- `verifier/verifications.json` — TLSN verification rules (host patterns, check types, user ID extraction)
- `verifier/oauth_verifications.json` — production OAuth provider/score mappings
- `verifier/oauth_verifications_staging.json` — staging OAuth configs (selected when `ENV=dev`)

### Verification Flow
1. Client submits proof (TLS presentation or OAuth signature) with credential group ID, semaphore commitment, and registry address
2. Verifier validates the proof (TLSN transcript verification or OAuth signature recovery)
3. Verification rules from JSON configs are applied (check types, custom handlers)
4. User ID is hashed with salt (keccak256) for privacy
5. Response is ABI-encoded and signed with verifier's private key for on-chain submission

### Custom Handlers
Pluggable functions registered at startup in `main.rs`. Each handler receives a `PresentationCheck` and transcript string, returns `(bool, Option<B256>)` (success flag + optional user_id_hash). Add new handlers by:
1. Creating a function in `custom_handlers/`
2. Exporting it in `custom_handlers.rs`
3. Registering it in `main.rs` via `HandlersManager::register()`
4. Referencing it by name in `verifications.json` with check type `custom`

### Environment Variables (verifier)
| Variable | Purpose |
|----------|---------|
| `ENV` | `dev` or production (affects OAuth config file and signer validation) |
| `PORT` | Server port (default 3000) |
| `NOTARY_KEY_ALG` | Key algorithm: `K256` or `P256` |
| `NOTARY_KEY_HEX` | Notary's public key (hex) |
| `PRIVATE_KEY_HEX` | Verifier's signing private key (hex) |
| `SALT_HEX` | Salt for user ID hashing |
| `OAUTH_SIGNER_ADDRESS` | Expected Ethereum address of OAuth signer |
| `RUST_LOG` | Log level for tracing |

### Key Dependencies
- `tlsn-core` (from git) — TLSNotary protocol
- `axum` + `tokio` — async HTTP server
- `alloy` — Ethereum signing, ABI encoding, RLP
- `tracing` — structured logging

### Dev vs Production Behavior
In dev mode (`ENV=dev`):
- Uses `oauth_verifications_staging.json` instead of `oauth_verifications.json`
- OAuth verification generates random `user_id_hash` instead of validating signer address
