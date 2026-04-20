/// Property-based tests for the UOS codec, using `proptest`.
///
/// Properties verified:
///   1. `decode(encode(payload)) == payload` for all valid UosPayload shapes.
///   2. `decode(encode(sig)) == sig` for all valid UosSignature shapes.
///   3. Reassembling the encoder's frame set (any permutation) always gives
///      back the original payload.
use proptest::prelude::*;
use signer_core::uos::{
    constants::{
        ACCOUNT_ID_LENGTH, CMD_SIGN_HASH, CMD_SIGN_IMMORTAL, CMD_SIGN_MSG, CMD_SIGN_TX,
        CRYPTO_ECDSA, CRYPTO_ED25519, CRYPTO_SR25519, SIGNATURE_LENGTH_ECDSA,
        SIGNATURE_LENGTH_ED25519_SR25519,
    },
    multipart::{FrameDecodeProgress, MultiPartQrDecoder, MultiPartQrEncoder},
    payload::UosPayload,
    signature::UosSignature,
};

// ── Payload round-trip ────────────────────────────────────────────────────────

fn arb_crypto_type() -> impl Strategy<Value = u8> {
    prop_oneof![Just(CRYPTO_ED25519), Just(CRYPTO_SR25519), Just(CRYPTO_ECDSA),]
}

fn arb_action() -> impl Strategy<Value = u8> {
    prop_oneof![Just(CMD_SIGN_TX), Just(CMD_SIGN_HASH), Just(CMD_SIGN_IMMORTAL), Just(CMD_SIGN_MSG),]
}

fn arb_account_id() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(any::<u8>(), ACCOUNT_ID_LENGTH)
}

fn arb_inner_payload() -> impl Strategy<Value = Vec<u8>> {
    // 0..=8192 bytes to cover empty, small, multi-frame, and large
    proptest::collection::vec(any::<u8>(), 0..=8192_usize)
}

proptest! {
    #[test]
    fn payload_encode_decode_roundtrip(
        crypto_type in arb_crypto_type(),
        action in arb_action(),
        account_id in arb_account_id(),
        payload in arb_inner_payload(),
    ) {
        let original = UosPayload::new(crypto_type, action, account_id.clone(), payload.clone());
        let encoded = original.encode().expect("encode must not fail for valid input");
        let decoded = UosPayload::decode(&encoded).expect("decode must not fail on valid encoded bytes");
        prop_assert_eq!(decoded.crypto_type, crypto_type);
        prop_assert_eq!(decoded.action, action);
        prop_assert_eq!(decoded.account_id, account_id);
        prop_assert_eq!(decoded.payload, payload);
    }
}

// ── Signature round-trip ──────────────────────────────────────────────────────

fn arb_sr25519_sig() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(any::<u8>(), SIGNATURE_LENGTH_ED25519_SR25519)
}
fn arb_ecdsa_sig() -> impl Strategy<Value = Vec<u8>> {
    proptest::collection::vec(any::<u8>(), SIGNATURE_LENGTH_ECDSA)
}

proptest! {
    #[test]
    fn signature_sr25519_roundtrip(sig in arb_sr25519_sig()) {
        let original = UosSignature::new(CRYPTO_SR25519, sig.clone()).unwrap();
        let encoded = original.encode();
        let decoded = UosSignature::decode(&encoded).unwrap();
        prop_assert_eq!(decoded.crypto_type, CRYPTO_SR25519);
        prop_assert_eq!(decoded.signature, sig);
    }

    #[test]
    fn signature_ed25519_roundtrip(sig in arb_sr25519_sig()) {
        let original = UosSignature::new(CRYPTO_ED25519, sig.clone()).unwrap();
        let encoded = original.encode();
        let decoded = UosSignature::decode(&encoded).unwrap();
        prop_assert_eq!(decoded.crypto_type, CRYPTO_ED25519);
        prop_assert_eq!(decoded.signature, sig);
    }

    #[test]
    fn signature_ecdsa_roundtrip(sig in arb_ecdsa_sig()) {
        let original = UosSignature::new(CRYPTO_ECDSA, sig.clone()).unwrap();
        let encoded = original.encode();
        let decoded = UosSignature::decode(&encoded).unwrap();
        prop_assert_eq!(decoded.crypto_type, CRYPTO_ECDSA);
        prop_assert_eq!(decoded.signature, sig);
    }
}

// ── Multipart frame-set permutation property ──────────────────────────────────

// For any payload, feeding the encoder's frames in any permutation to the
// decoder should reassemble the original payload.
//
// Design constraint: the MultiPartQrDecoder detects single-frame payloads by
// checking for the 0x53 substrate magic byte. Single-frame payloads that do
// NOT start with 0x53 are indistinguishable from a truncated multi-part frame.
// In practice every real UOS payload begins with 0x53; this property test
// covers (a) multi-frame payloads of any byte content, and (b) single-frame
// payloads that start with 0x53.
proptest! {
    #[test]
    fn multipart_permutation_reassembles(
        // Force the first byte to be 0x53 to ensure single-frame detection works;
        // the rest of the payload is arbitrary.
        rest in proptest::collection::vec(any::<u8>(), 0..=5999_usize),
        seed in any::<u64>(),
    ) {
        // Build payload: always starts with 0x53.
        let mut payload = vec![0x53u8];
        payload.extend_from_slice(&rest);

        let enc = MultiPartQrEncoder::new();
        let frames = enc.encode(payload.clone());

        // Build a permuted frame order using a deterministic Fisher-Yates shuffle.
        let mut indices: Vec<usize> = (0..frames.len()).collect();
        let mut rng_state = seed;
        for i in (1..indices.len()).rev() {
            rng_state = rng_state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let j = (rng_state >> 33) as usize % (i + 1);
            indices.swap(i, j);
        }

        let dec = MultiPartQrDecoder::new();
        let mut result: Option<Vec<u8>> = None;

        for &idx in &indices {
            if let FrameDecodeProgress { is_complete: true, complete_data: Some(data), .. } = dec.add_frame(frames[idx].clone()).expect("add_frame must not hard-error on valid frames") {
                result = Some(data);
            }
        }

        let assembled = result.expect("decoder must complete after all frames delivered");
        prop_assert_eq!(assembled, payload);
    }
}
