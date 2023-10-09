use std::collections::HashMap;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet},
    env,
    json_types::{Base64VecU8, U128},
    near_bindgen,
    serde::{Deserialize, Serialize},
    AccountId, Balance, BorshStorageKey, CryptoHash, PanicOnDefault, Promise, PromiseOrValue,
};

pub use crate::approval::*;
use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::royalties::*;

mod approval;
mod enumeration;
mod internal;
mod metadata;
mod mint;
mod nft_core;
mod royalties;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub token_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,
    pub token_by_id: LookupMap<TokenId, Token>,
    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,
    pub metadata: LazyOption<NFTContractMetadata>,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    TokenPerOwner,
    TokenById,
    TokenMetadataById,
    NFTContractMetadata,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new_default(owner_id: AccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: "Pixicle_1.0".to_string(),
                name: "Pixicle".to_string(),
                symbol: "PIX".to_string(),
                icon: None,
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata) -> Self {
        Self {
            owner_id,
            token_per_owner: LookupMap::new(StorageKey::TokenPerOwner),
            token_by_id: LookupMap::new(StorageKey::TokenById),
            token_metadata_by_id: UnorderedMap::new(StorageKey::TokenMetadataById),
            metadata: LazyOption::new(
                StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
                Some(&metadata),
            ),
        }
    }
}
