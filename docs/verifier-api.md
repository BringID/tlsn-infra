# Verifier API Reference

Reference for consuming repos (widget, browser extension) integrating with the zkBring TLSNotary Verifier.

## Service URLs

| Environment | URL |
|-------------|-----|
| Via Zuplo gateway | `https://api.bringid.org/v1/verifier` |
| Production | TBD |

When calling through Zuplo, add `Authorization: Bearer {ZUPLO_API_KEY}` and append `?environment={mode}` (`dev` or `production`).

## Breaking Changes from v1

Both `/verify` and `/verify/oauth` requests now require a new **`app_id`** field. The response format has changed:

| v1 | v2 | Notes |
|----|-----|-------|
| `verifier_message` | `attestation` | Response wrapper renamed |
| `verifier_message.id_hash` | `attestation.credential_id` | Field renamed |
| — | `attestation.app_id` | **New field** in response |
| — | `attestation.issued_at` | **New field** — unix timestamp, contract enforces 30 min validity |

### Widget changes required

**`src/app/content/api/verifier/index.tsx`** — add `app_id` to both request bodies:

```diff
 // POST /verify (ZK-TLS)
 {
   tlsn_presentation: string,
   registry: string,
   credential_group_id: string,
+  app_id: string,
   semaphore_identity_commitment: string
 }

 // POST /verify/oauth
 {
   message: { domain, userId, score, timestamp },
   signature: string,
   registry: string,
   credential_group_id: string,
+  app_id: string,
   semaphore_identity_commitment: string
 }
```

**`src/app/content/api/task-manager/index.tsx`** — update task creation body:

```diff
 {
   registry: string,
   credential_group_id: string,
-  id_hash: string,
+  credential_id: string,
+  app_id: string,
   identity_commitment: string,
   verifier_signature: string
 }
```

**Response parsing** — update references from `verifier_message` to `attestation` and from `id_hash` to `credential_id`:

```diff
- const idHash = response.verifier_message.id_hash
+ const credentialId = response.attestation.credential_id

- const signature = response.signature
+ const signature = response.signature  // unchanged
```

### Extension changes required

**`VERIFICATION_DATA_READY` payload** — add `tlsn_presentation` field to the exported data (see [Extension Download Format](./uber_extension_prompt.md)):

```diff
 {
   transcriptRecv: string,
-  presentationData: string,
+  presentationData: string,
+  tlsn_presentation: string,  // hex(bincode(Presentation)) — sent to verifier
 }
```

### Credential group ID renumbering

| v1 ID | v2 ID | Credential |
|-------|-------|------------|
| 8-10 | 4-6 | GitHub (Low/Med/High) |
| 11-13 | 7-9 | X Twitter (Low/Med/High) |
| 14-16 | 1-3 | Farcaster (Low/Med/High) |
| 17 | 10 | zkPassport |
| 1 | 12 | Uber Rides |

Update all `credentialGroupId` references in task configs, verification flows, and any hardcoded IDs.

## Endpoints

### `GET /`

Returns service info and the verifier's Ethereum address.

```json
{
  "info": "zkBring TLSNotary Verifier",
  "version": "0.1.0",
  "verifier_address": "0x3c50f7055D804b51e506Bc1EA7D082cB1548376C"
}
```

### `POST /verify` — TLSNotary Verification

Verifies a TLSNotary presentation and returns a signed attestation for on-chain submission.

**Request:**

