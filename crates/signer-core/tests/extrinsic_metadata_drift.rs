//! Metadata corpus drift-detect test.
//!
//! Reads the checked-in `tests/corpora/metadata/metadata_v14.scale` blob and
//! asserts that the pallet indices for `CladToken` and `Multisig` match the
//! hard-coded constants in `src/extrinsic/call.rs` and `src/extrinsic/metadata.rs`.
//!
//! Runs on every `cargo test` without a live node — no feature flag, no `#[ignore]`.
//!
//! If this test fails:
//!   - A runtime upgrade changed a pallet or call index.
//!   - Regenerate the corpus: `./scripts/regen-metadata-corpus.sh` (requires a
//!     running `clad-node --dev` instance).
//!   - Audit the diff to ensure the constant changes in `call.rs` are intentional.
//!
//! Implementation note: uses a hand-rolled SCALE cursor to avoid pulling in
//! `frame-metadata` or `subxt-metadata` as dependencies, which have transitive
//! `std`-only paths incompatible with `signer-core`'s `no_std + alloc` constraint.

use signer_core::extrinsic::call::CLAD_TOKEN_PALLET;
use signer_core::extrinsic::metadata::KNOWN_PALLETS;
use std::path::Path;

const SCALE_MAGIC: &[u8; 4] = b"meta"; // 0x6d657461
const METADATA_V14: u8 = 14;

#[test]
fn metadata_v14_pallet_indices_match_constants() {
    let corpus_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpora/metadata/metadata_v14.scale");

    let data = std::fs::read(&corpus_path).unwrap_or_else(|e| {
        panic!(
            "failed to read metadata corpus at {}: {}\n\
             Run ./scripts/regen-metadata-corpus.sh with a live clad-node to generate it.",
            corpus_path.display(),
            e
        )
    });

    assert!(data.len() > 5, "corpus file too short (len={})", data.len());
    assert_eq!(&data[..4], SCALE_MAGIC, "metadata magic mismatch — corpus may be corrupted");
    assert_eq!(data[4], METADATA_V14, "expected metadata V14, got V{}", data[4]);

    let mut cursor = ScaleCursor::new(&data[5..]);
    cursor.skip_portable_registry();
    let pallets = cursor.read_pallets();

    // Assert CladToken pallet index.
    let clad = pallets
        .iter()
        .find(|(name, _)| name == "CladToken")
        .unwrap_or_else(|| panic!("CladToken not found in metadata pallets: {pallets:?}"));
    assert_eq!(
        clad.1, CLAD_TOKEN_PALLET,
        "CladToken pallet index drift: metadata={}, constant={}",
        clad.1, CLAD_TOKEN_PALLET
    );

    // Assert Multisig pallet index.
    let multisig_constant = KNOWN_PALLETS
        .iter()
        .find(|(name, _)| *name == "Multisig")
        .map(|(_, idx)| *idx)
        .expect("Multisig not present in KNOWN_PALLETS");
    let multisig = pallets
        .iter()
        .find(|(name, _)| name == "Multisig")
        .unwrap_or_else(|| panic!("Multisig not found in metadata pallets: {pallets:?}"));
    assert_eq!(
        multisig.1, multisig_constant,
        "Multisig pallet index drift: metadata={}, constant={}",
        multisig.1, multisig_constant
    );
}

// ── Hand-rolled SCALE cursor ──────────────────────────────────────────────────
//
// Parses just enough of the Metadata V15 binary format to extract pallet names
// and indices without requiring frame-metadata or subxt-metadata as dependencies.
//
// Structure traversed:
//   magic(4) + version(1) + RuntimeMetadataV14 {
//     types: PortableRegistry,   <-- skipped
//     pallets: Vec<PalletMetadata>,  <-- parsed (name + index only)
//     ...                            <-- not parsed
//   }

struct ScaleCursor<'a> {
    data: &'a [u8],
}

impl<'a> ScaleCursor<'a> {
    fn new(data: &'a [u8]) -> Self {
        ScaleCursor { data }
    }

    fn read_u8(&mut self) -> u8 {
        assert!(!self.data.is_empty(), "SCALE cursor underrun");
        let b = self.data[0];
        self.data = &self.data[1..];
        b
    }

    fn skip_n(&mut self, n: usize) {
        assert!(
            self.data.len() >= n,
            "SCALE cursor underrun: need {n} bytes, have {}",
            self.data.len()
        );
        self.data = &self.data[n..];
    }

    fn read_compact(&mut self) -> u64 {
        let b0 = self.read_u8();
        match b0 & 0b11 {
            0b00 => (b0 >> 2) as u64,
            0b01 => {
                let b1 = self.read_u8();
                (u16::from_le_bytes([b0, b1]) >> 2) as u64
            }
            0b10 => {
                let b1 = self.read_u8();
                let b2 = self.read_u8();
                let b3 = self.read_u8();
                (u32::from_le_bytes([b0, b1, b2, b3]) >> 2) as u64
            }
            _ => {
                // Big-integer mode: (b0 >> 2) + 4 bytes encode the value LE.
                let n = (b0 >> 2) as usize + 4;
                self.skip_n(n);
                0 // value not needed for traversal
            }
        }
    }

    fn skip_compact(&mut self) {
        self.read_compact();
    }

    fn read_string(&mut self) -> String {
        let len = self.read_compact() as usize;
        let bytes = &self.data[..len];
        self.skip_n(len);
        String::from_utf8(bytes.to_vec()).expect("SCALE string must be valid UTF-8")
    }

    fn skip_string(&mut self) {
        let len = self.read_compact() as usize;
        self.skip_n(len);
    }

