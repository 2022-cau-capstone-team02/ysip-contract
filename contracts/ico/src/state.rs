use cosmwasm_std::{Addr, StdResult, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::error::ContractError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub target_funding_amount: Uint128,
    pub current_funding_amount: Uint128,
    pub deadline: u64,
    pub finished: bool,
    pub token_contract: Addr,
}

impl Config {
    pub fn is_finished(&self) -> bool {
        self.finished
    }
    pub fn update_funding_state(&mut self, block_height: u64) {
        self.finished = block_height > self.deadline;
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const FUNDING: Map<Addr, Uint128> = Map::new("funding");
