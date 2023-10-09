use crate::*;
use near_sdk::ext_contract;
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

#[near_bindgen]
impl NonFungibleTokenCore for Contract {
    fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        todo!()
    }
    fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) {
        todo!()
    }
    fn nft_token(&self, token_id: TokenId) -> Option<JsonToken> {
        if let Some(token) = self.token_by_id.get(&token_id) {
            let metadata = self.token_metadata_by_id.get(&token_id).unwrap();
            Some(JsonToken {
                token_id,
                owner_id: token.owner_id,
                metadata,
            })
        } else {
            None
        }
    }
}
