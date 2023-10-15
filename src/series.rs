use crate::*;

#[near_bindgen]
impl Contract {
    // Get the total supply of series on the contract
    pub fn get_series_total_supply(&self) -> u64 {
        self.series_by_id.len()
    }

    // Paginate through all the series on the contract and return the a vector of JsonSeries
    pub fn get_series(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<JsonSeries> {
        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = u128::from(from_index.unwrap_or(U128(0)));

        //iterate through each series using an iterator
        self.series_by_id
            .keys()
            //skip to the index we specified in the start variable
            .skip(start as usize)
            //take the first "limit" elements in the vector. If we didn't specify a limit, use 50
            .take(limit.unwrap_or(50) as usize)
            //we'll map the series IDs which are strings into Json Series
            .map(|series_id| self.get_series_details(series_id.clone()).unwrap())
            //since we turned the keys into an iterator, we need to turn it back into a vector to return
            .collect()
    }

    //get series by owner
    pub fn get_series_by_owner(
        &self,
        owner_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonSeries> {
        let series_for_owner_set = self.series_per_owner.get(&owner_id);
        let series = if let Some(series_for_owner_set) = series_for_owner_set {
            series_for_owner_set
        } else {
            return vec![];
        };

        let start = u128::from(from_index.unwrap_or(U128(0)));

        series
            .iter()
            .skip(start as usize)
            .take(limit.unwrap_or(50) as usize)
            .map(|series_id| self.get_series_details(series_id.clone()).unwrap())
            .collect()
    }

    // get info for a specific series
    pub fn get_series_details(&self, id: u64) -> Option<JsonSeries> {
        //get the series from the map
        let series = self.series_by_id.get(&id);
        //if there is some series, we'll return the series
        if let Some(series) = series {
            Some(JsonSeries {
                series_id: id,
                metadata: series.metadata,
                royalty: series.royalty,
                owner_id: series.owner_id,
                volume: series.volume,
                price: series.price,
            })
        } else {
            //if there isn't a series, we'll return None
            None
        }
    }

    //get the total supply of NFTs on a current series
    pub fn nft_supply_for_series(&self, id: u64) -> U128 {
        //get the series
        let series = self.series_by_id.get(&id);

        //if there is some series, get the length of the tokens. Otherwise return -
        if let Some(series) = series {
            U128(series.tokens.len() as u128)
        } else {
            U128(0)
        }
    }

    #[payable]
    pub fn create_series(&mut self, metadata: SeriesMetadata) {
        // Measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();

        // Ensure the caller is an approved creator
        let caller = env::predecessor_account_id();
        let owner = env::predecessor_account_id();
        // require!(
        //     self.approved_creators.contains(&caller) == true,
        //     "only approved creators can add a type"
        // );
        let id = self.get_series_total_supply() + 1;
        // Insert the series and ensure it doesn't already exist
        require!(
            self.series_by_id
                .insert(
                    &id,
                    &Series {
                        metadata,
                        volume: None,
                        royalty: None,
                        tokens: UnorderedSet::new(StorageKey::SeriesByIdInner {
                            // We get a new unique prefix for the collection
                            account_id_hash: hash_account_id(&caller),
                        }),
                        owner_id: caller,
                        price: None,
                    }
                )
                .is_none(),
            "collection ID already exists"
        );

        self.internal_add_series_to_owner(&owner, &id);

        //calculate the required storage which was the used - initial
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        //refund any excess storage if the user attached too much. Panic if they didn't attach enough to cover the required.
        refund_deposit(required_storage_in_bytes);
    }
}
