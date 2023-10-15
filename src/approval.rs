use crate::*;
use near_sdk::{assert_one_yocto, ext_contract};
pub trait NonFungibleTokenCore {
    fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>);
    fn nft_is_approved(
        &self,
        token_id: TokenId,
        account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool;
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId);
    fn nft_revoke_all(&mut self, token_id: TokenId);
}

#[ext_contract(ext_non_fungible_approval_receiver)]
trait NonFungibleApprovalReceiver {
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
}

#[ext_contract(ext_contract)]
trait ExtContract {
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId, //purchaser (person to transfer the NFT to)
        token_id: TokenId,      //token ID to transfer
        approval_id: u64, //market contract's approval ID in order to transfer the token on behalf of the owner
        memo: String,     //memo (to include some context)
        /*
            the price that the token was purchased for. This will be used in conjunction with the royalty percentages
            for the token in order to determine how much money should go to which account.
        */
        balance: U128,
        //the maximum amount of accounts the market can payout at once (this is limited by GAS)
        max_len_payout: u32,
    );
}

#[near_bindgen]
impl NonFungibleTokenCore for Contract {
    #[payable]
    fn nft_approve(&mut self, token_id: TokenId, owner_id: AccountId, msg: Option<String>) {
        assert_one_yocto();
        let mut token = self.token_by_id.get(&token_id).expect("No token");

        assert_eq!(
            &env::predecessor_account_id(),
            &token.owner_id,
            "You are not the token Owner."
        );

        let approval_id: u64 = token.next_approval_id;

        let is_new_approval = token
            .approved_account_ids
            .insert(owner_id.clone(), approval_id)
            .is_none();

        let storage_used = if is_new_approval {
            bytes_for_approved_account_id(&owner_id)
        } else {
            0
        };

        token.next_approval_id += 1;
        self.token_by_id.insert(&token_id, &token);

        refund_deposit(storage_used);

        if let Some(msg) = msg {
            ext_non_fungible_approval_receiver::ext(owner_id)
                .nft_on_approve(token_id, token.owner_id, approval_id, msg)
                .as_return();
        }
    }

    fn nft_is_approved(
        &self,
        token_id: TokenId,
        account_id: AccountId,
        approval_id: Option<u64>,
    ) -> bool {
        let token = self.token_by_id.get(&token_id).expect("No token");

        let approval = token.approved_account_ids.get(&account_id);

        if let Some(approval) = approval {
            if let Some(approval_id) = approval_id {
                approval_id == *approval
            } else {
                true
            }
        } else {
            false
        }
    }

    #[payable]
    fn nft_revoke(&mut self, token_id: TokenId, account_id: AccountId) {
        assert_one_yocto();
        let mut token = self.token_by_id.get(&token_id).expect("No token");

        let predecessor_account_id = env::predecessor_account_id();
        assert_eq!(&predecessor_account_id, &token.owner_id);

        if token.approved_account_ids.remove(&account_id).is_some() {
            refund_approved_account_ids_iter(predecessor_account_id, [account_id].iter());

            self.token_by_id.insert(&token_id, &token);
        }
    }

    #[payable]
    fn nft_revoke_all(&mut self, token_id: TokenId) {
        assert_one_yocto();

        let mut token = self.token_by_id.get(&token_id).expect("No token");
        let predecessor_account_id = env::predecessor_account_id();
        assert_eq!(&predecessor_account_id, &token.owner_id);

        if !token.approved_account_ids.is_empty() {
            refund_approved_account_ids(predecessor_account_id, &token.approved_account_ids);
            token.approved_account_ids.clear();
            self.token_by_id.insert(&token_id, &token);
        }
    }
}

#[near_bindgen]
impl NonFungibleApprovalReceiver for Contract {
    /// where we add the sale because we know nft owner can only call nft_approve

    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    ) {
        // get the contract ID which is the predecessor
        let nft_contract_id = env::predecessor_account_id();
        //get the signer which is the person who initiated the transaction
        let signer_id = env::signer_account_id();

        //make sure that the signer isn't the predecessor. This is so that we're sure
        //this was called via a cross-contract call
        assert_ne!(
            nft_contract_id, signer_id,
            "nft_on_approve should only be called via cross-contract call"
        );
        //make sure the owner ID is the signer.
        assert_eq!(owner_id, signer_id, "owner_id should be signer_id");

        //we need to enforce that the user has enough storage for 1 EXTRA sale.

        //get the storage for a sale. dot 0 converts from U128 to u128
        let storage_amount = self.storage_minimum_balance().0;
        //get the total storage paid by the owner
        let owner_paid_storage = self.storage_deposits.get(&signer_id).unwrap_or(0);
        //get the storage required which is simply the storage for the number of sales they have + 1
        let signer_storage_required =
            (self.get_supply_by_owner_id(signer_id).0 + 1) as u128 * storage_amount;

        //make sure that the total paid is >= the required storage
        assert!(
            owner_paid_storage >= signer_storage_required,
            "Insufficient storage paid: {}, for {} sales at {} rate of per sale",
            owner_paid_storage,
            signer_storage_required / STORAGE_PER_SALE,
            STORAGE_PER_SALE
        );

        //if all these checks pass we can create the sale conditions object.
        let SaleArgs { sale_conditions } =
            //the sale conditions come from the msg field. The market assumes that the user passed
            //in a proper msg. If they didn't, it panics. 
            near_sdk::serde_json::from_str(&msg).expect("Not valid SaleArgs");

        //create the unique sale ID which is the contract + DELIMITER + token ID
        let contract_and_token_id = format!("{}{}{}", nft_contract_id, DELIMETER, token_id);

        //insert the key value pair into the sales map. Key is the unique ID. value is the sale object
        self.sales.insert(
            &contract_and_token_id,
            &Sale {
                owner_id: owner_id.clone(),                   //owner of the sale / token
                approval_id, //approval ID for that token that was given to the market
                nft_contract_id: nft_contract_id.to_string(), //NFT contract the token was minted on
                token_id: token_id.clone(), //the actual token ID
                sale_conditions, //the sale conditions
            },
        );

        //Extra functionality that populates collections necessary for the view calls

        //get the sales by owner ID for the given owner. If there are none, we create a new empty set
        let mut by_owner_id = self.sale_by_owner.get(&owner_id).unwrap_or_else(|| {
            UnorderedSet::new(
                StorageKey::SaleByOwnerIdInner {
                    //we get a new unique prefix for the collection by hashing the owner
                    account_id_hash: hash_account_id(&owner_id),
                }
                .try_to_vec()
                .unwrap(),
            )
        });

        //insert the unique sale ID into the set
        by_owner_id.insert(&contract_and_token_id);
        //insert that set back into the collection for the owner
        self.sale_by_owner.insert(&owner_id, &by_owner_id);

        //get the token IDs for the given nft contract ID. If there are none, we create a new empty set
        let mut by_nft_contract_id = self
            .by_nft_contract_id
            .get(&nft_contract_id)
            .unwrap_or_else(|| {
                UnorderedSet::new(
                    StorageKey::ByNFTContractIdInner {
                        //we get a new unique prefix for the collection by hashing the owner
                        account_id_hash: hash_account_id(&nft_contract_id),
                    }
                    .try_to_vec()
                    .unwrap(),
                )
            });

        //insert the token ID into the set
        by_nft_contract_id.insert(&token_id);
        //insert the set back into the collection for the given nft contract ID
        self.by_nft_contract_id
            .insert(&nft_contract_id, &by_nft_contract_id);
    }
}
