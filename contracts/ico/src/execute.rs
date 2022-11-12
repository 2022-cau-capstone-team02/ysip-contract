use crate::contract::END_FUNDING_REPLAY_ID;
use crate::error::ContractError;
use crate::state::{CONFIG, FUNDING};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, Order, Reply, ReplyOn, Response,
    StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, MinterResponse};
use cw20_base::state::BALANCES;
use cw_storage_plus::Bound;
use ysip::asset::AssetInfo;
use ysip::utils::{get_bank_transfer_to_msg, get_cw20_transfer_msg};

pub fn fund_channel_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if env.block.height > config.deadline {
        return Err(ContractError::FundingFinished {});
    }

    if config.is_finished() {
        return Err(ContractError::FundingFinished {});
    }

    let max_fund_available_amount = config
        .target_funding_amount
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
            let refund_amount = input_coin
                .amount
                .checked_sub(max_fund_available_amount)
                .map_err(StdError::overflow)?;
            (max_fund_available_amount, refund_amount)
        } else {
            (input_coin.amount, Uint128::zero())
        }
    };

    let mut refund_msg: Vec<CosmosMsg> = vec![];

    if !refund_amount.eq(&Uint128::zero()) {
        refund_msg.push(get_bank_transfer_to_msg(
            &info.sender,
            "ukrw",
            refund_amount,
        ));
    }

    config.current_funding_amount = config
        .current_funding_amount
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
            let new_funding = match funding {
                Some(funding) => funding
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
        .add_messages(refund_msg))
}

pub fn end_funding(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if !config.is_finished() && env.block.height < config.deadline {
        return Err(ContractError::FundingNotFinished {});
    }

    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut sub_msg: Vec<SubMsg> = vec![];

    if config.is_finished() {
        sub_msg.push(SubMsg {
            id: END_FUNDING_REPLAY_ID,
            msg: WasmMsg::Instantiate {
                admin: Some(info.sender.to_string()),
                code_id: config.token_code_id,
                msg: to_binary(&cw20_base::msg::InstantiateMsg {
                    name: config.token_name.to_string(),
                    symbol: config.token_symbol.to_string(),
                    decimals: 6,
                    initial_balances: vec![],
                    mint: Some(MinterResponse {
                        minter: env.contract.address.to_string(),
                        cap: None,
                    }),
                    marketing: None,
                })?,
                funds: vec![],
                label: format!("{} channel token", config.token_name),
            }
            .into(),
            gas_limit: None,
            reply_on: ReplyOn::Success,
        });
    }

    Ok(Response::new()
        .add_attribute("action", "end_funding")
        .add_submessages(sub_msg))
}

pub fn refund(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if env.block.height < config.deadline {
        return Err(ContractError::FundingNotFinished {});
    }

    if config.is_finished() {
        return Err(ContractError::FundingFinished {});
    }

    let funded_amount = FUNDING.may_load(deps.storage, info.sender.clone())?;

    let mut msgs: Vec<CosmosMsg> = vec![];

    match funded_amount {
        Some(amount) => {
            msgs.push(get_bank_transfer_to_msg(&info.sender, "ukrw", amount));
        }
        None => return Err(ContractError::NotFound {}),
    }

    FUNDING.remove(deps.storage, info.sender);

    Ok(Response::new()
        .add_attribute("action", "refund")
        .add_attribute("amount", funded_amount.unwrap())
        .add_messages(msgs))
}

#[cfg(test)]
mod test_ico {
    use crate::execute::{end_funding, fund_channel_token};
    use crate::state::{Config, CONFIG, FUNDING};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, Addr, Uint128};

    const ADDR: &str = "cosmos18zfp9u7zxg3gel4r3txa2jqxme7jkw7dnvfjc8";

    #[test]
    fn test_fund_channel_token() {
        let mut deps = mock_dependencies();
        let config = CONFIG
            .save(
                &mut deps.storage,
                &Config {
                    admin: Addr::unchecked(ADDR),
                    token_code_id: 1,
                    pair_code_id: 2,
                    token_name: "channel".to_string(),
                    token_symbol: "CHANNEL".to_string(),
                    target_funding_amount: Uint128::new(100),
                    current_funding_amount: Uint128::zero(),
                    channel_token_amount: Uint128::new(100000),
                    deadline: 12_346,
                    finished: false,
                    token_contract: Addr::unchecked(""),
                },
            )
            .unwrap();

        let res = fund_channel_token(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR, &[coin(10000, "ukrw")]),
        )
        .unwrap();
        let funding = FUNDING.load(&deps.storage, Addr::unchecked(ADDR)).unwrap();

        let res = end_funding(
            deps.as_mut(),
            mock_env(),
            mock_info(ADDR, &[coin(10000, "ukrw")]),
        )
        .unwrap();
    }
}
