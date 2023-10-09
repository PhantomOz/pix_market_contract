use crate::*;
use near_sdk::ext_contract;
pub trait NonFungibleTokenCore {
    fn nft_approve(&mut self, token_id: TokenId, account_id: AccountId, msg: Option<String>);
    fn nft_is_approved(&self, token_id: TokenId, account_id: AccountId) -> bool;
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
