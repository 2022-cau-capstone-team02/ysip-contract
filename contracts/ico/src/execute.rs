use crate::contract::END_FUNDING_REPLAY_ID;
use crate::error::ContractError;
use crate::state::{CONFIG, FUNDING};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, DepsMut, Env, MessageInfo, ReplyOn, Response, StdError, SubMsg,
    Uint128, WasmMsg,
};
use cw20::{AllAccountsResponse, MinterResponse, TokenInfoResponse};
use cw20_base::msg::QueryMsg::{AllAccounts, TokenInfo};
use ysip::querier::query_token_balance;
use ysip::utils::get_bank_transfer_to_msg;

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
    let mut config = CONFIG.load(deps.storage)?;
    if config.is_token_distributed {
        return Err(ContractError::TokenAlreadyDistributed {});
    }

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

    config.is_token_distributed = true;

    CONFIG.save(deps.storage, &config)?;

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

pub fn transfer_fund(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if !config.is_finished() {
        return Err(ContractError::FundingNotFinished {});
    }

    let transfer_msg = get_bank_transfer_to_msg(&config.recipient, "ukrw", amount);

    Ok(Response::new()
        .add_attribute("action", "transfer_fund")
        .add_attribute("to", config.recipient)
        .add_attribute("amount", amount)
        .add_message(transfer_msg))
}

pub fn allocation(
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let fund = info
        .funds
        .iter()
        .find(|coin| coin.denom == "ukrw")
        .expect("coin not found");
    if fund.amount != amount {
        return Err(ContractError::InvalidCoinAmount {});
    }

    let config = CONFIG.load(deps.storage)?;

    let all_accounts: AllAccountsResponse = deps.querier.query_wasm_smart(
        config.token_contract.clone(),
        &AllAccounts {
            start_after: None,
            limit: None,
        },
    )?;

    let token_info: TokenInfoResponse = deps
        .querier
        .query_wasm_smart(config.token_contract.clone(), &TokenInfo {})?;

    let total_supply = token_info.total_supply;

    let mut transfer_msgs: Vec<CosmosMsg> = vec![];

    for account in all_accounts.accounts {
        let balance = query_token_balance(
            &deps.querier,
            &config.token_contract,
            &Addr::unchecked(account.clone()),
        )
        .expect("token balance not found");

        if !balance.eq(&Uint128::zero()) {
            transfer_msgs.push(get_bank_transfer_to_msg(
                &Addr::unchecked(account),
                "ukrw",
                fund.amount
                    .checked_multiply_ratio(balance, total_supply)
                    .expect("overflow"),
            ));
        }
    }

    Ok(Response::new()
        .add_attribute("action", "allocation")
        .add_messages(transfer_msgs))
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
                    recipient: Addr::unchecked(ADDR),
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
