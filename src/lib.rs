use std::collections::HashMap;

use near_sdk::{
    assert_one_yocto,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet},
    env::{self, STORAGE_PRICE_PER_BYTE},
    json_types::{Base64VecU8, U128},
    near_bindgen, require,
    serde::{Deserialize, Serialize},
    AccountId, Balance, BorshStorageKey, CryptoHash, Gas, PanicOnDefault, Promise, PromiseOrValue,
};

pub use crate::approval::*;
pub use crate::events::*;
use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::royalties::*;
pub use crate::sale::*;
pub use crate::sale_views::*;
pub use crate::series::*;
pub use crate::user::*;

mod approval;
mod enumeration;
mod events;
mod internal;
mod metadata;
mod mint;
mod nft_core;
mod royalties;
mod sale;
mod sale_views;
mod series;
mod user;

pub const NFT_METADATA_SPEC: &str = "1.0.0";
pub const NFT_STANDARD_NAME: &str = "nep171";
//the minimum storage to have a sale on the contract.
const STORAGE_PER_SALE: u128 = 1000 * STORAGE_PRICE_PER_BYTE;

//every sale will have a unique ID which is `CONTRACT + DELIMITER + TOKEN_ID`
static DELIMETER: &str = ".";
const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(10_000_000_000_000);
const GAS_FOR_NFT_ON_TRANSFER: Gas = Gas(25_000_000_000_000);

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub owner_id: AccountId,
    pub token_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,
    pub token_by_id: LookupMap<TokenId, Token>,
    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,
    pub user_metadata_by_owner: UnorderedMap<AccountId, UserMetadata>,
    pub metadata: LazyOption<NFTContractMetadata>,
    pub series_by_id: UnorderedMap<SeriesId, Series>,
    pub series_per_owner: LookupMap<AccountId, UnorderedSet<SeriesId>>,
    pub storage_deposits: LookupMap<AccountId, Balance>,
    pub sale_by_owner: UnorderedMap<AccountId, UnorderedSet<TokenId>>,
    pub sales: UnorderedMap<TokenId, Sale>,
    pub by_nft_contract_id: LookupMap<AccountId, UnorderedSet<TokenId>>,
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKey {
    TokenPerOwner,
    TokenById,
    TokenMetadataById,
    UserMetadataByOwner,
    UserMetadataByOwnerInner { account_id_hash: CryptoHash },
    NFTContractMetadata,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    SeriesByIdInner { account_id_hash: CryptoHash },
    SeriesById,
    SeriesPerOwner,
    SeriesPerOwnerInner { account_id_hash: CryptoHash },
    StorageDeposits,
    SaleByOwner,
    Sales,
    ByNFTContractId,
    SaleByOwnerIdInner { account_id_hash: CryptoHash },
    ByNFTContractIdInner { account_id_hash: CryptoHash },
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
            user_metadata_by_owner: UnorderedMap::new(StorageKey::UserMetadataByOwner),
            series_by_id: UnorderedMap::new(StorageKey::SeriesById),
            series_per_owner: LookupMap::new(StorageKey::SeriesPerOwner),
            metadata: LazyOption::new(
                StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
                Some(&metadata),
            ),
            storage_deposits: LookupMap::new(StorageKey::StorageDeposits),
            sale_by_owner: UnorderedMap::new(StorageKey::SaleByOwner),
            sales: UnorderedMap::new(StorageKey::Sales),
            by_nft_contract_id: LookupMap::new(StorageKey::ByNFTContractId),
        }
    }

    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<AccountId>) {
        //get the account ID to pay for storage for
        let storage_account_id = account_id
            //convert the valid account ID into an account ID
            .map(|a| a.into())
            //if we didn't specify an account ID, we simply use the caller of the function
            .unwrap_or_else(env::predecessor_account_id);

        //get the deposit value which is how much the user wants to add to their storage
        let deposit = env::attached_deposit();

        //make sure the deposit is greater than or equal to the minimum storage for a sale
        assert!(
            deposit >= STORAGE_PER_SALE,
            "Requires minimum deposit of {}",
            STORAGE_PER_SALE
        );

        //get the balance of the account (if the account isn't in the map we default to a balance of 0)
        let mut balance: u128 = self.storage_deposits.get(&storage_account_id).unwrap_or(0);
        //add the deposit to their balance
        balance += deposit;
        //insert the balance back into the map for that account ID
        self.storage_deposits.insert(&storage_account_id, &balance);
    }

    //Allows users to withdraw any excess storage that they're not using. Say Bob pays 0.01N for 1 sale
    //Alice then buys Bob's token. This means bob has paid 0.01N for a sale that's no longer on the marketplace
    //Bob could then withdraw this 0.01N back into his account.
    #[payable]
    pub fn storage_withdraw(&mut self) {
        //make sure the user attaches exactly 1 yoctoNEAR for security purposes.
        //this will redirect them to the NEAR wallet (or requires a full access key).
        assert_one_yocto();

        //the account to withdraw storage to is always the function caller
        let owner_id = env::predecessor_account_id();
        //get the amount that the user has by removing them from the map. If they're not in the map, default to 0
        let mut amount = self.storage_deposits.remove(&owner_id).unwrap_or(0);

        //how many sales is that user taking up currently. This returns a set
        let sales = self.sale_by_owner.get(&owner_id);
        //get the length of that set.
        let len = sales.map(|s| s.len()).unwrap_or_default();
        //how much NEAR is being used up for all the current sales on the account
        let diff = u128::from(len) * STORAGE_PER_SALE;

        //the excess to withdraw is the total storage paid - storage being used up.
        amount -= diff;

        //if that excess to withdraw is > 0, we transfer the amount to the user.
        if amount > 0 {
            Promise::new(owner_id.clone()).transfer(amount);
        }
        //we need to add back the storage being used up into the map if it's greater than 0.
        //this is so that if the user had 500 sales on the market, we insert that value here so
        //if those sales get taken down, the user can then go and withdraw 500 sales worth of storage.
        if diff > 0 {
            self.storage_deposits.insert(&owner_id, &diff);
        }
    }

    /// views
    //return the minimum storage for 1 sale
    pub fn storage_minimum_balance(&self) -> U128 {
        U128(STORAGE_PER_SALE)
    }

    //return how much storage an account has paid for
    pub fn storage_balance_of(&self, account_id: AccountId) -> U128 {
        U128(self.storage_deposits.get(&account_id).unwrap_or(0))
    }
}
