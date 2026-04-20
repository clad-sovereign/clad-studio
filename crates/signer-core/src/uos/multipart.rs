use std::collections::HashMap;
use std::sync::Mutex;

use crate::uos::constants::{FRAME_HEADER_SIZE, FRAME_SIZE, SUBSTRATE_ID};
use crate::uos::error::UosError;

/// The result of calling [`MultiPartQrDecoder::add_frame`].
///
/// Exposed to UniFFI as a `dictionary` so every field is addressable from
/// Kotlin/Swift without a sealed-class hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameDecodeProgress {
    /// `true` when all frames have been received and `complete_data` is present.
    pub is_complete: bool,
    /// Present (and non-empty) only when `is_complete` is `true`.
    pub complete_data: Option<Vec<u8>>,
    /// Frames received so far (including this one).
    pub received: u32,
    /// Total frames expected; 0 before the first multi-part frame is seen
    /// (a single-frame payload immediately sets `is_complete`).
    pub total: u32,
    /// Non-`None` when the frame was rejected by the decoder (e.g. malformed
    /// header).  This is a soft error: the decoder remains usable.
    pub error_message: Option<String>,
}

/// Splits a binary payload into one or more QR-code frames.
///
/// Single-frame payloads are returned as-is (no header).
/// Multi-frame payloads receive a 4-byte big-endian header on every frame:
///
/// ```text
/// ┌──────────────────┬──────────────────┬──────────────┐
/// │ Frame index (2B) │ Frame count (2B) │ Data (≤1020B)│
/// │   big-endian     │   big-endian     │              │
/// └──────────────────┴──────────────────┴──────────────┘
/// ```
///
/// Exposed to UniFFI as an `interface` (stateless, but needs `Arc` wrapping).
pub struct MultiPartQrEncoder {
    frame_size: usize,
}

impl MultiPartQrEncoder {
    /// Creates an encoder with the default 1024-byte frame size.
    pub fn new() -> Self {
        Self { frame_size: FRAME_SIZE }
    }

    /// Creates an encoder with a custom frame size (useful for tests).
    pub fn with_frame_size(frame_size: usize) -> Self {
        Self { frame_size }
    }

    /// Encodes `payload` into one or more QR frames.
    ///
    /// If the payload fits in a single frame it is returned verbatim.
    /// Otherwise every chunk is wrapped in the 4-byte header.
    pub fn encode(&self, payload: Vec<u8>) -> Vec<Vec<u8>> {
        let data_per_frame = self.frame_size - FRAME_HEADER_SIZE;
        let frame_count = Self::frame_count_for(payload.len(), data_per_frame);

        if frame_count == 1 {
            return vec![payload];
        }

        (0..frame_count)
            .map(|idx| {
                let start = idx * data_per_frame;
                let end = (start + data_per_frame).min(payload.len());
                let chunk = &payload[start..end];

                let mut frame = Vec::with_capacity(FRAME_HEADER_SIZE + chunk.len());
                frame.push(((idx >> 8) & 0xFF) as u8);
                frame.push((idx & 0xFF) as u8);
                frame.push(((frame_count >> 8) & 0xFF) as u8);
                frame.push((frame_count & 0xFF) as u8);
                frame.extend_from_slice(chunk);
                frame
            })
            .collect()
    }

    /// Returns `true` if `payload` fits in a single frame (no header needed).
    pub fn is_single_frame(&self, payload: Vec<u8>) -> bool {
        let data_per_frame = self.frame_size - FRAME_HEADER_SIZE;
        payload.len() <= data_per_frame
    }

    /// Returns the number of frames that would be produced for a payload of
    /// `payload_size` bytes.
    pub fn frame_count(&self, payload_size: u64) -> u32 {
        let data_per_frame = self.frame_size - FRAME_HEADER_SIZE;
        Self::frame_count_for(payload_size as usize, data_per_frame) as u32
    }

    fn frame_count_for(payload_size: usize, data_per_frame: usize) -> usize {
        if payload_size == 0 {
            return 1;
        }
        payload_size.div_ceil(data_per_frame)
    }
}

impl Default for MultiPartQrEncoder {
    fn default() -> Self {
        Self::new()
    }
}

