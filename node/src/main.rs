mod chain_spec;
mod cli;
mod command;
mod rpc;
mod service;

// sc_cli::Error is 176+ bytes due to Substrate's comprehensive error variants.
// This is acceptable for the entry point which is called once at startup.
#[allow(clippy::result_large_err)]
fn main() -> sc_cli::Result<()> {
    command::run()
}
