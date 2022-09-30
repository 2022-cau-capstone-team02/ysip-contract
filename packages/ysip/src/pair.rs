use cosmwasm_std::{Addr, Binary};
use crate::asset::AssetInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Information about the two assets in the pool
    pub asset_infos: [AssetInfo; 2],
    /// The token contract code ID used for the tokens in the pool
    pub token_code_id: u64,
    /// The factory contract address
    pub factory_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PairInfo {
    pub asset_infos: [AssetInfo; 2],
    pub contract_addr: Addr,
    pub liquidity_token: Addr,
}

impl PairInfo {
    pub fn init(contract_addr: Addr, asset_infos: [AssetInfo; 2]) -> PairInfo {
        Self {
            asset_infos,
            contract_addr,
            liquidity_token: Addr::unchecked(""),
        }
    }
}
