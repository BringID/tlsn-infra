# zkBring TLSNotary Verifier — API Documentation

Base URL: `http://localhost:3000`

The server is running in **dev mode** (`ENV=dev`), which means:
- OAuth signer address validation is skipped (random `credential_id` is generated)
- Staging OAuth verification configs are used

## Endpoints

### GET /

Health check / service info.

**Response:**
```json
{
  "info": "zkBring TLSNotary Verifier",
  "version": "0.1.0",
  "verifier_address": "0x3c50f7055D804b51e506Bc1EA7D082cB1548376C"
}
```

---

### POST /verify/oauth

Verify an OAuth credential and return a signed attestation.

**Request body:**
```json
{
  "message": {
    "domain": "string",
    "userId": "string",
    "score": "string (uint256)",
    "timestamp": "string (uint256)"
  },
  "signature": "string (hex-encoded ECDSA signature over keccak256 of ABI-encoded message)",
  "registry": "string (Ethereum address, e.g. 0xbF9b2556e6Dd64D60E08E3669CeF2a4293e006db)",
  "chain_id": "number (8453 for Base Mainnet, 84532 for Base Sepolia)",
  "credential_group_id": "string (see available groups below)",
  "app_id": "string (uint256, the app ID on the CredentialRegistry contract)",
  "semaphore_identity_commitment": "string (uint256)"
}
```

**Field details:**

| Field | Type | Description |
|-------|------|-------------|
| `message.domain` | string | OAuth provider domain. Must match the credential group's configured domain |
| `message.userId` | string | User identifier from the OAuth provider |
| `message.score` | string | Score value (uint256 as string). Must be >= the credential group's required score |
| `message.timestamp` | string | Timestamp (uint256 as string) |
| `signature` | string | ECDSA signature over `keccak256(abi.encode(domain, userId, score, timestamp))`. In dev mode, signer validation is skipped |
| `registry` | string | CredentialRegistry contract address |
| `chain_id` | number | Target chain ID. Must be `8453` (Base Mainnet) or `84532` (Base Sepolia) |
| `credential_group_id` | string | ID of the credential group to verify against |
| `app_id` | string | Application ID on the CredentialRegistry contract (uint256 as string) |
| `semaphore_identity_commitment` | string | Semaphore identity commitment (uint256 as string) |

**Successful response (200):**
```json
{
  "attestation": {
    "registry": "0xbf9b2556e6dd64d60e08e3669cef2a4293e006db",
    "chain_id": 84532,
    "credential_group_id": "1",
    "credential_id": "0x...",
    "app_id": "1",
    "semaphore_identity_commitment": "12345",
    "issued_at": 1740268800
  },
  "verifier_hash": "0x...",
  "signature": "0x..."
}
```

**Response field details:**

| Field | Type | Description |
|-------|------|-------------|
| `attestation.registry` | address | CredentialRegistry contract address (checksummed) |
| `attestation.chain_id` | number | Chain ID |
| `attestation.credential_group_id` | string | Credential group ID |
| `attestation.credential_id` | bytes32 | Hashed credential identifier (deterministic in prod, random in dev) |
| `attestation.app_id` | string | Application ID |
| `attestation.semaphore_identity_commitment` | string | Semaphore identity commitment |
| `attestation.issued_at` | number | Unix timestamp (seconds) |
| `verifier_hash` | string | `keccak256(abi.encode(registry, chainId, credentialGroupId, credentialId, appId, semaphoreIdentityCommitment, issuedAt))` — the hash that was signed |
| `signature` | string | ECDSA signature over the verifier_hash (as EthSignedMessage) |

