use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub target_funding: Uint128,
    pub deadline: u64,
    pub is_finished: bool,
    pub token_contract: Addr,
}

impl Config {
    pub fn is_finished(&self) -> bool {
        self.is_finished
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const FUNDING: Map<Addr, Uint128> = Map::new("funding");
