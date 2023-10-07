use near_sdk::AccountId;

impl Contract {
    #[init]
    pub fn new_default(owner_id: AccountId) -> Self {
        Self {}
    }
}
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
