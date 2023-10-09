use crate::*;

#[near_bindgen]
impl Contract {
    pub fn nft_total_supply(&self) -> U128 {
        todo!()
    }

    pub fn nft_tokens(&self, from_index: u128, limit: U128) -> Vec<JsonToken> {
        todo!()
    }

    pub fn nft_supply_for_owner(&self, account_id: AccountId) -> U128 {
        todo!()
    }

    pub fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: U128,
        limit: U128,
    ) -> Vec<JsonToken> {
        todo!()
    }
}
