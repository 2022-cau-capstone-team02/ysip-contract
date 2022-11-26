use crate::msg::{
    FundingAmountResponse, IcoInfoResponse, IsFundingFinishedResponse, TokenAddressResponse,
    TotalFundingAmountResponse,
};
use crate::state::{CONFIG, FUNDING};
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult, Uint128};

pub fn ico_info(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_binary(&IcoInfoResponse {
        admin: config.admin.to_string(),
        token_name: config.token_name.to_string(),
        token_symbol: config.token_symbol.to_string(),
        target_funding_amount: config.target_funding_amount,
        deadline: config.deadline,
    })?)
}

pub fn funding_amount(deps: Deps, addr: &str) -> StdResult<Binary> {
    let address = deps.api.addr_validate(addr)?;
    let funding = FUNDING.load(deps.storage, address)?;
    Ok(to_binary(&FundingAmountResponse { amount: funding })?)
}

pub fn total_funding_amount(deps: Deps) -> StdResult<Binary> {
    let funding: Uint128 = FUNDING
        .range(deps.storage, None, None, Order::Descending)
        .into_iter()
        .filter_map(|val| val.ok())
        .map(|val| val.1)
        .sum();

    Ok(to_binary(&TotalFundingAmountResponse { amount: funding })?)
}

pub fn funding_finished(deps: Deps, env: Env) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    match config.is_finished() || env.block.height > config.deadline {
        true => Ok(to_binary(&IsFundingFinishedResponse { status: true })?),
        false => Ok(to_binary(&IsFundingFinishedResponse { status: false })?),
    }
}

pub fn token_address(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_binary(&TokenAddressResponse {
        address: config.token_contract.to_string(),
    })?)
}
