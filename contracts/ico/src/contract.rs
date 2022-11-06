use crate::error::ContractError;
use crate::execute::{end_funding, fund_channel_token, refund};
use cosmwasm_std::{entry_point, Addr, Deps, DepsMut, Env, MessageInfo, Reply, Response};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "ysip-ico-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};

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
        target_funding: msg.target_funding,
        deadline: msg.deadline,
        is_finished: false,
        token_contract: Addr::unchecked(""),
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("target_funding", msg.target_funding))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::FundChannelToken {} => fund_channel_token(deps, info),
        ExecuteMsg::EndFunding {} => end_funding(deps, info),
        ExecuteMsg::Refund {} => refund(deps, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    env: Env,
    info: MessageInfo,
    msg: QueryMsg,
) -> Result<Response, ContractError> {
    match msg {}
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let res = cw_utils::parse_reply_instantiate_data(msg).unwrap();
    let addr = res.contract_address;
    Ok(Response::new())
}
