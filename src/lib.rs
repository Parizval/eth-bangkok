//!
//! Stylus Hello World
//!
//! The following contract implements the Counter example from Foundry.
//!

//!
//! The program is ABI-equivalent with Solidity, which means you can call it from both Solidity and Rust.
//! To do this, run `cargo stylus export-abi`.
//!
//! Note: this code is a template-only and has not been audited.
//!

// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use alloy_primitives::FixedBytes;
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    crypto::keccak,
    prelude::*,
};

use alloy_sol_types::sol_data::Address as SOLAddress;

// Define some persistent storage using the Solidity ABI.
// `LendingHook` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct LendingHook {
        uint256 number;
    }
}

/// Declare that `LendingHook` is a contract with the following external methods.
#[public]
impl LendingHook {
    pub fn deposit(tokenAddress: Address, recipient: Address) {}

    pub fn getCallData(&self, func: String, tokenAddress: Address, recipient: Address) -> Vec<u8> {
        type DepositType = (SOLAddress, SOLAddress);

        let deposit_data = (tokenAddress, recipient);

        // let data = DepositType::abi_encode_sequence(&deposit_data);

        let hashed_function_selector: FixedBytes<32> = keccak(func.as_bytes().to_vec()).into();

        // TODO: Add function data
        let calldata = [&hashed_function_selector[..4]].concat();

        calldata
    }
}
