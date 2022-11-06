use crate::error::ContractError;
use crate::state::{CONFIG, FUNDING};
use cosmwasm_std::{CosmosMsg, DepsMut, Env, MessageInfo, Response, StdError, Uint128};
use crate::utils::get_bank_transfer_to_msg;

pub fn fund_channel_token(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    config.update_funding_state(env.block.height);

    if config.is_finished() {
        return Err(ContractError::FundingFinished {});
    }

    let max_fund_available_amount = config.target_funding_amount
        .checked_sub(config.current_funding_amount)
        .map_err(StdError::overflow)?;

    let input_coin = info
        .funds
        .iter()
        .find(|fund| fund.denom == "ukrw")
        .ok_or_else(|| ContractError::InvalidCoinAmount {})?;

    if input_coin.amount == Uint128::zero() {
        return Err(ContractError::InvalidCoinAmount {});
    }

    let (fund_amount, refund_amount) = {
        if max_fund_available_amount < input_coin.amount {
            let refund_amount = input_coin.amount
                .checked_sub(max_fund_available_amount)
                .map_err(StdError::overflow)?;
            (max_fund_available_amount, refund_amount)
        } else {
            (input_coin.amount, Uint128::zero())
        }
    };

    let mut refund_msg: Vec<CosmosMsg> = vec![];

    if !refund_amount.eq(&Uint128::zero()) {
        refund_msg.push(get_bank_transfer_to_msg(&info.sender, "ukrw", refund_amount));
    }

    config.current_funding_amount = config.current_funding_amount
        .checked_add(fund_amount)
        .map_err(StdError::overflow)?;

    if config.current_funding_amount >= config.target_funding_amount {
        config.finished = true;
    }

    CONFIG.save(deps.storage, &config)?;

    FUNDING.update(
        deps.storage,
        info.sender,
        |funding| -> Result<_, ContractError> {
            let mut new_funding = match funding {
                Some(mut funding) => funding
                    .checked_add(fund_amount)
                    .map_err(StdError::overflow)?,
                None => fund_amount,
            };
            Ok(new_funding)
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "fund_channel_token")
        .add_attribute("amount", fund_amount)
        .add_messages(refund_msg)
    )
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
            target_funding_amount: Uint128::new(100),
            current_funding_amount: Uint128::zero(),
            deadline: 12_346,
            finished: false,
            token_contract: Addr::unchecked(""),
        }).unwrap();

        let res = fund_channel_token(
            deps.as_mut(),
            mock_env(),
            mock_info(
                ADDR,
                &[coin(10000, "ukrw")],
            ),
        ).unwrap();
        let funding = FUNDING.load(&deps.storage, Addr::unchecked(ADDR)).unwrap();
        assert_eq!(funding, Uint128::new(100))
    }
}
