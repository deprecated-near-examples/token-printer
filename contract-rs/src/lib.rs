use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, Promise};
use near_sdk::collections::Set;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

mod types;
use types::{U128, StrPublicKey};

pub type AccountId = String;
pub type Salt = u64;

/// A Transfer PoW Faucet contract that allows to request token transfer towards a given account.
/// It uses basic proof of work to avoid sybil attacks.
/// The new account always receives selected amount of tokens.
/// Proof of Work works the following way:
/// You need to compute a u64 salt (nonce) for a given account in such a way
/// that the `sha256(account_id + ':' + salt)` has more leading zero bits than
/// the required `min_difficulty`. The hash has to be unique in order to receive transfer.
/// One account can request multiple transfers.
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct TransferFaucet {
    /// Transfer amount
    pub transfer_amount: u128,
    /// Number of leading zeros in binary representation for a hash
    pub min_difficulty: u32,
    /// Created accounts
    pub existing_hashes: Set<Vec<u8>>,
}

impl Default for TransferFaucet {
    fn default() -> Self {
        panic!("Faucet is not initialized yet")
    }
}

/// Returns the number of leading zero bits for a given slice of bits.
fn num_leading_zeros(v: &[u8]) -> u32 {
    let mut res = 0;
    for z in v.iter().map(|b| b.leading_zeros()) {
        res += z;
        if z < 8 {
            break;
        }
    }
    res
}

fn assert_self() {
    assert_eq!(env::current_account_id(), env::predecessor_account_id(), "Can only be called by owner");
}

#[near_bindgen]
impl TransferFaucet {
    #[init]
    pub fn new(transfer_amount: U128, min_difficulty: u32) -> Self {
        assert!(env::state_read::<Self>().is_none(), "Already initialized");
        Self {
            transfer_amount: transfer_amount.into(),
            min_difficulty,
            existing_hashes: Set::new(b"h".to_vec())
        }
    }

    pub fn get_transfer_amount(&self) -> U128 { self.transfer_amount.into() }

    pub fn get_min_difficulty(&self) -> u32 {
        self.min_difficulty
    }

    pub fn get_num_transfers(&self) -> u64 {
        self.existing_hashes.len()
    }

    pub fn request_transfer(&mut self, account_id: AccountId, salt: Salt) -> Promise {
        // Checking proof of work
        //     Constructing a message for checking
        let mut message = account_id.as_bytes().to_vec();
        message.push(b':');
        message.extend_from_slice(&salt.to_le_bytes());
        //     Computing hash of the message
        let hash = env::sha256(&message);
        //     Checking that the resulting hash has enough leading zeros.
        assert!(num_leading_zeros(&hash) >= self.min_difficulty, "The proof is work is too weak");

        // Checking that the given hash is not used yet and remembering it.
        assert!(!self.existing_hashes.insert(&hash), "The given hash is already used for transfer");

        // Creating a transfer. It still can fail (e.g. account doesn't exists or the name is invalid),
        // but this contract will get the refund back.
        Promise::new(account_id)
            .transfer(self.transfer_amount)
            .into()
    }

    // Owner's methods. Can only be called by the owner

    pub fn set_min_difficulty(&mut self, min_difficulty: u32) {
        assert_self();
        self.min_difficulty = min_difficulty;
    }

    pub fn set_transfer_amount(&mut self, transfer_amount: U128) {
        assert_self();
        self.transfer_amount = transfer_amount.into();
    }

