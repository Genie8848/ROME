//! A smart contract that allows users to generate a double signature for the
//! distribution of funds every time they sign a transaction.
//!
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
//!

#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod doublesig {

    #[ink(storage)]
    pub struct DoubleSig {
        user: AccountId,
        expiration: Timestamp,
        amount_held: Balance,
    }

    /// Errors that can occur upon calling this contract.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo))]
    pub enum Error {
        NotYetExpired,
        /// Returned if caller is not the `sender` while required to.
        CallerIsNotOwner,
        InsufficientFunds {
            total_balance: Balance,
            /// refers to the contract amount left after potential deduction
            potential_balance: Balance,
            funds_to_transfer: Balance,
            existential_deposit: Balance,
        },
        TransferAmountTooLarge,
        WithdrawalFailed,
    }

    /// Type alias for the contract's `Result` type.
    pub type Result<T> = core::result::Result<T, Error>;

    const FEE: f64 = 0.03; // 3%

    impl DoubleSig {
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

        /// Transfer `amount` to specified `destination`. An additional 3% of the transaction
        /// would be deducted and stored.
        ///
        /// # Errors
        ///
        /// - Panics in case the requested transfer exceeds the contract balance.
        /// - Panics in case the requested transfer would have brought this
        ///   contract's balance below the minimum balance (i.e. the chain's
        ///   existential deposit).
        /// - Panics in case the transfer failed for another reason.
        #[ink(message)]
        pub fn transfer_funds(&mut self, destination: AccountId, amount: Balance) -> Result<()> {
            // ensure the amount held is greater than the contract's balance
            let balance = self.get_balance();
            // since fractions aren't supported, use the `ceil` value
            // Therefore the minimum fee is 1 unit
            let amount_to_deduct = {
                if amount > f64::MAX as Balance {
                    return Err(Error::TransferAmountTooLarge);
                };
                (amount as f64 * FEE).ceil() as Balance
            };
            let current_balance = balance
                .checked_sub(self.env().minimum_balance())
                .and_then(|res| res.checked_sub(amount_to_deduct))
                .unwrap_or_default();
            if current_balance <= amount {
                return Err(Error::InsufficientFunds {
                    total_balance: self.get_balance(),
                    funds_to_transfer: amount,
                    potential_balance: current_balance,
                    existential_deposit: self.env().minimum_balance(),
                });
            }
            if self.env().transfer(destination, amount).is_err() {
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds or if the transfer would have brought the\
                     contract's balance below minimum balance."
                )
            }
            self.amount_held += amount_to_deduct;
            Ok(())
        }

        /// Withdraw all funds in the contract and terminate the contract.
        /// This returns an error when the expiration date has not reached
        #[ink(message)]
        pub fn claim_funds(&mut self) -> Result<()> {
            if self.env().caller() != self.user {
                return Err(Error::CallerIsNotOwner);
            }
            let now = self.env().block_timestamp();
            if now < self.expiration {
                return Err(Error::NotYetExpired);
            }
            self.env().terminate_contract(self.user);
        }

        /// Transfer all savings to `senders` account
        ///
        /// # Errors
        /// Ideally this method doesn't panic. Please report any panics
        pub fn withdraw_savings(&mut self) -> Result<()> {
            if self.env().caller() != self.user {
                return Err(Error::CallerIsNotOwner);
            }
            let now = self.env().block_timestamp();
            if now < self.expiration {
                return Err(Error::NotYetExpired);
            }
            let remainder = self
                .get_balance()
                .checked_sub(self.amount_held)
                .unwrap_or_default();
            if remainder < self.env().minimum_balance() {
                ink::env::debug_println!(
                    "Balance would fall below existential deposit. \
                    Terminate contract to withdraw all funds"
                );
                return Err(Error::WithdrawalFailed);
            }
            if self
                .env()
                .transfer(self.env().caller(), self.amount_held)
                .map(|_| self.amount_held = 0)
                .is_err()
            {
                panic!(
                    "requested transfer failed. this can be the case if the contract does not\
                     have sufficient free funds or if the transfer would have brought the\
                     contract's balance below minimum balance."
                )
            }
            Ok(())
        }

        /// Get the current spendable amount (`free balance`)
        #[ink(message)]
        pub fn free(&self) -> Balance {
            self.get_balance()
                .checked_sub(self.env().minimum_balance())
                .and_then(|res| res.checked_sub(self.amount_held))
                .unwrap_or_default()
        }

        /// Get the total value of the funds which has been saved
        #[ink(message)]
        pub fn amount_stored(&self) -> Balance {
            self.amount_held
        }

        /// Returns the total `balance` of the contract.
        #[ink(message)]
        pub fn get_balance(&self) -> Balance {
            self.env().balance()
        }

        /// Returns the `expiration` of the contract.
        #[ink(message)]
        pub fn get_expiration(&self) -> Timestamp {
            self.expiration
        }
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        fn default_accounts() -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
            ink::env::test::default_accounts::<ink::env::DefaultEnvironment>()
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(account_id, balance)
        }

        fn set_sender(sender: AccountId) {
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(sender);
        }

        fn get_balance(account_id: AccountId) -> Balance {
            ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(account_id)
                .expect("Cannot get account balance")
        }

        fn contract_id() -> AccountId {
            ink::env::test::callee::<ink::env::DefaultEnvironment>()
        }

        fn create_contract(initial_balance: Balance) -> DoubleSig {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(contract_id(), initial_balance);
            DoubleSig::new(1000)
        }

        fn advance_block() {
            ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        }

        #[ink::test]
        fn test_transfer_works() {
            let contract_balance = 100_000_000;
            let accounts = default_accounts();
            let mut smart_contract = create_contract(contract_balance);
            set_sender(accounts.eve);
            set_balance(accounts.eve, 0);
            smart_contract
                .transfer_funds(accounts.eve, 2_000_000)
                .unwrap();
            assert_eq!(get_balance(accounts.eve), 2_000_000);
        }

        #[ink::test]
        fn test_deduction_works() {
            let contract_balance = 100_000_000;
            let accounts = default_accounts();
            let mut smart_contract = create_contract(contract_balance);
            // send initial funds
            set_sender(accounts.eve);
            set_balance(accounts.eve, 0);
            smart_contract
                .transfer_funds(accounts.eve, 2_000_000)
                .unwrap();
            assert_eq!(get_balance(accounts.eve), 2_000_000);
            assert_eq!(smart_contract.amount_stored(), 60_000); // 3% of transaction (2 million)
            assert_eq!(smart_contract.get_balance(), 98_000_000);

            // send larger funds
            set_sender(accounts.bob);
            set_balance(accounts.bob, 0);
            smart_contract
                .transfer_funds(accounts.bob, 90_000_000)
                .unwrap();
            assert_eq!(get_balance(accounts.bob), 90_000_000);
            assert_eq!(smart_contract.amount_stored(), 2_760_000); //2.7 mil + 60k (initial)
            assert_eq!(smart_contract.get_balance(), 8_000_000);
        }

        #[ink::test]
        fn test_deduction_overflow_works() {
            let contract_balance = 100_000_000;
            let accounts = default_accounts();
            let mut smart_contract = create_contract(contract_balance);
            // send initial large funds
            set_sender(accounts.eve);
            set_balance(accounts.eve, 0);
            let transaction = smart_contract.transfer_funds(accounts.eve, 98_000_000);

            assert_eq!(
                transaction.unwrap_err(),
                Error::InsufficientFunds {
                    existential_deposit: 1_000_000,
                    total_balance: 100_000_000,
                    potential_balance: 96_060_000,
                    funds_to_transfer: 98_000_000
                }
            );
        }

        #[ink::test]
        fn test_claim_funds() {
            let contract_balance = 100_000_000;
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(contract_id(), contract_balance);
            let expiration = 1;
            let mut smart_contract = DoubleSig::new(expiration);
            smart_contract
                .transfer_funds(accounts.bob, 9_000_000)
                .unwrap();
            assert_eq!(smart_contract.amount_stored(), 270_000); //3% of 9 mill
            let total_balance_left = 91_000_000;
            advance_block();
            let should_close = move || smart_contract.claim_funds().unwrap();
            ink::env::test::assert_contract_termination::<ink::env::DefaultEnvironment, _>(
                should_close,
                accounts.alice,
                total_balance_left,
            );
        }
    }
}
