#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod doublesig {

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    pub struct Doublesig {
        /// Stores a single `bool` value on the storage.
        value: bool,
    }

    impl Doublesig {
        /// Instantiate contract by:
        /// 1. Specifying the escrow address
        /// 2. Specifying the expiration of the contract
        /// Note: Percentage deducted from each transaction is currently set to 3%
        #[ink(constructor)]
        pub fn new(escrow: AccountId,expiration: Timestamp) -> Self {
            todo!()
        }

        /// Transfer `amount` to specified destination
        #[ink(message)]
        pub fn transfer_funds(&mut self, destination: AccountId, amount: Balance) {
            todo!()
        }
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        //TODO
    }
}
