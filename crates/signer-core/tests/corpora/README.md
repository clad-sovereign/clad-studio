# UOS Protocol Test Corpora

This directory contains **golden test vectors** for the Rust port of the UOS
(Universal Offline Signatures) protocol.  Each JSON file encodes a fixed input
and the expected binary output.  The corpus tests in `tests/uos_*_corpus.rs`
load these files and assert that the Rust implementation produces identical bytes.

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
├── payload/              # UosPayload encode/decode tests
│   ├── sign_tx_empty.json
│   ├── sign_tx_1byte.json
│   ├── sign_tx_1024bytes.json
│   ├── sign_tx_5000bytes.json
│   ├── sign_immortal_*.json
│   ├── sign_hash_*.json
│   └── sign_message_*.json
├── signature/            # UosSignature encode/decode tests
│   ├── sr25519_zeros.json
│   ├── sr25519_nonzero.json
│   ├── ed25519_*.json
│   └── ecdsa_*.json
├── multipart/            # MultiPartQr{Encoder,Decoder} tests
│   ├── single_frame.json
│   ├── three_frame_balanced.json
│   ├── three_frame_uneven_tail.json
│   └── large_5kb.json
└── account_introduction/ # AccountIntroduction URI tests
    ├── minimal.json
    ├── with_genesis.json
    ├── with_name_ascii.json
    ├── with_name_unicode.json
    └── with_name_reserved_chars.json
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

---

## Notes

- Account-introduction URI encoding parity: Java `URLEncoder` differs subtly
  from RFC 3986 (`*` and `~` unencoded, spaces as `+`).  The Rust helper
  in `uos/account_introduction.rs` mirrors Java's rules exactly.  See Phase 1
  open position 3 in the restructure plan for a discussion of the implications.
- The `large_5kb` multipart corpus file contains a 5 000-byte all-zero payload
  split across 5 frames and is the largest single test vector.  CI loads it at
  test runtime; there is no inlining via `include_bytes!`.
