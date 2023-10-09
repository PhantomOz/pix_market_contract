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