**Error responses:** See [Error Codes](#error-codes) below.

---

### POST /verify

Verify a TLSNotary proof and return a signed attestation.

**Request body:**
```json
{
  "tlsn_presentation": "string (hex-encoded bincode-serialized TLSNotary Presentation)",
  "registry": "string (Ethereum address)",
  "chain_id": "number (8453 or 84532)",
  "credential_group_id": "string",
  "app_id": "string (uint256)",
  "semaphore_identity_commitment": "string (uint256)"
}
```

**Response:** Same format as `/verify/oauth`.

**Error responses:** See [Error Codes](#error-codes) below.

---

## Error Codes

All error responses return JSON in this format:

```json
{
  "success": false,
  "errors": ["ERROR_CODE"],
  "message": "human-readable description"
}
```

- `success` — always `false` for errors
- `errors` — array of error code strings
- `message` — human-readable description of what went wrong

### Shared errors (both endpoints)

These errors can occur on both `/verify` and `/verify/oauth` since they share the response-building step.

| HTTP | Error Code | Cause |
|------|------------|-------|
| 400 | `INVALID_REGISTRY_ADDRESS` | `registry` is not a valid Ethereum address |
| 400 | `REGISTRY_NOT_WHITELISTED` | `registry` is not in the allowed whitelist |
| 400 | `UNSUPPORTED_CHAIN_ID` | `chain_id` is not `8453` (Base Mainnet) or `84532` (Base Sepolia) |
| 400 | `INVALID_APP_ID` | `app_id` is not a valid uint256 |
| 400 | `INVALID_CREDENTIAL_GROUP_ID` | `credential_group_id` is not a valid uint256 |
| 400 | `INVALID_SEMAPHORE_COMMITMENT` | `semaphore_identity_commitment` is not a valid uint256 |
| 500 | `SIGNING_FAILED` | Internal error during ECDSA message signing |

### POST /verify errors

| HTTP | Error Code | Cause |
|------|------------|-------|
| 400 | `PRESENTATION_DECODE_FAILED` | `tlsn_presentation` is not valid hex |
| 400 | `PRESENTATION_DESERIALIZE_FAILED` | Decoded bytes are not a valid bincode-serialized TLSNotary Presentation |
| 400 | `PROOF_VERIFICATION_FAILED` | TLSNotary proof verification failed (invalid notary key, wrong server name, check failure, etc.) |

### POST /verify/oauth errors

| HTTP | Error Code | Cause |
|------|------------|-------|
| 400 | `SIGNATURE_PARSE_FAILED` | `signature` is not a valid ECDSA signature |
| 400 | `ADDRESS_RECOVERY_FAILED` | Could not recover signer address from the signature |
| 401 | `WRONG_OAUTH_SIGNER` | Recovered signer does not match the trusted OAuth signer (production only; skipped in dev mode) |
| 400 | `CREDENTIAL_ID_FAILED` | Failed to compute credential ID from user ID |
| 500 | `VERIFICATION_NOT_FOUND` | No verification config found for the given `credential_group_id` |
| 400 | `VERIFICATION_CHECK_FAILED` | Domain or score does not meet the credential group requirements |

### Note on missing fields

If a required field is missing from the request body, Axum returns a `422 Unprocessable Entity` with a framework-generated message before any application error code is reached.

---

## Available OAuth Credential Groups (staging)

| ID | Domain | Min Score |
|----|--------|-----------|
| 1 | farcaster.xyz | 10 |
| 2 | farcaster.xyz | 30 |
| 3 | farcaster.xyz | 70 |
| 4 | github.com | 10 |
| 5 | github.com | 30 |
| 6 | github.com | 70 |
| 7 | x.com | 10 |
| 8 | x.com | 30 |
| 9 | x.com | 70 |
| 10 | zkpassport.id | 100 |
| 11 | self.xyz | 100 |

## Attestation ABI Encoding

The attestation is ABI-encoded as a Solidity struct:

```solidity
struct Attestation {
    address registry;
    uint256 chainId;
    uint256 credentialGroupId;
    bytes32 credentialId;
    uint256 appId;
    uint256 semaphoreIdentityCommitment;
    uint256 issuedAt;
}
```

The `verifier_hash` is `keccak256(abi.encode(attestation))` and the `signature` is an ECDSA signature over `toEthSignedMessageHash(verifier_hash)`.

## Notes for Testing

- In dev mode, the `signature` field in the OAuth request is still required and must be a valid ECDSA signature format, but the recovered signer address is **not** validated against the trusted signer.
- The `message` fields (`domain`, `userId`, `score`, `timestamp`) in the OAuth request use Solidity ABI encoding internally. The `signature` should be over `keccak256(abi.encode(domain, userId, score, timestamp))` using `abi.encode` for the packed Solidity types `(string, string, uint256, uint256)`.
- All uint256 values in the request are passed as **strings**.
- The `chain_id` must be `8453` or `84532` — any other value returns 400.
