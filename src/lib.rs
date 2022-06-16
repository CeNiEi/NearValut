use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LookupMap, LookupSet},
    env, near_bindgen, AccountId, Balance, BorshStorageKey, PanicOnDefault, Promise, Timestamp,
};

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Pools,
    PoolParticipants { pool_hash: Vec<u8> },
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    admin: AccountId,
    // the key here can be a unique identifier like UUID (string for now)
    pools: LookupMap<String, Pool>,
}

// a deadline after which money gets locked in the account?
#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Pool {
    creator: AccountId,
    participants: LookupSet<AccountId>,
    max_num_of_participants: u32,
    current_num_of_participants: u32,
    created_at: Timestamp,
    //per-person
    amount: Balance,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(admin: AccountId) -> Self {
        Self {
            admin,
            pools: LookupMap::new(StorageKeys::Pools),
        }
    }

    pub fn new_pool(&mut self, key: String, amount: Balance, max_num_of_participants: u32) {
        assert_eq!(
            env::predecessor_account_id(),
            self.admin,
            "Caller must be the Admin"
        );

        self.pools.insert(
            &key,
            &Pool {
                creator: self.admin.clone(),
                participants: LookupSet::new(StorageKeys::PoolParticipants {
                    pool_hash: env::sha256(key.as_bytes()),
                }),
                current_num_of_participants: 0,
                max_num_of_participants,
                created_at: env::block_timestamp(),
                amount,
            },
        );
    }

    #[payable]
    pub fn join_pool(&mut self, key: String) {
        let pool = &mut self.pools.get(&key).expect("Invalid key");
        assert!(
            pool.current_num_of_participants < pool.max_num_of_participants,
            "Pool limit reached"
        );

        assert!(
            env::attached_deposit() == pool.amount,
            "Incorrect amount received"
        );

        pool.participants.insert(&env::predecessor_account_id());
        pool.current_num_of_participants += 1;
    }

    pub fn leave_pool(&mut self, key: String) {
        let pool = &mut self.pools.get(&key).expect("Invalid Key");
        
        let pres = pool.participants.remove(&env::predecessor_account_id());
        assert!(pres, "Account not in pool");

        pool.current_num_of_participants -= 1;
    }

    pub fn reward_pool_winners(&mut self, key: String, winners: &[AccountId]) {
        let pool = &mut self.pools.get(&key).expect("Invalid key");
        assert_eq!(
            env::predecessor_account_id(),
            pool.creator,
            "Caller must be the Creator of the pool"
        );

        assert!(
            winners.len() <= pool.current_num_of_participants as usize,
            "Too many winners"
        );

        let winning_amount = pool
            .amount
            .checked_div(winners.len() as u128)
            .expect("No Winners given")
            .checked_mul(pool.current_num_of_participants as u128)
            .unwrap();

        winners.iter().for_each(|account| {
            Promise::new(account.clone()).transfer(winning_amount);
        });

        self.pools.remove(&key);
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    fn 
}
