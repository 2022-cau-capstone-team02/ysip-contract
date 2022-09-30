use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, SubMsg, to_binary, WasmMsg};
use cw20::MinterResponse;
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
use cw2::set_contract_version;
use ysip::asset::format_lp_token_name;
use ysip::pair::{InstantiateMsg, PairInfo};
use crate::error::ContractError;
use crate::state::{Config, CONFIG};

const CONTRACT_NAME: &str = "ysip-pair-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const INSTANTIATE_TOKEN_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    msg.asset_infos.into_iter().for_each(|asset_info| asset_info.check_is_valid(deps.api)?);

    if msg.asset_infos[0] == msg.asset_infos[1] {
        return Err(ContractError::OverlappingAssets {});
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        pair_info: PairInfo::init(env.contract.address.clone(), msg.asset_infos.clone()),
        factory_addr: Addr::unchecked(""),
    };

    CONFIG.save(deps.storage, &config)?;

    let lp_token_name = format_lp_token_name(msg.asset_infos, &deps.querier)?;

    let sub_msg = SubMsg {
        id: INSTANTIATE_TOKEN_REPLY_ID,
        msg: WasmMsg::Instantiate {
            admin: Some(env.contract.address.to_string()),
            code_id: msg.token_code_id,
            msg: to_binary(&Cw20InstantiateMsg {
                name: lp_token_name,
                symbol: "uLP".to_string(),
                decimals: 6,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.to_string(),
                    cap: None,
                }),
                marketing: None,
            })?,
            funds: vec![],
            label: "YSIP LP token".to_string(),
        }.into(),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    };

    Ok(Response::new().add_submessage(sub_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    // If liquidity token have already instantiated
    if config.pair_info.liquidity_token != Addr::unchecked("") {
        return Err(ContractError::Unauthorized {});
    }

    let res = cw_utils::parse_reply_instantiate_data(msg).unwrap();

    config.pair_info.liquidity_token = deps.api.addr_validate(res.contract_address.as_str())?;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("liquidity_token_addr", config.pair_info.liquidity_token))
}