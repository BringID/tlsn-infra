# Extension Download Format: Add Presentation to JSON

## Current Format

The extension downloads a JSON file with the decoded transcript:

```json
{
  "transcriptRecv": "...",
  "transcriptSent": "..."
}
```

## Required Format

Add a `tlsn_presentation` field containing the hex-encoded raw TLSNotary presentation:

```json
{
  "transcriptRecv": "...",
  "transcriptSent": "...",
  "tlsn_presentation": "014000000000000000..."
}
```

The `transcriptRecv` and `transcriptSent` fields stay as-is. The new `tlsn_presentation` field is the only addition.

## How to Produce the `tlsn_presentation` Field

The extension already has the `Presentation` object in memory (from `tlsn-core`). Serialize it and hex-encode:

```
Presentation → bincode::serialize → Vec<u8> → hex::encode → String
```

### Rust / WASM

```rust
let presentation: Presentation = /* already available after notarization */;
let bytes = bincode::serialize(&presentation).unwrap();
let hex_string = hex::encode(bytes);
// include as "tlsn_presentation" field in the JSON
```

### JavaScript (if presentation is a Uint8Array of bincode bytes)

```javascript
const hexString = Array.from(presentationBytes)
  .map(b => b.toString(16).padStart(2, '0'))
  .join('');
// include as "tlsn_presentation" field in the JSON
```

## `tlsn_presentation` Field Requirements

| Property | Value |
|----------|-------|
| Content | `bincode::serialize(&presentation)` then `hex::encode` |
| Characters | Lowercase hex (`0-9a-f`) |
| Prefix | No `0x` prefix |
| Whitespace | None — continuous string |

## Why This Is Needed

The verifier API (`POST /verify`) requires the raw presentation to perform cryptographic verification:

```rust
let bytes = hex::decode(tlsn_presentation)?;
let presentation: Presentation = bincode::deserialize(&bytes)?;
presentation.verify(&CryptoProvider::default())?;
```

The transcript fields alone cannot be used — they lack the notary signatures, commitments, and selective disclosure proofs that the `Presentation` object contains.
