use cosmwasm_std::{StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Funding finished")]
    FundingFinished {},

    #[error("Invalid coin amount")]
    InvalidCoinAmount {},

    #[error("Funding not finished")]
    FundingNotFinished {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Not Found")]
    NotFound {},

    #[error("{inner}")]
    Generic { inner: String },

    #[error("Token already distributed")]
    TokenAlreadyDistributed {},
}
