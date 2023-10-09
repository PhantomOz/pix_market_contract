use crate::*;
pub trait NonFungibleTokenMetadata {
    fn nft_payout(
        &self,
        token_id: TokenId,
        account_id: AccountId,
        balance: u128,
        max_len_payout: u32,
    ) -> Payout;
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        balance: u128,
        max_len_payout: u32,
    ) -> Payout;
}
