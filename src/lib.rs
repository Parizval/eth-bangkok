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
    alloy_primitives::{Address, FixedBytes, U256},
    call::Call,
    contract,
    crypto::keccak,
    evm, msg,
    prelude::*,
};

use alloy_sol_types::{sol, sol_data::Address as SOLAddress, SolType};

const OWNER: &str = "0x9C96CFe9A37605bdb2D1462022265754f76B5E4B";

// Define some persistent storage using the Solidity ABI.
// `LendingHook` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct LendingHook {

        // Token-to-vault address mappings
        mapping(address => address) aave_contracts;

        mapping(address=> address) fluid_contracts;

        mapping(address=> address) compound_contracts;
    }
}

sol! {

    // Errors
    error InsufficentTokenBalance();

    error ApproveCallFailed();

    error TokenTransferFailed();

    error DepositCallFailed();

    error NotOwnerAddress();

    // Evm Events
    event AddedVault (address sender, address token, address vault);

    event Deposit(address sender, address token, address vault, address recipient);

    event RecoverToken(address sender, address token, address recipient);

}

#[derive(SolidityError)]
pub enum LendingHookErrors {
    InsufficentTokenBalance(InsufficentTokenBalance),
    ApproveCallFailed(ApproveCallFailed),
    TokenTransferFailed(TokenTransferFailed),
    DepositCallFailed(DepositCallFailed),
    NotOwnerAddress(NotOwnerAddress),
}

sol_interface! {

    interface IERC20{

        function balanceOf(address account) external view returns (uint256);

        function approve(address spender, uint256 amount) external returns (bool);

        function transfer(address recipient, uint256 amount)
              external
              returns (bool);

    }

    interface Aave{

        function supply(
            address asset,
            uint256 amount,
            address onBehalfOf,
            uint16 referralCode
        ) external ;
    }

    interface Fluidx {

        function deposit(uint256 assets_, address receiver_) external;
    }

    interface Compound {

        function supplyTo(address dst, address asset, uint amount) external ;

    }
}

/// Declare that `LendingHook` is a contract with the following external methods.
#[public]
impl LendingHook {
    pub fn aave(&mut self, token: Address, recipient: Address) -> Result<(), LendingHookErrors> {
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

        let aave_contract_address = self.aave_contracts.get(token);

        // Deposit Call
        let deposit_contract = Aave::new(aave_contract_address);
        let config = Call::new_in(self);

        evm::log(Deposit {
            sender: msg::sender(),
            token,
            vault: aave_contract_address,
            recipient,
        });

        match deposit_contract.supply(config, token, token_balance, recipient, u16::MIN) {
            Ok(_) => Ok(()),
            Err(_) => Err(LendingHookErrors::DepositCallFailed(DepositCallFailed {})),
        }
    }

    pub fn compound(
        &mut self,
        token: Address,
        recipient: Address,
    ) -> Result<(), LendingHookErrors> {
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

        let compound_contract = self.compound_contracts.get(token);

        let vault = Compound::new(compound_contract);
        let config = Call::new_in(self);

        evm::log(Deposit {
            sender: msg::sender(),
            token,
            vault: compound_contract,
            recipient,
        });

        match vault.supply_to(config, recipient, token, token_balance) {
            Ok(_) => Ok(()),
            Err(_) => Err(LendingHookErrors::DepositCallFailed(DepositCallFailed {})),
        }
    }

    pub fn fluid(&mut self, token: Address, recipient: Address) -> Result<(), LendingHookErrors> {
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

        let fluidx_contract = self.compound_contracts.get(token);

        let vault = Fluidx::new(fluidx_contract);
        let config = Call::new_in(self);

        evm::log(Deposit {
            sender: msg::sender(),
            token,
            vault: fluidx_contract,
            recipient,
        });

        match vault.deposit(config, token_balance, recipient) {
            Ok(_) => Ok(()),
            Err(_) => Err(LendingHookErrors::DepositCallFailed(DepositCallFailed {})),
        }
    }

    pub fn get_aave_vault(&self, token: Address) -> Address {
        let token_vault = self.aave_contracts.getter(token);
        token_vault.get()
    }

    pub fn get_compound_vault(&self, token: Address) -> Address {
        let token_vault = self.compound_contracts.getter(token);
        token_vault.get()
    }

    pub fn get_fluid_vault(&self, token: Address) -> Address {
        let token_vault = self.fluid_contracts.getter(token);
        token_vault.get()
    }

    pub fn get_call_data(&self, func: String, token: Address, recipient: Address) -> Vec<u8> {
        type DepositType = (SOLAddress, SOLAddress);

        let deposit_data = (token, recipient);

        let data = DepositType::abi_encode_sequence(&deposit_data);

        let hashed_function_selector: FixedBytes<32> = keccak(func.as_bytes().to_vec()).into();

        let calldata = [&hashed_function_selector[..4], &data].concat();

        calldata
    }

    pub fn add_aave_vault(
        &mut self,
        token: Address,
        vault: Address,
    ) -> Result<(), LendingHookErrors> {
        // Owner Address Check
        let owner_address = Address::parse_checksummed(OWNER, None).expect("Invalid Address");

        if msg::sender() != owner_address {
            return Err(LendingHookErrors::NotOwnerAddress(NotOwnerAddress {}));
        }

        // Store Vault Address
        let mut token_vault = self.aave_contracts.setter(token);
        token_vault.set(vault);

        evm::log(AddedVault {
            sender: msg::sender(),
            token,
            vault,
        });

        // Infinite Approve Call
        let token_contract = IERC20::new(token);
        let config = Call::new_in(self);
        match token_contract.approve(config, vault, U256::MAX) {
            Ok(_) => Ok(()),
            Err(_) => Err(LendingHookErrors::ApproveCallFailed(ApproveCallFailed {})),
        }
    }

    pub fn recover_token(
        &mut self,
        token: Address,
        recipient: Address,
    ) -> Result<(), LendingHookErrors> {
        // Owner Address Check
        let owner_address = Address::parse_checksummed(OWNER, None).expect("Invalid Address");

        if msg::sender() != owner_address {
            return Err(LendingHookErrors::NotOwnerAddress(NotOwnerAddress {}));
        }

        // Get token balance
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

        evm::log(RecoverToken {
            sender: msg::sender(),
            token,
            recipient,
        });

        // Transfer tokens
        let config = Call::new_in(self);
        match token_contract.transfer(config, recipient, token_balance) {
            Ok(_) => Ok(()),
            Err(_) => Err(LendingHookErrors::TokenTransferFailed(
                TokenTransferFailed {},
            )),
        }
    }
}

impl LendingHook {
    pub fn calculate_compound_interest_rate(&self) -> U256 {
        U256::ZERO
    }
}

// #[cfg(test)]
// mod tests {

//     use super::*;

//     #[motsu::test]
//     fn it_gets_calldata(contract: LendingHook) {
//         let testnet_token_address = "0xb1D4538B4571d411F07960EF2838Ce337FE1E80E";

//         let recipient_address = "0xE451141fCE63EB38e85F08a991fC5878Ee6335b2";

//         let call_data = contract.get_call_data(
//             "deposit".to_string(),
//             testnet_token_address,
//             recipient_address,
//         );
//     }
// }
