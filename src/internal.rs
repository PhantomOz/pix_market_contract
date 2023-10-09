use crate::*;
use near_sdk::CryptoHash;
use std::mem::size_of;

pub(crate) fn bytes_for_approved_account_id(account_id: &AccountId) -> u64 {
    account_id.as_str().len() as u64 + 4 + size_of::<u64>() as u64
}

pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

pub(crate) fn refund_deposit(storage_used: u64) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);

    let attached_deposit = env::attached_deposit();

    assert!(
        required_cost <= attached_deposit,
        "Not enough deposit to cover storage, {} yoctoNEAR required",
        required_cost,
    );

    let refund = attached_deposit - required_cost;

    if refund > 1 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}

impl Contract {
    pub(crate) fn internal_add_token_to_owner(
        &mut self,
        account_id: &AccountId,
        token_id: &TokenId,
    ) {
        let mut tokens_set = self.token_per_owner.get(account_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::TokenPerOwnerInner {
                    account_id_hash: hash_account_id(&account_id),
                }
                .try_to_vec()
                .unwrap(),
            )
        });
        tokens_set.insert(token_id);
        self.token_per_owner.insert(account_id, &tokens_set);
    }

    pub(crate) fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        token_id: &TokenId,
        memo: Option<String>,
    ) -> Token {
        let token = self.token_by_id.get(token_id).expect("No token");

        if sender_id != &token.owner_id {
            env::panic_str("Unauthorized")
        }
        assert_ne!(
            &token.owner_id, receiver_id,
            "The token is already owned by the receiver"
        );

        self.internal_remove_token_from_owner(&token.owner_id, token_id);
        self.internal_add_token_to_owner(receiver_id, token_id);

        let new_token = Token {
            owner_id: receiver_id.clone(),
            approved_account_ids: Default::default(),
            next_approval_id: 0,
        };

        self.token_by_id.insert(token_id, &new_token);

        if let Some(memo) = memo {
            env::log_str(&format!("memo: {}", memo).to_string());
        }

        token
    }

    pub(crate) fn internal_remove_token_from_owner(
        &mut self,
        account_id: &AccountId,
        token_id: &TokenId,
    ) {
        let mut tokens_set = self
            .token_per_owner
            .get(account_id)
            .expect("Token should be owned by the sender");
        tokens_set.remove(token_id);
        if tokens_set.is_empty() {
            self.token_per_owner.remove(account_id);
        } else {
            self.token_per_owner.insert(account_id, &tokens_set);
        }
    }
}
