use cosmwasm_std::StdError;
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
}
