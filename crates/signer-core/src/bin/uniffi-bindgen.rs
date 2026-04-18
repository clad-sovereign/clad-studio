//! Standalone binary wrapping UniFFI's bindings generator.
//!
//! UniFFI ships its CLI as a library feature rather than a published binary;
//! the recommended pattern is for each consuming crate to expose its own thin
//! wrapper so the generator version and the library version are guaranteed
//! to match. Invoked by `build-android.sh` and `build-ios.sh` (and the CI
//! equivalents) as:
//!
//!     cargo run -p signer-core --features uniffi-cli --bin uniffi-bindgen -- <args>

fn main() {
    uniffi::uniffi_bindgen_main()
}
