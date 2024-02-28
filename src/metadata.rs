use crate::*;

pub type TokenId = String;
pub type SeriesId = u64;
pub type SalePriceInYoctoNear = U128;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
}
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NFTContractMetadata {
    pub spec: String,
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub base_uri: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<Base64VecU8>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub media: Option<String>,
    pub media_hash: Option<String>,
    pub copies: Option<u64>,
    pub issued_at: Option<String>,
    pub extra: Option<String>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Token {
    pub series_id: u64,
    pub owner_id: AccountId,
    pub approved_account_ids: HashMap<AccountId, u64>,
    pub next_approval_id: u64,
    pub royalty: HashMap<AccountId, u32>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SeriesMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub logo_media: Option<String>,
    pub banner_media: Option<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UserMetadata {
    pub name: Option<String>,
    pub image_url: Option<String>,
    pub banner_url: Option<String>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Series {
    // Royalty used for all tokens in the collection
    pub royalty: Option<HashMap<AccountId, u32>>,
    // Set of tokens in the collection
    pub tokens: UnorderedSet<TokenId>,
    // What is the price of each token in this series? If this is specified, when minting,
    // Users will need to attach enough $NEAR to cover the price.
    pub price: Option<Balance>,
    //total price of the series
    pub volume: Option<Balance>,
    // Owner of the collection
    pub owner_id: AccountId,
    //collection logo
    pub metadata: SeriesMetadata,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonToken {
    pub series_id: SeriesId,
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub metadata: TokenMetadata,
    pub approved_account_ids: HashMap<AccountId, u64>,
    pub royalty: HashMap<AccountId, u32>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonSeries {
    pub series_id: SeriesId,
    // Metadata including title, num copies etc.. that all tokens will derive from
    pub metadata: SeriesMetadata,
    // Royalty used for all tokens in the collection
    pub royalty: Option<HashMap<AccountId, u32>>,
    // Owner of the collection
    pub owner_id: AccountId,
    //volume
    pub volume: Option<Balance>,
    // price
    pub price: Option<Balance>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SaleArgs {
    pub sale_conditions: SalePriceInYoctoNear,
}

pub trait NonFungibleTokenMetadata {
    fn nft_metadata(&self) -> NFTContractMetadata;
}

#[near_bindgen]
impl NonFungibleTokenMetadata for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}
