//! Compile-time glue that converts the UDL interface into Rust scaffolding.
//!
//! Runs before `lib.rs` is compiled so the `uniffi::include_scaffolding!`
//! macro in `lib.rs` has a generated file to pull in.

fn main() {
    uniffi::generate_scaffolding("src/signer_core.udl")
        .expect("failed to generate UniFFI scaffolding from src/signer_core.udl");
}
