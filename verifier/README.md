# BringID Verifier

Privacy-preserving credential verification service. Verifies TLSNotary proofs and OAuth credentials, then signs attestations with an Ethereum-compatible key for on-chain submission to the CredentialRegistry contract.

## Quick Start

```bash
cargo build
cargo run          # reads .env.dev in dev mode
```

The server starts on the port specified by `PORT` (default 3000).

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `ENV` | No | Set to `dev` for dev mode (relaxed OAuth validation, staging configs) |
| `PORT` | Yes | Server port |
| `NOTARY_KEY_ALG` | Yes | Notary key algorithm: `K256` or `P256` |
| `NOTARY_KEY_HEX` | Yes | Notary's public key (hex-encoded) |
| `PRIVATE_KEY_HEX` | Yes | Verifier's ECDSA signing private key (hex-encoded) |
| `SALT_HEX` | Yes | Salt for user ID hashing (hex-encoded) |
| `OAUTH_SIGNER_ADDRESS` | Yes | Expected Ethereum address of the OAuth signer |
| `RUST_LOG` | No | Log level for tracing (e.g. `info`, `debug`) |

In dev mode, the service loads `.env.dev` automatically.

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Service info (version, verifier address) |
| `/verify` | POST | TLSNotary proof verification |
| `/verify/oauth` | POST | OAuth credential verification |

See [docs/verifier-api.md](../docs/verifier-api.md) for full API documentation, request/response examples, and credential group listings.

## Error Response Format

All errors return structured JSON:

```json
{
  "success": false,
  "errors": ["ERROR_CODE"],
  "message": "human-readable description"
}
```

| HTTP | Error Code | Endpoint | Cause |
|------|------------|----------|-------|
| 400 | `PRESENTATION_DECODE_FAILED` | `/verify` | `tlsn_presentation` is not valid hex |
| 400 | `PRESENTATION_DESERIALIZE_FAILED` | `/verify` | Invalid bincode-serialized TLSNotary Presentation |
| 400 | `PROOF_VERIFICATION_FAILED` | `/verify` | TLSNotary proof verification failed |
| 400 | `SIGNATURE_PARSE_FAILED` | `/verify/oauth` | `signature` is not a valid ECDSA signature |
| 400 | `ADDRESS_RECOVERY_FAILED` | `/verify/oauth` | Could not recover signer address |
| 401 | `WRONG_OAUTH_SIGNER` | `/verify/oauth` | Recovered signer doesn't match trusted signer (production only) |
| 400 | `CREDENTIAL_ID_FAILED` | `/verify/oauth` | Failed to compute credential ID |
| 500 | `VERIFICATION_NOT_FOUND` | `/verify/oauth` | No config for the given `credential_group_id` |
| 400 | `VERIFICATION_CHECK_FAILED` | `/verify/oauth` | Domain or score doesn't meet requirements |
| 400 | `INVALID_REGISTRY_ADDRESS` | Both | `registry` is not a valid Ethereum address |
| 400 | `INVALID_CHAIN_ID` | Both | `chain_id` is not a valid integer |
| 400 | `UNSUPPORTED_CHAIN_ID` | Both | `chain_id` is not `8453` or `84532` |
| 400 | `INVALID_APP_ID` | Both | `app_id` is not a valid uint256 |
| 400 | `INVALID_CREDENTIAL_GROUP_ID` | Both | `credential_group_id` is not a valid uint256 |
| 400 | `INVALID_SEMAPHORE_COMMITMENT` | Both | `semaphore_identity_commitment` is not a valid uint256 |
| 500 | `SIGNING_FAILED` | Both | Internal ECDSA signing error |

## Project Structure

```
verifier/
  src/
    main.rs              # Entry point, handler registration, server startup
    config.rs            # Singleton config loaded from env vars
    tlsn.rs              # TLSNotary proof deserialization and verification
    signer.rs            # Ethereum-compatible ECDSA signing
    core/                # Domain models (Verification, check types)
    custom_handlers/     # Pluggable verification logic (Apple, Uber)
    helpers/             # Response construction, user ID hashing, error types
    services/            # Axum router, HTTP handlers, verification config managers
  verifications.json           # TLSN verification rules
  oauth_verifications.json     # Production OAuth provider configs
  oauth_verifications_staging.json  # Staging OAuth configs
```

## Dev vs Production

| Behavior | Dev (`ENV=dev`) | Production |
|----------|-----------------|------------|
| OAuth signer validation | Skipped | Enforced |
| `credential_id` | Random per request | Deterministic (hashed) |
| OAuth config file | `oauth_verifications_staging.json` | `oauth_verifications.json` |
