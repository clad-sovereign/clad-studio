#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::construct_runtime;
use pallet_clad_token as clad_token;

construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system,
        CladToken: clad_token,
    }
);
