use crate::{
    error::Result,
    storage::{Mapping, Slot, thread_local::AddressGuard},
};
use alloy::primitives::{Address, U256};

pub struct ThreadLocalToken {
    total_supply: Slot<U256>,
    balances: Mapping<Address, U256>,
}

impl ThreadLocalToken {
    // macro generated
    pub fn _new() -> Self {
        Self {
            total_supply: Slot::new(U256::ZERO),
            balances: Mapping::new(U256::ONE),
        }
    }

    // macro generated
    pub fn staticcall<F, R>(address: Address, f: F) -> Result<R>
    where
        F: for<'b> FnOnce(&Self) -> Result<R>,
    {
        let _guard = AddressGuard::new(address)?;
        let instance = Self::_new();
        f(&instance)
    }

    // macro generated
    pub fn call_mut<F, R>(address: Address, f: F) -> Result<R>
    where
        F: for<'b> FnOnce(&mut Self) -> Result<R>,
    {
        let _guard = AddressGuard::new(address)?;
        let mut instance = Self::_new();
        f(&mut instance)
    }

    pub fn total_supply(&self) -> Result<U256> {
        self.total_supply.read_tl()
    }

    fn set_total_supply(&mut self, value: U256) -> Result<()> {
        self.total_supply.write_tl(value)
    }

    pub fn balance_of(&self, account: Address) -> Result<U256> {
        self.balances.at(account).read_tl()
    }

    fn set_balance(&mut self, account: Address, balance: U256) -> Result<()> {
        self.balances.at(account).write_tl(balance)
    }

    pub fn mint(&mut self, to: Address, amount: U256) -> Result<()> {
        let balance = self.balance_of(to)?;
        let supply = self.total_supply()?;

        self.set_balance(to, balance + amount)?;
        self.set_total_supply(supply + amount)?;

        Ok(())
    }

    pub fn transfer(&mut self, from: Address, to: Address, amount: U256) -> Result<()> {
        let from_balance = self.balance_of(from)?;
        let to_balance = self.balance_of(to)?;

        self.set_balance(from, from_balance - amount)?;
        self.set_balance(to, to_balance + amount)?;

        Ok(())
    }

    pub fn transfer_with_rewards(
        &mut self,
        from: Address,
        to: Address,
        amount: U256,
    ) -> Result<()> {
        self.transfer(from, to, amount)?;

        ThreadLocalRewards::call_mut(REWARDS_ADDRESS, |rewards| rewards.distribute(amount))?;

        Ok(())
    }
}

const REWARDS_ADDRESS: Address = Address::new([0xEE; 20]);

pub struct ThreadLocalRewards {
    rewards_pool: Slot<U256>,
}

impl ThreadLocalRewards {
    // macro generated
    fn _new() -> Self {
        Self {
            rewards_pool: Slot::new(U256::ZERO),
        }
    }

    // macro generated
    pub fn staticcall<F, R>(address: Address, f: F) -> Result<R>
    where
        F: for<'b> FnOnce(&Self) -> Result<R>,
    {
        let _guard = AddressGuard::new(address)?;
        let instance = Self::_new();
        f(&instance)
    }

    // macro generated
    pub fn call_mut<F, R>(address: Address, f: F) -> Result<R>
    where
        F: for<'b> FnOnce(&mut Self) -> Result<R>,
    {
        let _guard = AddressGuard::new(address)?;
        let mut instance = Self::_new();
        f(&mut instance)
    }

    pub fn distribute(&mut self, transfer_amount: U256) -> Result<()> {
        let reward = transfer_amount / U256::from(100);
        let pool = self.rewards_pool.read_tl()?;
        self.rewards_pool.write_tl(pool + reward)?;

        Ok(())
    }

    pub fn get_pool(&self) -> Result<U256> {
        self.rewards_pool.read_tl()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{hashmap::HashMapStorageProvider, thread_local::StorageGuard};

    #[test]
    fn test_pure_thread_local() -> Result<()> {
        let mut storage = HashMapStorageProvider::new(1);
        let _storage_guard = unsafe { StorageGuard::new(&mut storage) };

        let token_address = Address::new([0x01; 20]);
        let alice = Address::new([0xA1; 20]);
        let bob = Address::new([0xB0; 20]);

        ThreadLocalToken::call_mut(token_address, |token| {
            // mint
            token.mint(alice, U256::from(1000))?;
            assert_eq!(token.balance_of(alice)?, U256::from(1000));
            assert_eq!(token.total_supply()?, U256::from(1000));

            // transfer
            token.transfer(alice, bob, U256::from(100))?;
            assert_eq!(token.balance_of(alice)?, U256::from(900));
            assert_eq!(token.balance_of(bob)?, U256::from(100));

            Ok(())
        })
    }

    #[test]
    fn test_cross_contract_calls() -> Result<()> {
        let mut storage = HashMapStorageProvider::new(1);
        let _storage_guard = unsafe { StorageGuard::new(&mut storage) };

        let token_address = Address::new([0x01; 20]);
        let alice = Address::new([0xA1; 20]);
        let bob = Address::new([0xB0; 20]);

        ThreadLocalToken::call_mut(token_address, |token| {
            token.mint(alice, U256::from(1000))?;

            // transfer with rewards - demonstrates scoped cross-contract call
            token.transfer_with_rewards(alice, bob, U256::from(100))?;
            assert_eq!(token.balance_of(alice)?, U256::from(900));
            assert_eq!(token.balance_of(bob)?, U256::from(100));

            Ok(())
        })?;

        // verify rewards were distributed
        ThreadLocalRewards::staticcall(REWARDS_ADDRESS, |rewards| {
            let pool = rewards.get_pool()?;
            assert_eq!(pool, U256::from(1));
            Ok(())
        })
    }

    #[test]
    fn test_nested_call_depth() -> Result<()> {
        use crate::storage::thread_local::context;

        let mut storage = HashMapStorageProvider::new(1);
        let addr1 = Address::new([0x01; 20]);
        let addr2 = Address::new([0x02; 20]);
        let addr3 = Address::new([0x03; 20]);

        let _storage_guard = unsafe { StorageGuard::new(&mut storage) };

        // demonstrate nested contract calls with automatic address stack management
        ThreadLocalToken::staticcall(addr1, |_token1| {
            assert_eq!(context::call_depth(), 1);

            ThreadLocalToken::staticcall(addr2, |_token2| {
                assert_eq!(context::call_depth(), 2);

                ThreadLocalToken::staticcall(addr3, |_token3| {
                    assert_eq!(context::call_depth(), 3);
                    Ok(())
                })?;

                assert_eq!(context::call_depth(), 2);
                Ok(())
            })?;

            assert_eq!(context::call_depth(), 1);
            Ok(())
        })
    }
}
