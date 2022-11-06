use crate::error::ContractError;
use crate::state::{CONFIG, FUNDING};
use cosmwasm_std::{DepsMut, MessageInfo, Response, StdError, Uint128};

pub fn fund_channel_token(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.is_finished() {
        return Err(ContractError::FundingFinished {});
    }

    let fund_coin = info
        .funds
        .iter()
        .find(|fund| fund.denom == "ukrw")
        .ok_or_else(|| ContractError::InvalidCoinAmount {})?;

    if fund_coin.amount == Uint128::zero() {
        return Err(ContractError::InvalidCoinAmount {});
    }

    FUNDING.update(
        deps.storage,
        info.sender,
        |funding| -> Result<_, ContractError> {
            let mut new_funding = match funding {
                Some(mut funding) => funding
                    .checked_add(fund_coin.amount)
                    .map_err(StdError::overflow)?,
                None => fund_coin.amount,
            };
            Ok(new_funding)
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

#[cfg(test)]
mod test_ico {
    use crate::execute::fund_channel_token;
    use cosmwasm_std::{Addr, coin, Uint128};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use crate::state::{CONFIG, Config, FUNDING};

    const ADDR: &str = "cosmos18zfp9u7zxg3gel4r3txa2jqxme7jkw7dnvfjc8";

    #[test]
    fn test_fund_channel_token() {
        let mut deps = mock_dependencies();
        let config = CONFIG.save(&mut deps.storage, &Config {
            admin: Addr::unchecked(ADDR),
            target_funding: Uint128::new(100),
            deadline: 10,
            is_finished: false,
            token_contract: Addr::unchecked(""),
        }).unwrap();

        let res = fund_channel_token(deps.as_mut(), mock_info(ADDR, &[coin(10000, "ukrw")])).unwrap();
        let funding = FUNDING.load(&deps.storage, Addr::unchecked(ADDR)).unwrap();
        assert_eq!(funding, Uint128::new(10000))
    }
}
