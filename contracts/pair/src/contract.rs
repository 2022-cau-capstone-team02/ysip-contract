use cosmwasm_std::{Addr, Decimal, DepsMut, Env, from_binary, MessageInfo, Reply, ReplyOn, Response, SubMsg, to_binary, WasmMsg};
use cw20::{Cw20ReceiveMsg, MinterResponse};
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
use cw2::set_contract_version;
use ysip::asset::{Asset, AssetInfo, format_lp_token_name};
use ysip::pair::{ExecuteMsg, InstantiateMsg, PairInfo, Cw20HookMsg};
use ysip::querier::query_token_precision;
use crate::error::ContractError;
use crate::state::{Config, CONFIG};
use crate::utils::compute_swap;

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

pub struct SwapParams {
    offer_asset: Asset,
    belief_price: Option<Decimal>,
    max_spread: Option<Decimal>,
    to: Option<Addr>,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
        ExecuteMsg::ProvideLiquidity { assets, receiver } => unimplemented!(),
        ExecuteMsg::Swap { offer_asset, belief_price, max_spread, to } => unimplemented!()
    }
}

fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&msg.msg) {
        Ok(Cw20HookMsg::Swap {
               belief_price,
               max_spread,
               to
           }) => {
            let mut authorized = false;
            let config: Config = CONFIG.load(deps.storage)?;

            config.pair_info.asset_infos.into_iter().for_each(|asset_info| {
                if let AssetInfo::Token { contract_addr } = &asset_info {
                    if contract_addr == &info.sender {
                        authorized = true;
                    }
                }
            });

            if !authorized {
                return Err(ContractError::Unauthorized {});
            }

            let to_addr = if let Some(to_addr) = to {
                Some(deps.api.addr_validate(to_addr.as_str())?)
            } else {
                None
            };

            let sender = deps.api.addr_validate(msg.sender.as_str())?;

            swap(
                deps,
                env,
                info,
                sender,
                SwapParams {
                    offer_asset: Asset {
                        info: AssetInfo::Token { contract_addr },
                        amount: msg.amount,
                    },
                    belief_price,
                    max_spread,
                    to: to_addr,
                },
            )
        }
        Ok(Cw20HookMsg::WithdrawLiquidity {}) => {
            withdraw_liquidity(deps, env, info, Addr::unchecked(msg.sender), msg.amount)
        }
        Err(err) => Err(ContractError::Std(err))
    }
}

fn swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    params: SwapParams,
) -> Result<Response, ContractError> {
    params.offer_asset.assert_sent_native_token_balance(&info)?;
    let mut config = CONFIG.load(deps.storage)?;

    Ok(Response::new())
}
