use crate::*;
use near_sdk::CryptoHash;
use std::mem::size_of;

pub(crate) fn royalty_to_payout(percentage: u32, amount_to_pay: Balance) -> U128 {
    U128((percentage as u128 * amount_to_pay) / 10_000u128)
}

pub(crate) fn bytes_for_approved_account_id(account_id: &AccountId) -> u64 {
    account_id.as_str().len() as u64 + 4 + size_of::<u64>() as u64
}

//Assert that the user has attached at least 1 yoctoNEAR (for security reasons and to pay for storage)
pub(crate) fn assert_at_least_one_yocto() {
    assert!(
        env::attached_deposit() >= 1,
        "Requires attached deposit of at least 1 yoctoNEAR",
    )
}

pub(crate) fn refund_approved_account_ids_iter<'a, I>(
    account_id: AccountId,
    approved_account_ids: I,
) -> Promise
where
    I: Iterator<Item = &'a AccountId>,
{
    let storage_released: u64 = approved_account_ids
        .map(bytes_for_approved_account_id)
        .sum();
    Promise::new(account_id).transfer(Balance::from(storage_released) * env::storage_byte_cost())
}

pub(crate) fn refund_approved_account_ids(
    account_id: AccountId,
    approved_account_ids: &HashMap<AccountId, u64>,
) -> Promise {
    refund_approved_account_ids_iter(account_id, approved_account_ids.keys())
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

    pub(crate) fn internal_add_series_to_owner(
        &mut self,
        account_id: &AccountId,
        series_id: &SeriesId,
    ) {
        let mut series_set = self.series_per_owner.get(account_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::SeriesPerOwnerInner {
                    account_id_hash: hash_account_id(&account_id),
                }
                .try_to_vec()
                .unwrap(),
            )
        });
        series_set.insert(series_id);
        self.series_per_owner.insert(account_id, &series_set);
    }

    pub(crate) fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        token_id: &TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) -> Token {
        let token = self.token_by_id.get(token_id).expect("No token");

        if sender_id != &token.owner_id {
            if !token.approved_account_ids.contains_key(sender_id) {
                env::panic_str("Unauthorized");
            }

            if let Some(enforced_approval_id) = approval_id {
                let actual_approval_id = token
                    .approved_account_ids
                    .get(sender_id)
                    .expect("Sender is not approved account");
                assert_eq!(
                    actual_approval_id, &enforced_approval_id,
                    "The actual approval_id {} is different from the given approval_id {}",
                    actual_approval_id, enforced_approval_id,
                );
            }
        }
        assert_ne!(
            &token.owner_id, receiver_id,
            "The token is already owned by the receiver"
        );

        self.internal_remove_token_from_owner(&token.owner_id, token_id);
        self.internal_add_token_to_owner(receiver_id, token_id);

        let new_token = Token {
            series_id: token.series_id,
            owner_id: receiver_id.clone(),
            approved_account_ids: Default::default(),
            next_approval_id: 0,
            royalty: token.royalty.clone(), //change this  when listing
        };

        self.token_by_id.insert(token_id, &new_token);

        if let Some(memo) = memo.as_ref() {
            env::log_str(&format!("memo: {}", memo).to_string());
        }

        // Default the authorized ID to be None for the logs.
        let mut authorized_id = None;
        //if the approval ID was provided, set the authorized ID equal to the sender
        if approval_id.is_some() {
            authorized_id = Some(sender_id.to_string());
        }

        // Construct the transfer log as per the events standard.
        let nft_transfer_log: EventLog = EventLog {
            // Standard name ("nep171").
            standard: NFT_STANDARD_NAME.to_string(),
            // Version of the standard ("nft-1.0.0").
            version: NFT_METADATA_SPEC.to_string(),
            // The data related with the event stored in a vector.
            event: EventLogVariant::NftTransfer(vec![NftTransferLog {
                // The optional authorized account ID to transfer the token on behalf of the old owner.
                authorized_id,
                // The old owner's account ID.
                old_owner_id: token.owner_id.to_string(),
                // The account ID of the new owner of the token.
                new_owner_id: receiver_id.to_string(),
                // A vector containing the token IDs as strings.
                token_ids: vec![token_id.to_string()],
                // An optional memo to include.
                memo,
            }]),
        };

        // Log the serialized json.
        env::log_str(&nft_transfer_log.to_string());

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

    pub(crate) fn internal_remove_sale(
        &mut self,
        nft_contract_id: AccountId,
        token_id: TokenId,
    ) -> Sale {
        //get the unique sale ID (contract + DELIMITER + token ID)
        let contract_and_token_id = format!("{}{}{}", &nft_contract_id, DELIMETER, token_id);
        //get the sale object by removing the unique sale ID. If there was no sale, panic
        let sale = self.sales.remove(&contract_and_token_id).expect("No sale");

        //get the set of sales for the sale's owner. If there's no sale, panic.
        let mut by_owner_id = self
            .sale_by_owner
            .get(&sale.owner_id)
            .expect("No sale by_owner_id");
        //remove the unique sale ID from the set of sales
        by_owner_id.remove(&contract_and_token_id);

        //if the set of sales is now empty after removing the unique sale ID, we simply remove that owner from the map
        if by_owner_id.is_empty() {
            self.sale_by_owner.remove(&sale.owner_id);
        //if the set of sales is not empty after removing, we insert the set back into the map for the owner
        } else {
            self.sale_by_owner.insert(&sale.owner_id, &by_owner_id);
        }

        //get the set of token IDs for sale for the nft contract ID. If there's no sale, panic.
        let mut by_nft_contract_id = self
            .by_nft_contract_id
            .get(&nft_contract_id)
            .expect("No sale by nft_contract_id");

        //remove the token ID from the set
        by_nft_contract_id.remove(&token_id);

        //if the set is now empty after removing the token ID, we remove that nft contract ID from the map
        if by_nft_contract_id.is_empty() {
            self.by_nft_contract_id.remove(&nft_contract_id);
        //if the set is not empty after removing, we insert the set back into the map for the nft contract ID
        } else {
            self.by_nft_contract_id
                .insert(&nft_contract_id, &by_nft_contract_id);
        }

        //return the sale object
        sale
    }
}
