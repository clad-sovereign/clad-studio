# signer-core Test Corpora

This directory contains **golden test vectors** for the Rust port of two
protocol layers:

- **Phase 1 (UOS):** Universal Offline Signatures encode/decode.  Vectors
  generated from the Kotlin oracle.
- **Phase 2 (crypto + extrinsic):** SR25519/ED25519 signing, Blake2b hashing,
  SS58 encoding, and SCALE-encoded Substrate call data.  Vectors sourced from
  public Substrate test vectors (Alice key, SS58 prefix 42) or computed from
  audited constants; Kotlin oracle extraction deferred to Phase 2b.

Each JSON file encodes a fixed input and expected output.  The corpus tests load
these files at runtime and assert the Rust implementation produces identical output.

---

## Corpus provenance

The `(input, expected_bytes)` values were **generated from the Kotlin reference
implementation** using `UosCorpusExport.kt` (see
`clad-mobile/shared/src/jvmTest/kotlin/tech/wideas/clad/uos/UosCorpusExport.kt`).
The runner calls each existing Kotlin UOS class with a fixed seed input set and
writes JSON into this directory.  The Kotlin code is the **oracle**; these
checked-in JSON files are the **frozen snapshot** of that oracle's output.

### Regeneration procedure

```bash
cd clad-mobile
./gradlew :shared:jvmTest --tests "tech.wideas.clad.uos.UosCorpusExport"
```

This overwrites the JSON files in this directory.  Re-run the Rust tests
immediately after:

```bash
cd ../clad-studio
cargo test -p signer-core --locked
```

Any byte-level difference between the new Kotlin output and the Rust
implementation is a parity regression and must be resolved before merging.

**CI does not regenerate.**  Corpus regeneration is a manual gate.  Automating
it would defeat the oracle purpose — the test would always pass because it
compares the Rust output against itself.

---

## Directory layout

```
corpora/
├── payload/              # UosPayload encode/decode tests  (Phase 1 / Kotlin oracle)
│   ├── sign_tx_empty.json
│   ├── sign_tx_1byte.json
│   ├── sign_tx_1024bytes.json
│   ├── sign_tx_5000bytes.json
│   ├── sign_immortal_*.json
│   ├── sign_hash_*.json
│   └── sign_message_*.json
├── signature/            # UosSignature encode/decode tests  (Phase 1 / Kotlin oracle)
│   ├── sr25519_zeros.json
│   ├── sr25519_nonzero.json
│   ├── ed25519_*.json
│   └── ecdsa_*.json
├── multipart/            # MultiPartQr{Encoder,Decoder} tests  (Phase 1 / Kotlin oracle)
│   ├── single_frame.json
│   ├── three_frame_balanced.json
│   ├── three_frame_uneven_tail.json
│   └── large_5kb.json
├── account_introduction/ # AccountIntroduction URI tests  (Phase 1 / Kotlin oracle)
│   ├── minimal.json
│   ├── with_genesis.json
│   ├── with_name_ascii.json
│   ├── with_name_unicode.json
│   └── with_name_reserved_chars.json
├── crypto/               # Crypto KAT vectors  (Phase 2; Kotlin oracle pending Phase 2b)
│   ├── ss58_encode.json  # Alice pubkey + prefix 42 → SS58 address
│   ├── ss58_decode.json  # Alice SS58 address → pubkey + prefix
│   └── blake2b_256.json  # Blake2b-256 known-answer vectors
└── extrinsic/            # SCALE call data vectors  (Phase 2; Kotlin oracle pending Phase 2b)
    └── call_data.json    # CladToken calls (mint, transfer, freeze, …) for Alice AccountId
```

---

## JSON schema

### Payload

```json
{
  "description": "human-readable description",
  "input": {
    "crypto_type": 1,
    "action": 0,
    "account_id_hex": "<64 hex chars — 32 bytes>",
    "inner_payload_hex": "<0 or more hex bytes>"
  },
  "expected_bytes_hex": "<hex encoding of the UOS binary>"
}
```

Binary layout produced by `UosPayload::encode()`:

```
Byte 0:    0x53 (Substrate ID)
Byte 1:    crypto_type
Byte 2:    action
Bytes 3–34: account_id (32 bytes)
Bytes 35+:  inner payload (variable)
```

### Signature

```json
{
  "description": "...",
  "input": {
    "crypto_type": 1,
    "signature_hex": "<128 or 130 hex chars>"
  },
  "expected_bytes_hex": "<hex encoding of the UOS signature binary>"
}
```

Binary layout:
```
Byte 0:   crypto_type
Bytes 1+: signature bytes (64 for Sr25519/Ed25519, 65 for ECDSA)
```

### Multipart

```json
{
  "description": "...",
  "payload_hex": "<hex encoding of the original payload>",
  "frames_hex": ["<frame 0 hex>", "<frame 1 hex>", ...]
}
```

Multi-frame header format (big-endian, per frame):
```
Bytes 0–1: frame index (u16 BE)
Bytes 2–3: total frame count (u16 BE)
Bytes 4+:  frame data (≤ 1020 bytes)
```

Single-frame payloads are returned as-is (no header).

### Account Introduction

```json
{
  "description": "...",
  "input": {
    "address": "<SS58 address>",
    "genesis_hash": "<hex string or null>",
    "name": "<string or null>"
  },
  "expected_uri": "substrate:<address>[:<genesis_hash>][?name=<encoded>]"
}
```

The `name` field is URL-encoded per `application/x-www-form-urlencoded` (Java
`URLEncoder.encode(s, "UTF-8")` rules):
- `A-Z a-z 0-9 . - _ *` — unchanged
- ` ` (space) → `+`
- Everything else → `%XX` (two uppercase hex digits)

This matches the Kotlin reference implementation's encoding exactly.

### Phase 2 — crypto/ss58_encode.json

```json
{
  "description": "...",
  "input": { "public_key_hex": "<64 hex chars>", "prefix": 42 },
  "expected_address": "<SS58 string>"
}
```

### Phase 2 — crypto/ss58_decode.json

```json
{
  "description": "...",
  "input": { "address": "<SS58 string>" },
  "expected_public_key_hex": "<64 hex chars>",
  "expected_prefix": 42
}
```

### Phase 2 — crypto/blake2b_256.json

```json
{
  "description": "...",
  "vectors": [
    { "description": "...", "input_hex": "<hex>", "expected_hash_hex": "<64 hex chars>" }
  ]
}
```

### Phase 2 — extrinsic/call_data.json

```json
{
  "description": "...",
  "alice_account_hex": "<64 hex chars>",
  "vectors": [
    {
      "call": "mint",
      "args": { "account_hex": "<64 hex>", "amount": 1 },
      "expected_bytes_hex": "<hex of [pallet_u8][call_u8][0x00][32-byte AccountId][compact amount]>"
    }
  ]
}
```

Wire format: `[pallet_index: u8][call_index: u8][0x00 MultiAddress::Id][32-byte AccountId][optional SCALE Compact<u128> amount]`

---

## Notes

- Account-introduction URI encoding parity: Java `URLEncoder` differs subtly
  from RFC 3986 (`*` and `~` unencoded, spaces as `+`).  The Rust helper
  in `uos/account_introduction.rs` mirrors Java's rules exactly.  See Phase 1
  open position 3 in the restructure plan for a discussion of the implications.
- The `large_5kb` multipart corpus file contains a 5 000-byte all-zero payload
  split across 5 frames and is the largest single test vector.  CI loads it at
  test runtime; there is no inlining via `include_bytes!`.
