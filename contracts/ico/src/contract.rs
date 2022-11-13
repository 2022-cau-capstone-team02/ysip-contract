use crate::error::ContractError;
use crate::{execute, query};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order, Reply,
    ReplyOn, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;
use ysip::asset::AssetInfo;
use ysip::utils::get_cw20_transfer_msg;

const CONTRACT_NAME: &str = "ysip-ico-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const END_FUNDING_REPLAY_ID: u64 = 1;
pub const LP_POOL_INSTANTIATE_ID: u64 = 2;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, FUNDING};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        admin: info.sender,
        token_code_id: msg.token_code_id,
        pair_code_id: msg.pair_code_id,
        token_name: msg.token_name.to_string(),
        token_symbol: msg.token_symbol.to_string(),
        target_funding_amount: msg.target_funding,
        current_funding_amount: Uint128::zero(),
        channel_token_amount: Uint128::new(msg.channel_token_amount),
        deadline: msg.deadline,
        token_contract: Addr::unchecked(""),
        finished: false,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("target_funding", msg.target_funding))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::FundChannelToken {} => execute::fund_channel_token(deps, env, info),
        ExecuteMsg::EndFunding {} => execute::end_funding(deps, env, info),
        ExecuteMsg::Refund {} => execute::refund(deps, env, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::FundingAmount { addr } => query::funding_amount(deps, &addr),
        QueryMsg::IsFundingFinished {} => query::funding_finished(deps, env),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    let res = cw_utils::parse_reply_instantiate_data(msg.clone()).unwrap();

    match msg.id {
        END_FUNDING_REPLAY_ID => {
            let mint_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: res.contract_address.clone(),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: env.contract.address.to_string(),
                    amount: config.channel_token_amount,
                })?,
                funds: vec![],
            });

            let funding: StdResult<Vec<(Addr, Uint128)>> = FUNDING
                .range(deps.storage, None, None, Order::Ascending)
                .collect();

            let transfer_ico_tokens_msgs = funding
                .map_err(|_| ContractError::NotFound {})?
                .iter()
                .map(|(addr, amount)| -> Result<CosmosMsg, ContractError> {
                    Ok(get_cw20_transfer_msg(
                        addr,
                        &Addr::unchecked(&res.contract_address),
                        config
                            .channel_token_amount
                            .checked_multiply_ratio(*amount, config.current_funding_amount)
                            .map_err(|e| ContractError::Generic {
                                inner: format!("{:?}", e),
                            })?,
                    )
                    .map_err(|_| ContractError::NotFound {})?)
                })
                .collect::<Result<Vec<CosmosMsg>, ContractError>>();

            config.token_contract = Addr::unchecked(res.contract_address.clone());
            CONFIG.save(deps.storage, &config)?;

            let instantiate_lp_msg = SubMsg {
                id: LP_POOL_INSTANTIATE_ID,
                msg: WasmMsg::Instantiate {
                    admin: None,
                    code_id: config.pair_code_id,
                    msg: to_binary(&ysip::pair::InstantiateMsg {
                        asset_infos: [
                            AssetInfo::Token {
                                contract_addr: Addr::unchecked(res.contract_address.clone()),
                            },
                            AssetInfo::NativeToken {
                                denom: "ukrw".to_string(),
                            },
                        ],
                        token_code_id: config.token_code_id,
                        protocol_fee_recipient: config.admin.to_string(),
                        protocol_fee_percent: "0.15".to_string(),
                        lp_fee_percent: "0.15".to_string(),
                    })?,
                    funds: vec![],
                    label: format!("{}-ukrw lp pair", config.token_symbol),
                }
                .into(),
                gas_limit: None,
                reply_on: ReplyOn::Success,
            };

            Ok(Response::new()
                .add_attribute("channel_token_instantiate", res.clone().contract_address)
                .add_attribute("channel_token_mint", config.channel_token_amount)
                .add_message(mint_msg)
                .add_messages(transfer_ico_tokens_msgs?)
                .add_submessage(instantiate_lp_msg))
        }
        LP_POOL_INSTANTIATE_ID => {
            Ok(Response::new().add_attribute("lp_pool_instantiate", res.contract_address))
        }
        _ => Err(ContractError::NotFound {}),
    }
}
