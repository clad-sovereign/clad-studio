/// Known-answer tests for `MultiPartQrEncoder` and `MultiPartQrDecoder`.
///
/// JSON files live in `tests/corpora/multipart/`.
/// Each file records a payload and the frames produced by the encoder.
/// Tests verify:
///   1. Encoder output matches expected frames.
///   2. Decoder reassembles frames (in order) back to the original payload.
///   3. Decoder handles out-of-order frame delivery.
use signer_core::uos::multipart::{FrameDecodeProgress, MultiPartQrDecoder, MultiPartQrEncoder};
use std::path::Path;

fn corpus_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpora/multipart")
}

#[test]
fn multipart_corpus_encoder() {
    let dir = corpus_dir();
    let mut count = 0;

    for entry in std::fs::read_dir(&dir).expect("corpus/multipart dir missing") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let raw = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();

        let payload_hex = v["payload_hex"].as_str().unwrap();
        let payload =
            hex::decode(payload_hex).unwrap_or_else(|_| panic!("{path:?}: bad payload_hex"));

        let expected_frames: Vec<Vec<u8>> = v["frames_hex"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| hex::decode(f.as_str().unwrap()).unwrap())
            .collect();

        let enc = MultiPartQrEncoder::new();
        let actual_frames = enc.encode(payload.clone());

        assert_eq!(actual_frames.len(), expected_frames.len(), "{path:?}: frame count mismatch");
        for (i, (actual, expected)) in actual_frames.iter().zip(expected_frames.iter()).enumerate()
        {
            assert_eq!(actual, expected, "{path:?}: frame {i} mismatch");
        }

        count += 1;
    }

    assert!(count > 0, "no multipart corpus files found in {dir:?}");
}

#[test]
fn multipart_corpus_decoder_in_order() {
    let dir = corpus_dir();

    for entry in std::fs::read_dir(&dir).expect("corpus/multipart dir missing") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let raw = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();

        let payload_hex = v["payload_hex"].as_str().unwrap();
        let expected_payload = hex::decode(payload_hex).unwrap();

        let frames: Vec<Vec<u8>> = v["frames_hex"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| hex::decode(f.as_str().unwrap()).unwrap())
            .collect();

        let dec = MultiPartQrDecoder::new();
        let mut result: Option<Vec<u8>> = None;

        for frame in &frames {
            match dec.add_frame(frame.clone()).expect("add_frame error") {
                FrameDecodeProgress { is_complete: true, complete_data: Some(data), .. } => {
                    result = Some(data);
                }
                FrameDecodeProgress { is_complete: false, error_message: Some(msg), .. } => {
                    panic!("{path:?}: frame decode soft error: {msg}");
                }
                _ => {}
            }
        }

        let assembled = result.unwrap_or_else(|| panic!("{path:?}: decoder never completed"));
        assert_eq!(assembled, expected_payload, "{path:?}: assembled payload mismatch");
    }
}

#[test]
fn multipart_corpus_decoder_out_of_order() {
    let dir = corpus_dir();

    for entry in std::fs::read_dir(&dir).expect("corpus/multipart dir missing") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }

        let raw = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();

        let payload_hex = v["payload_hex"].as_str().unwrap();
        let expected_payload = hex::decode(payload_hex).unwrap();

        let mut frames: Vec<Vec<u8>> = v["frames_hex"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| hex::decode(f.as_str().unwrap()).unwrap())
            .collect();

        if frames.len() <= 1 {
            // Single-frame: out-of-order doesn't apply; just verify it works.
            let dec = MultiPartQrDecoder::new();
            let progress = dec.add_frame(frames[0].clone()).unwrap();
            assert!(progress.is_complete);
            assert_eq!(progress.complete_data.unwrap(), expected_payload);
            continue;
        }

        // Reverse the frame order.
        frames.reverse();

        let dec = MultiPartQrDecoder::new();
        let mut result: Option<Vec<u8>> = None;

        for frame in &frames {
            if let FrameDecodeProgress { is_complete: true, complete_data: Some(data), .. } =
                dec.add_frame(frame.clone()).unwrap()
            {
                result = Some(data);
            }
        }

        let assembled =
            result.unwrap_or_else(|| panic!("{path:?}: out-of-order decoder never completed"));
        assert_eq!(
            assembled, expected_payload,
            "{path:?}: out-of-order assembled payload mismatch"
        );
    }
}