// ── Internal mutable state (kept behind Mutex for UniFFI thread-safety) ──────

struct DecoderState {
    frames: HashMap<usize, Vec<u8>>,
    expected_count: Option<usize>,
}

impl DecoderState {
    fn new() -> Self {
        Self { frames: HashMap::new(), expected_count: None }
    }

    fn reset(&mut self) {
        self.frames.clear();
        self.expected_count = None;
    }
}

/// Reassembles a multi-part QR scan into the original payload.
///
/// Thread-safe via an internal `Mutex` so it can be held behind `Arc` as
/// required by UniFFI `interface` types.  Handles out-of-order frame arrival.
/// Automatically detects single-frame payloads (which start with `0x53` and
/// carry no header) and returns them immediately.
pub struct MultiPartQrDecoder {
    state: Mutex<DecoderState>,
}

impl MultiPartQrDecoder {
    pub fn new() -> Self {
        Self { state: Mutex::new(DecoderState::new()) }
    }

    /// Feeds a raw QR frame into the decoder.
    ///
    /// Returns a [`FrameDecodeProgress`] describing the current reassembly
    /// state.  Hard errors (e.g. a zero frame-count in the header) propagate
    /// via `Err(UosError)`; soft errors (malformed individual frames) are
    /// surfaced in `progress.error_message` so the user can keep scanning.
    pub fn add_frame(&self, data: Vec<u8>) -> Result<FrameDecodeProgress, UosError> {
        let mut s = self.state.lock().unwrap();

        // Single-frame detection: payload starts with the Substrate magic byte.
        if data.first() == Some(&SUBSTRATE_ID) {
            return Ok(FrameDecodeProgress {
                is_complete: true,
                complete_data: Some(data),
                received: 1,
                total: 1,
                error_message: None,
            });
        }

        if data.len() < FRAME_HEADER_SIZE {
            return Ok(FrameDecodeProgress {
                is_complete: false,
                complete_data: None,
                received: s.frames.len() as u32,
                total: s.expected_count.unwrap_or(0) as u32,
                error_message: Some(format!("frame too short: {} bytes", data.len())),
            });
        }

        let index = (usize::from(data[0]) << 8) | usize::from(data[1]);
        let count = (usize::from(data[2]) << 8) | usize::from(data[3]);

        if count == 0 {
            return Err(UosError::MalformedFrameHeader);
        }

        match s.expected_count {
            None => {
                s.expected_count = Some(count);
            }
            Some(prev) if prev != count => {
                s.reset();
                s.expected_count = Some(count);
            }
            _ => {}
        }

        if index >= count {
            return Ok(FrameDecodeProgress {
                is_complete: false,
                complete_data: None,
                received: s.frames.len() as u32,
                total: count as u32,
                error_message: Some(format!("frame index {index} out of range for count {count}")),
            });
        }

        s.frames.insert(index, data[FRAME_HEADER_SIZE..].to_vec());

        let received = s.frames.len() as u32;
        let total = count as u32;

        if received == total {
            let assembled =
                assemble_frames(&s.frames, count).ok_or(UosError::MalformedFrameHeader)?;
            Ok(FrameDecodeProgress {
                is_complete: true,
                complete_data: Some(assembled),
                received,
                total,
                error_message: None,
            })
        } else {
            Ok(FrameDecodeProgress {
                is_complete: false,
                complete_data: None,
                received,
                total,
                error_message: None,
            })
        }
    }

    /// Resets the decoder for a new scan session.
    pub fn reset(&self) {
        self.state.lock().unwrap().reset();
    }

    /// Number of distinct frames received so far.
    pub fn received_count(&self) -> u32 {
        self.state.lock().unwrap().frames.len() as u32
    }

    /// Total expected frame count, or `None` if no multi-part frame seen yet.
    pub fn total_count(&self) -> Option<u32> {
        self.state.lock().unwrap().expected_count.map(|c| c as u32)
    }
}

impl Default for MultiPartQrDecoder {
    fn default() -> Self {
        Self::new()
    }
}

fn assemble_frames(frames: &HashMap<usize, Vec<u8>>, total: usize) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    for i in 0..total {
        result.extend_from_slice(frames.get(&i)?);
    }
    Some(result)
}
