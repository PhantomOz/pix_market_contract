impl Contract {
    pub fn nft_total_supply(&self) -> u128;

    pub fn nft_tokens(&self, from_index: u128, limit: u128) -> Vec<JsonToken>;

    pub fn nft_token(&self, token_id: TokenId) -> Option<JsonToken>;

    pub fn nft_supply_for_owner(&self, account_id: AccountId) -> u128;

    pub fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: u128,
        limit: u128,
    ) -> Vec<JsonToken>;
}
