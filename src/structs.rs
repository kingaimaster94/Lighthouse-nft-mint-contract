use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CustomMsg};
#[cw_serde]
pub struct CollectionsResponse<T> {
    pub collections: Vec<T>,
}
#[cw_serde]
pub struct CollectionResponseMinimal {
    pub cw721_address: Option<Addr>
}
#[cw_serde]
pub struct Creator {
    pub address: String,
    pub share: u8,
}
#[cw_serde]
pub struct Cw2981InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub minter: String,
    pub frozen:bool,
    pub hidden_metadata:bool,
    pub placeholder_token_uri: Option<String>,
}

#[cw_serde]
pub enum Cw2981LHExecuteMsg {
    Unfreeze {},
    Reveal {},
    UpdateRevealData {
        placeholder_token_uri: Option<String>,
    },
}
impl CustomMsg for Cw2981LHExecuteMsg {}