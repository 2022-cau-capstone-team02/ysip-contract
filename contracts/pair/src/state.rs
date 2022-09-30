use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ysip::pair::PairInfo;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub pair_info: PairInfo,
    pub factory_addr: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");