    fn skip_option_string(&mut self) {
        let tag = self.read_u8();
        match tag {
            0x00 => {}
            0x01 => self.skip_string(),
            _ => panic!("invalid Option<String> tag: {tag:#x}"),
        }
    }

    fn skip_vec_string(&mut self) {
        let n = self.read_compact() as usize;
        for _ in 0..n {
            self.skip_string();
        }
    }

    fn skip_bytes_blob(&mut self) {
        let n = self.read_compact() as usize;
        self.skip_n(n);
    }

    // ── PortableRegistry ──────────────────────────────────────────────────

    fn skip_portable_registry(&mut self) {
        let n = self.read_compact() as usize;
        for _ in 0..n {
            self.skip_portable_type();
        }
    }

    fn skip_portable_type(&mut self) {
        self.skip_compact(); // id: Compact<u32>
        self.skip_type();
    }

    fn skip_type(&mut self) {
        // path: Vec<String>
        self.skip_vec_string();
        // type_params: Vec<TypeParameter>
        let n = self.read_compact() as usize;
        for _ in 0..n {
            self.skip_string(); // name
            let tag = self.read_u8(); // ty: Option<Compact<u32>>
            if tag == 0x01 {
                self.skip_compact();
            }
        }
        // type_def: TypeDef (enum)
        self.skip_type_def();
        // docs: Vec<String>
        self.skip_vec_string();
    }

    fn skip_type_def(&mut self) {
        let tag = self.read_u8();
        match tag {
            0 => {
                // Composite: Vec<Field>
                let n = self.read_compact() as usize;
                for _ in 0..n {
                    self.skip_field();
                }
            }
            1 => {
                // Variant: Vec<Variant>
                let n = self.read_compact() as usize;
                for _ in 0..n {
                    self.skip_variant();
                }
            }
            2 => self.skip_compact(), // Sequence: Compact<u32>
            3 => {
                self.skip_n(4); // Array: u32 LE (len)
                self.skip_compact(); // type_param: Compact<u32>
            }
            4 => {
                // Tuple: Vec<Compact<u32>>
                let n = self.read_compact() as usize;
                for _ in 0..n {
                    self.skip_compact();
                }
            }
            5 => {
                self.read_u8();
            } // Primitive: u8 enum tag
            6 => self.skip_compact(), // Compact: Compact<u32>
            7 => {
                // BitSequence: bit_store_type + bit_order_type
                self.skip_compact();
                self.skip_compact();
            }
            _ => panic!("unknown TypeDef variant tag: {tag}"),
        }
    }

    // Field = Option<String> + Compact<u32> + Option<String> + Vec<String>
    fn skip_field(&mut self) {
        self.skip_option_string(); // name
        self.skip_compact(); // ty: Compact<u32>
        self.skip_option_string(); // type_name
        self.skip_vec_string(); // docs
    }

    // Variant = String + Vec<Field> + u8 + Vec<String>
    fn skip_variant(&mut self) {
        self.skip_string(); // name
        let n = self.read_compact() as usize;
        for _ in 0..n {
            self.skip_field();
        }
        self.read_u8(); // index: u8
        self.skip_vec_string(); // docs
    }

    // ── Pallets ───────────────────────────────────────────────────────────

    fn read_pallets(&mut self) -> Vec<(String, u8)> {
        let n = self.read_compact() as usize;
        (0..n).map(|_| self.read_pallet()).collect()
    }

    fn read_pallet(&mut self) -> (String, u8) {
        let name = self.read_string();

        // storage: Option<PalletStorageMetadata>
        if self.read_u8() == 0x01 {
            self.skip_pallet_storage();
        }
        // calls: Option<{ ty: Compact<u32> }>
        if self.read_u8() == 0x01 {
            self.skip_compact();
        }
        // event: Option<{ ty: Compact<u32> }>
        if self.read_u8() == 0x01 {
            self.skip_compact();
        }
        // constants: Vec<PalletConstantMetadata>
        let nc = self.read_compact() as usize;
        for _ in 0..nc {
            self.skip_pallet_constant();
        }
        // error: Option<{ ty: Compact<u32> }>
        if self.read_u8() == 0x01 {
            self.skip_compact();
        }

        let index = self.read_u8();
        (name, index)
    }

    fn skip_pallet_storage(&mut self) {
        self.skip_string(); // prefix
        let n = self.read_compact() as usize;
        for _ in 0..n {
            self.skip_storage_entry();
        }
    }

    fn skip_storage_entry(&mut self) {
        self.skip_string(); // name
        self.read_u8(); // modifier: u8 (Optional=0, Default=1)
                        // ty: StorageEntryType (enum)
        let tag = self.read_u8();
        match tag {
            0 => self.skip_compact(), // Plain: Compact<u32>
            1 => {
                // Map: Vec<StorageHasher> + key: Compact<u32> + value: Compact<u32>
                let nh = self.read_compact() as usize;
                self.skip_n(nh); // each StorageHasher is 1 byte (unit enum)
                self.skip_compact(); // key
                self.skip_compact(); // value
            }
            _ => panic!("unknown StorageEntryType tag: {tag}"),
        }
        self.skip_bytes_blob(); // default: Vec<u8>
        self.skip_vec_string(); // docs
    }

    // PalletConstantMetadata = String + Compact<u32> + Vec<u8> + Vec<String>
    fn skip_pallet_constant(&mut self) {
        self.skip_string(); // name
        self.skip_compact(); // ty: Compact<u32>
        self.skip_bytes_blob(); // value: Vec<u8>
        self.skip_vec_string(); // docs
    }
}
