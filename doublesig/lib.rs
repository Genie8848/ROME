//! A smart contract that allows users to generate a double signature for the
//! distribution of funds every time they sign a transaction.
//! ## Overview
//!
//! This contract enables the user to save an amount of funds each time he makes
//! a `Transaction`. Currently, an additional fee of 3% of the transaction is deducted.
//! This fee is then stored in the contract and may act as a private retirement fund.
//! In this way, the user will be able to go little little by little storing a part
//! of their funds without hardly noticing it.
//!
//! ## Requirements
//!
//! When instantiating the contract, the user is to specify how long he would
//! like to keep saving funds this way. This is done through the `expiration` parameter.
//!
//! ### Withdrawals
//!
//! The user may opt out of this contract at anytime. This can be done using the
//! `terminate` method. After doing so, the funds accumulated would be transferred
//! to the user.
//!
//! ### Deposits
//! The creator of the contract, i.e the `sender`, can deposit funds to the payment
//! channel while creating the payment channel. Any subsequent deposits can be made by
//! transferring funds to the contract's address.
//! Ideally, a major part of the sender's funds should be transferred to this
//! contract and the user should then perform subsequent transactions through the
//! contract's interface.

#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod doublesig {
    use ink::codegen::Env;

    #[ink(storage)]
    pub struct Doublesig {
        user: AccountId,
        expiration: Timestamp,
        // store amount held as a decimal
        amount_held: Balance,
    }

    /// Errors that can occur upon calling this contract.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if caller is not the `sender` while required to.
        NotYetExpired,
        InsufficientFunds,
        TransferAmountTooLarge,
    }

    /// Type alias for the contract's `Result` type.
    pub type Result<T> = core::result::Result<T, Error>;

    impl Doublesig {
        /// Creates a new instance of this contract.
        /// `expiration` refers to how long you'd, like to keep your funds in this contract.
        /// After the expiration period, you are allowed to withdraw the funds.
        #[ink(constructor, payable)]
        pub fn new(expiration: Timestamp) -> Self {
            Self {
                user: Self::env().caller(),
                expiration,
                amount_held: 0,
            }
        }

        /// Transfer `amount` to specified destination. An addition 3% of the transaction
        /// would be deduced and stored
        #[ink(message)]
        pub fn transfer_funds(&mut self, destination: AccountId, amount: Balance) -> Result<()> {
            // ensure the amount held is greater than the contract's balance
            let balance = self.get_balance();
            let amount_to_deduct = {
                if amount > f64::MAX as Balance {
                    return Err(Error::TransferAmountTooLarge);
                };
                amount as f64 * 0.03
            };
            let current_balance = balance
                .checked_sub(self.env().minimum_balance())
                .and_then(|res| res.checked_sub(amount_to_deduct.ceil() as Balance))
                .unwrap_or_default();
            if current_balance < self.amount_held {
                return Err(Error::InsufficientFunds);
            }
            if self.env().transfer(destination, amount).is_err() {
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds or if the transfer would have brought the\
                     contract's balance below minimum balance."
                )
            }
            self.amount_held += amount_to_deduct.ceil() as Balance;
            Ok(())
        }

        /// Withdraw all funds in the contract and terminate the contract.
        /// This returns an error when the expiration date has not reached
        #[ink(message)]
        pub fn withdraw_stored_funds(&mut self) -> Result<()> {
            //
            let now = self.env().block_timestamp();
            if now < self.expiration {
                return Err(Error::NotYetExpired);
            }
            self.env().terminate_contract(self.user);
        }

        /// Returns the `balance` of the contract.
        #[ink(message)]
        pub fn get_balance(&self) -> Balance {
            self.env().balance()
        }
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        //TODO
    }
}