```json
{
  "tlsn_presentation": "<hex-encoded bincode-serialized Presentation>",
  "registry": "0x78Ce003ff79557A44eae862377a00F66df0557B2",
  "credential_group_id": "12",
  "app_id": "1",
  "semaphore_identity_commitment": "18829151068536933270973543420190644881638744892988094175595226199525274517143"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `tlsn_presentation` | string | Hex-encoded bincode serialization of `tlsn_core::presentation::Presentation`. No `0x` prefix. |
| `registry` | string | CredentialRegistry contract address |
| `credential_group_id` | string | Credential group ID (see tables below) |
| `app_id` | string | Application ID (uint256 as decimal string). Obtain via `registerApp()` on CredentialRegistry. |
| `semaphore_identity_commitment` | string | User's Semaphore identity commitment (uint256 as decimal string) |

**Response (200):**

```json
{
  "attestation": {
    "registry": "0x78ce003ff79557a44eae862377a00f66df0557b2",
    "credential_group_id": "12",
    "credential_id": "0xd569d4088919af7084a88000141f6d6ef9a7a3e348d6c05f7276b11d74a922b3",
    "app_id": "1",
    "semaphore_identity_commitment": "18829151068536933270973543420190644881638744892988094175595226199525274517143",
    "issued_at": "1770739162"
  },
  "verifier_hash": "0x95f9d7826276269517b01141b800e9df3aa4f57cdcd4af9d242172e8fdab324b",
  "signature": "0x182a92c9065524bf7fcd5d1797500bd9dd222848f8ce967fa9ab26ba6d6730531b639094feb8a99127752c2072f19c9cabb6d457a8218032616ad83bb36648b81b"
}
```

| Field | Description |
|-------|-------------|
| `attestation` | Matches the on-chain `Attestation` Solidity struct |
| `attestation.credential_id` | `keccak256(user_id + app_id + private_key)` — deterministic per user/app pair |
| `attestation.issued_at` | Unix timestamp (seconds). Contract enforces `block.timestamp <= issuedAt + attestationValidityDuration` (default 30 min) |
| `verifier_hash` | `keccak256(abi.encode(attestation))` — the signed message |
| `signature` | ECDSA signature over `verifier_hash`, recoverable to `verifier_address` |

**Error Response (400/500):** Plain text error message.

### `POST /verify/oauth` — OAuth Verification

Verifies an OAuth credential signature and returns a signed attestation.

**Request:**

```json
{
  "message": "{\"domain\":\"github.com\",\"user_id\":\"12345\",\"timestamp\":1770000000}",
  "signature": "0x...",
  "semaphore_identity_commitment": "18829151068536933270973543420190644881638744892988094175595226199525274517143",
  "credential_group_id": "4",
  "app_id": "1",
  "registry": "0x78Ce003ff79557A44eae862377a00F66df0557B2"
}
```

**Response:** Same format as `/verify`.

## Credential Groups

### TLSN (POST /verify)

| ID | Credential | Host | Validation |
|----|------------|------|------------|
| 12 | Uber Rides | `riders.uber.com` | >= 5 non-canceled rides |

### OAuth (POST /verify/oauth)

| ID | Credential | Domain | Score |
|----|------------|--------|-------|
| 1 | Farcaster Low | `farcaster.xyz` | 10 |
| 2 | Farcaster Medium | `farcaster.xyz` | 30 |
| 3 | Farcaster High | `farcaster.xyz` | 70 |
| 4 | GitHub Low | `github.com` | 10 |
| 5 | GitHub Medium | `github.com` | 30 |
| 6 | GitHub High | `github.com` | 70 |
| 7 | X (Twitter) Low | `x.com` | 10 |
| 8 | X (Twitter) Medium | `x.com` | 30 |
| 9 | X (Twitter) High | `x.com` | 70 |
| 10 | zkPassport | `zkpassport.id` | 100 |

## On-Chain Integration

The attestation + signature are passed to `CredentialRegistry.registerCredential(attestation, signature)`.

**Solidity struct:**

```solidity
struct Attestation {
    address registry;
    uint256 credentialGroupId;
    bytes32 credentialId;
    uint256 appId;
    uint256 semaphoreIdentityCommitment;
    uint256 issuedAt;
}
```

**Contract addresses (Base Sepolia):**

| Contract | Address |
|----------|---------|
| CredentialRegistry | `0x78Ce003ff79557A44eae862377a00F66df0557B2` |
| Semaphore | `0x8A1fd199516489B0Fb7153EB5f075cDAC83c693D` |
| DefaultScorer | `0x68a3CA701c6f7737395561E000B5cCF4ECa5185A` |

**Trusted verifier:** `0x3c50f7055D804b51e506Bc1EA7D082cB1548376C`

## Presentation File Format

The browser extension exports a JSON file with transcript + raw presentation:

```json
{
  "transcriptRecv": "...",
  "transcriptSent": "...",
  "tlsn_presentation": "014000000000000000..."
}
```

| Field | Description |
|-------|-------------|
| `transcriptRecv` | Decoded server response with redacted (`*`) and revealed sections |
| `transcriptSent` | Decoded client request with redacted sections |
| `tlsn_presentation` | `hex::encode(bincode::serialize(&presentation))` — sent to verifier API |

The `tlsn_presentation` is hex-encoded bincode of `tlsn_core::presentation::Presentation`. No `0x` prefix, lowercase hex, no whitespace.

## Dev Mode Behavior

When the verifier runs with `ENV=dev`:
- `credential_id` is **randomized** on every request (not deterministic)
- OAuth verification skips signer address validation
- Uses `oauth_verifications_staging.json` for OAuth credential group configs
