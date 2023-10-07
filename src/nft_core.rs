pub trait NonFungibleTokenCore {
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    );
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    );
    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken>;
}

#[ext_contract(ext_self)]
trait NonFungibleTokenCoreSelf {
    fn nft_resolve_transfer(
        &mut self,
        owner_id: AccountId,
        token_id: TokenId,
        receiver_id: AccountId,
    ) -> bool;
}
