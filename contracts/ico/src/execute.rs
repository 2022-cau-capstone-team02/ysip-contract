use crate::error::ContractError;
use crate::state::{CONFIG, FUNDING};
use cosmwasm_std::{DepsMut, MessageInfo, Response, Uint128};

pub fn fund_channel_token(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.is_finished() {
        return Err(ContractError::FundingFinished {});
    }

    let fund_coin = info
        .funds
        .iter()
        .find(|fund| fund.denom == "ukrw")
        .ok_or_else(|_| ContractError::InvalidCoinAmount {})?;

    if fund_coin.amount == Uint128::zero() {
        return Err(ContractError::InvalidCoinAmount {});
    }

    FUNDING.update(
        deps.storage,
        info.sender,
        |mut funding| -> Result<_, ContractError> {
            funding = match funding {
                Some(mut funding) => {
                    funding = funding.checked_add(fund_coin.amount)?;
                    Some(funding)
                }
                None => Some(fund_coin.amount),
            };
            Ok(funding)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "fund_channel_token")
        .add_attribute("amount", fund_coin.amount))
}

pub fn end_funding(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    Ok(Response::new())
}

pub fn refund(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    Ok(Response::new())
}
