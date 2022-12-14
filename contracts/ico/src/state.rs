use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    /// code_id for channel token contract
    pub token_code_id: u64,
    pub token_name: String,
    pub token_symbol: String,
    pub target_funding_amount: Uint128,
    pub current_funding_amount: Uint128,
    /// Circulating channel token amount
    pub channel_token_amount: Uint128,
    pub deadline: u64,
    pub finished: bool,
    pub token_contract: Addr,
    pub pair_contract: Addr,
    /// recipient who is getting fund
    pub recipient: Addr,
    /// is token already distributed after fund
    pub is_token_distributed: bool,
}

impl Config {
    pub fn is_finished(&self) -> bool {
        self.finished
    }
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const FUNDING: Map<Addr, Uint128> = Map::new("funding");
