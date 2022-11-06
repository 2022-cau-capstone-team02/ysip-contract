use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Overlapping assets in asset infos")]
    OverlappingAssets {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Not enough liquidity")]
    NotEnoughLiquidity {},

    #[error("Asset Mismatch")]
    AssetMismatch {},

    #[error("InvalidZeroAmount")]
    InvalidZeroAmount {},

    #[error("Not enough token amount: {need} required, {supplied} supplied")]
    NotEnoughTokenAmount { need: Uint128, supplied: Uint128 },

    #[error("Not enough balance: {avaiable} avaiable, {requested} requested")]
    NotEnoughBalance {
        avaiable: Uint128,
        requested: Uint128,
    },
}