    pub fn add_access_key(&mut self, public_key: StrPublicKey) -> Promise {
        assert_self();
        Promise::new(env::current_account_id())
            .add_access_key(
                public_key.into(),
                0,
                env::current_account_id(),
                b"request_transfer".to_vec(),
            )
            .into()
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::{MockedBlockchain, testing_env, VMContext};
    use std::panic;
    use std::convert::TryFrom;

    use super::*;

    fn catch_unwind_silent<F: FnOnce() -> R + panic::UnwindSafe, R>(f: F) -> std::thread::Result<R> {
        let prev_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));
        let result = panic::catch_unwind(f);
        panic::set_hook(prev_hook);
        result
    }

    fn get_context() -> VMContext {
        VMContext {
            current_account_id: "alice".to_string(),
            signer_account_id: "bob".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "bob".to_string(),
            input: vec![],
            block_index: 0,
            block_timestamp: 0,
            account_balance: 10u128.pow(30),
            account_locked_balance: 0,
            storage_usage: 100,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(15),
            random_seed: vec![0, 1, 2],
            is_view: false,
            output_data_receivers: vec![],
        }
    }

    #[test]
    fn test_new() {
        let context = get_context();
        testing_env!(context);
        let transfer_amount = 100 * 10u128.pow(24);
        let min_difficulty = 5;
        let contract = TransferFaucet::new(transfer_amount.into(), min_difficulty);
        assert_eq!(contract.get_min_difficulty(), min_difficulty);
        assert_eq!(contract.get_transfer_amount().0, transfer_amount);
        assert_eq!(contract.get_num_transfers(), 0);
    }

    #[test]
    fn test_request_transfer_ok() {
        let context = get_context();
        testing_env!(context.clone());
        let transfer_amount = 100 * 10u128.pow(24);
        let min_difficulty = 5;
        let account_id = "test.alice";
        let mut contract = TransferFaucet::new(transfer_amount.into(), min_difficulty);
        let mut salt: u64 = 0;
        loop {
            // To avoid draining all gas
            testing_env!(context.clone());
            let mut message = account_id.as_bytes().to_vec();
            message.push(b':');
            message.extend_from_slice(&salt.to_le_bytes());
            //     Computing hash of the message
            let hash = env::sha256(&message);
            //     Checking that the resulting hash has enough leading zeros.
            if num_leading_zeros(&hash) >= min_difficulty {
                break;
            }
            salt += 1;
        };
        println!("Salt is {}", salt);
        contract.request_transfer(account_id.to_string(), salt);
        assert_eq!(contract.get_num_transfers(), 1);
    }

    #[test]
    fn test_fail_default() {
        let context = get_context();
        testing_env!(context);
        catch_unwind_silent(|| {
            TransferFaucet::default();
        }).unwrap_err();
    }

    #[test]
    fn test_fail_request_transfer_already_used() {
        let context = get_context();
        testing_env!(context);
        let transfer_amount = 100 * 10u128.pow(24);
        let min_difficulty = 5;
        let mut contract = TransferFaucet::new(transfer_amount.into(), min_difficulty);
        let account_id = "test.alice";
        let salt = 58;
        contract.request_transfer(account_id.to_string(), salt);
        catch_unwind_silent(move || {
            contract.request_transfer(account_id.to_string(), salt);
        }).unwrap_err();
    }


    #[test]
    fn test_num_leading_zeros() {
        assert_eq!(num_leading_zeros(&[0u8; 4]), 32);
        assert_eq!(num_leading_zeros(&[255u8; 4]), 0);
        assert_eq!(num_leading_zeros(&[254u8; 4]), 0);
        assert_eq!(num_leading_zeros(&[]), 0);
        assert_eq!(num_leading_zeros(&[127u8]), 1);
        assert_eq!(num_leading_zeros(&[0u8; 32]), 256);
        assert_eq!(num_leading_zeros(&[1u8; 4]), 7);
        assert_eq!(num_leading_zeros(&[0u8, 0u8, 255u8 >> 3]), 19);
        assert_eq!(num_leading_zeros(&[0u8, 0u8, 255u8 >> 3, 0u8]), 19);
    }

    #[test]
    fn test_add_access_key() {
        let mut context = get_context();
        context.predecessor_account_id = "alice".to_string();
        testing_env!(context);
        let transfer_amount = 100 * 10u128.pow(24);
        let min_difficulty = 5;
        let mut contract = TransferFaucet::new(transfer_amount.into(), min_difficulty);
        contract.add_access_key(StrPublicKey::try_from("ed25519:CFsEoaPizaj2uPP5StphygRTVugh1anqG8JpiGzpFHs").unwrap());
    }

    #[test]
    fn test_bad_public_key() {
        let mut context = get_context();
        context.predecessor_account_id = "alice".to_string();
        testing_env!(context);
        let transfer_amount = 100 * 10u128.pow(24);
        let min_difficulty = 5;
        let mut contract = TransferFaucet::new(transfer_amount.into(), min_difficulty);
        catch_unwind_silent(move || {
            contract.add_access_key(StrPublicKey::try_from("ed25519:CFsEoaPTVugh1anqG8JpiGzpFHs").unwrap());
        }).unwrap_err();
    }
}
