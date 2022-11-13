use crate::msg::{FundingAmountResponse, IsFundingFinishedResponse, PairAddressResponse, TokenAddressResponse};
use crate::state::{CONFIG, FUNDING};
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

pub fn funding_amount(deps: Deps, addr: &str) -> StdResult<Binary> {
    let address = deps.api.addr_validate(addr)?;
    let funding = FUNDING.load(deps.storage, address)?;
    Ok(to_binary(&FundingAmountResponse { amount: funding })?)
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

pub fn pair_address(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    Ok(to_binary(&PairAddressResponse {
        address: config.token_contract.to_string(),
    })?)
}