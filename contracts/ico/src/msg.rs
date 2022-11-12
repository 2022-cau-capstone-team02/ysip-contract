use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub target_funding: Uint128,
    /// block height for deadline
    pub deadline: u64,
    pub token_code_id: u64,
    pub pair_code_id: u64,
    pub token_name: String,
    pub token_symbol: String,
    pub channel_token_amount: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    FundChannelToken {},
    /// only admin of ico contract can call EndFunding
    EndFunding {},
    /// if deadline ends, user can request refund
    Refund {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    FundingAmount { addr: String },
    IsFundingFinished {},
}
