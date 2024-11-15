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

use alloc::string::String;
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{Address, FixedBytes, U256},
    call::Call,
    contract,
    crypto::keccak,
    prelude::*,
};

use alloy_sol_types::{
    sol,
    sol_data::{Address as SOLAddress, Bytes as SOLBytes, String as SOLString, *},
    SolType,
};
// Define some persistent storage using the Solidity ABI.
// `LendingHook` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct LendingHook {

        // Token-to-vault address mappings
        //
        mapping(address => address) aave_contracts;
        // mapping(address=> address) fluid_contracts;
    }
}

sol! {
    error InsufficentTokenBalance();

    error ApproveCallFailed();
}

#[derive(SolidityError)]
pub enum LendingHookErrors {
    InsufficentTokenBalance(InsufficentTokenBalance),
    ApproveCallFailed(ApproveCallFailed),
}

sol_interface! {

    interface IERC20{

        function balanceOf(address account) external view returns (uint256);

        function approve(address spender, uint256 amount) external returns (bool);

    }

}

/// Declare that `LendingHook` is a contract with the following external methods.
#[public]
impl LendingHook {
    pub fn deposit(
        &mut self,
        token: Address,
        _recipient: Address,
    ) -> Result<(), LendingHookErrors> {
        // get contract token balance
        let token_contract = IERC20::new(token);
        let config = Call::new_in(self);
        let token_balance = token_contract
            .balance_of(config, contract::address())
            .unwrap_or(U256::from(0));

        if token_balance == U256::from(0) {
            return Err(LendingHookErrors::InsufficentTokenBalance(
                InsufficentTokenBalance {},
            ));
        }

        Ok(())
    }

    pub fn get_call_data(&self, func: String, token: Address, recipient: Address) -> Vec<u8> {
        type DepositType = (SOLAddress, SOLAddress);

        let deposit_data = (token, recipient);

        let data = DepositType::abi_encode_sequence(&deposit_data);

        let hashed_function_selector: FixedBytes<32> = keccak(func.as_bytes().to_vec()).into();

        let calldata = [&hashed_function_selector[..4], &data].concat();

        calldata
    }

    pub fn add_vault(&mut self, token: Address, vault: Address) -> Result<(), LendingHookErrors> {
        // Store Vault Address

        // Infinite Approve Call
        let token_contract = IERC20::new(token);
        let config = Call::new_in(self);
        match token_contract.approve(config, vault, U256::MAX) {
            Ok(_) => Ok(()),
            Err(_) => Err(LendingHookErrors::ApproveCallFailed(ApproveCallFailed {})),
        }
    }
}
