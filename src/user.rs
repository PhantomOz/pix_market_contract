use crate::*;

trait UserMetadataCore {
    fn get_user_metadata(&self, owner_Id: AccountId) -> Option<UserMetadata>;
    fn set_user_banner_image(&mut self, owner_id: AccountId, img_url: Option<String>) -> bool;
    fn set_user_image(&mut self, owner_id: AccountId, img_url: Option<String>) -> bool;
    fn set_user_name(&mut self, owner_id: AccountId, name: Option<String>) -> bool;
}

#[near_bindgen]
impl UserMetadataCore for Contract {
    fn get_user_metadata(&self, owner_id: AccountId) -> Option<UserMetadata> {
        if let Some(user_metadata) = self.user_metadata_by_owner.get(&owner_id) {
            return Some(UserMetadata {
                name: user_metadata.name,
                image_url: user_metadata.image_url,
                banner_url: user_metadata.banner_url,
            });
        } else {
            None
        }
    }

    fn set_user_banner_image(&mut self, owner_id: AccountId, img_url: Option<String>) -> bool {
        if let Some(user_metadata) = self.user_metadata_by_owner.get(&owner_id) {
            let metadata = UserMetadata {
                name: user_metadata.name,
                image_url: user_metadata.image_url,
                banner_url: img_url,
            };
            self.user_metadata_by_owner.insert(&owner_id, &metadata);
        } else {
            let metadata = UserMetadata {
                name: None,
                image_url: None,
                banner_url: img_url,
            };
            self.user_metadata_by_owner.insert(&owner_id, &metadata);
        }
        return true;
    }

    fn set_user_image(&mut self, owner_id: AccountId, img_url: Option<String>) -> bool {
        if let Some(user_metadata) = self.user_metadata_by_owner.get(&owner_id) {
            let metadata = UserMetadata {
                name: user_metadata.name,
                image_url: img_url,
                banner_url: user_metadata.banner_url,
            };
            self.user_metadata_by_owner.insert(&owner_id, &metadata);
        } else {
            let metadata = UserMetadata {
                name: None,
                image_url: img_url,
                banner_url: None,
            };
            self.user_metadata_by_owner.insert(&owner_id, &metadata);
        }
        return true;
    }

    fn set_user_name(&mut self, owner_id: AccountId, name: Option<String>) -> bool {
        if let Some(user_metadata) = self.user_metadata_by_owner.get(&owner_id) {
            let metadata = UserMetadata {
                name: name,
                image_url: user_metadata.image_url,
                banner_url: user_metadata.banner_url,
            };
            self.user_metadata_by_owner.insert(&owner_id, &metadata);
        } else {
            let metadata = UserMetadata {
                name: name,
                image_url: None,
                banner_url: None,
            };
            self.user_metadata_by_owner.insert(&owner_id, &metadata);
        }
        return true;
    }
}
