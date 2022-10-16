use crate::error::ContractError;
use crate::math::{get_lp_fee_amount, get_protocol_fee_amount, get_swap_output_amount};
use crate::state::{Config, Fees, Liquidity, CONFIG, LIQUIDITY};
use cosmwasm_std::{
    attr, entry_point, from_binary, to_binary, Addr, Decimal, DepsMut, Env, MessageInfo, Reply,
    ReplyOn, Response, StdError, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Cw20ReceiveMsg, MinterResponse};
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
use std::str::FromStr;
use ysip::asset::{format_lp_token_name, Asset, AssetInfo};
use ysip::pair::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, PairInfo, SwapParams};
use ysip::utils::{get_bank_transfer_to_msg, get_cw20_transfer_from_msg, get_fee_transfer_msg};

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
    msg.asset_infos[0].clone().check_is_valid(deps.api)?;
    msg.asset_infos[1].clone().check_is_valid(deps.api)?;

    if msg.asset_infos[0] == msg.asset_infos[1] {
        return Err(ContractError::OverlappingAssets {});
    }

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        pair_info: PairInfo::init(env.contract.address.clone(), msg.asset_infos.clone()),
        factory_addr: Addr::unchecked(""),
        fees: Fees {
            protocol_fee_recipient: Addr::unchecked(msg.protocol_fee_recipient),
            protocol_fee_percent: Decimal::from_str(&msg.protocol_fee_percent)?
                / Decimal::from_str("100")?,
            lp_fee_percent: Decimal::from_str(&msg.lp_fee_percent)? / Decimal::from_str("100")?,
        },
    };

    let liquidity = Liquidity {
        token_a: Asset {
            info: msg.asset_infos[0].clone(),
            amount: Uint128::zero(),
        },
        token_b: Asset {
            info: msg.asset_infos[1].clone(),
            amount: Uint128::zero(),
        },
    };

    CONFIG.save(deps.storage, &config)?;
    LIQUIDITY.save(deps.storage, &liquidity)?;

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
        }
        .into(),
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
        ExecuteMsg::Swap {
            offer_asset,
            min_output_amount,
            max_spread,
            to,
        } => {
            unimplemented!();
        }
    }
}

fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let contract_addr = info.sender.clone();

    match from_binary(&msg.msg) {
        Ok(Cw20HookMsg::Swap {
            min_output_amount,
            max_spread,
            to,
        }) => {
            let mut authorized = false;
            let config: Config = CONFIG.load(deps.storage)?;

            config
                .pair_info
                .asset_infos
                .into_iter()
                .for_each(|asset_info| {
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

            // let sender = deps.api.addr_validate(msg.sender.as_str())?;

            swap(
                deps,
                env,
                info,
                SwapParams {
                    offer_asset: Asset {
                        info: AssetInfo::Token { contract_addr },
                        amount: msg.amount,
                    },
                    min_output_amount,
                    max_spread,
                    to: to_addr,
                },
            )
        }
        Ok(Cw20HookMsg::WithdrawLiquidity {}) => {
            unimplemented!();
            // withdraw_liquidity(deps, env, info, Addr::unchecked(msg.sender), msg.amount)
        }
        Err(err) => Err(ContractError::Std(err)),
    }
}

fn swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    params: SwapParams,
) -> Result<Response, ContractError> {
    params.offer_asset.assert_sent_native_token_balance(&info)?;
    let config = CONFIG.load(deps.storage)?;

    let pools: [Asset; 2] = config
        .pair_info
        .query_pools(&deps.querier, env.contract.address.clone())?;

    let offer_pool: Asset;
    let ask_pool: Asset;

    if params.offer_asset.info.eq(&pools[0].info) {
        offer_pool = pools[0].clone();
        ask_pool = pools[1].clone();
    } else if params.offer_asset.info.eq(&pools[1].info) {
        offer_pool = pools[1].clone();
        ask_pool = pools[0].clone();
    } else {
        return Err(ContractError::AssetMismatch {});
    }

    let fees = config.fees;
    let two_dec = Decimal::new(Uint128::new(2));
    let total_fee_percent = (fees.lp_fee_percent / two_dec) + fees.protocol_fee_percent;

    let protocol_fee_amount =
        get_protocol_fee_amount(params.offer_asset.amount, fees.protocol_fee_percent)?;
    let net_input_amount = params.offer_asset.amount - protocol_fee_amount;

    let token_bought_amount = get_swap_output_amount(
        net_input_amount,
        offer_pool.amount,
        ask_pool.amount,
        total_fee_percent,
    )?;

    let (input_token_fee_amount, output_token_fee_amount) = get_lp_fee_amount(
        net_input_amount,
        token_bought_amount,
        fees.lp_fee_percent / two_dec,
    )?;

    let net_token_output_amount = token_bought_amount - output_token_fee_amount;

    params.assert_min_token_bought(Decimal::new(net_token_output_amount))?;

    // send input token or coin to contract
    let mut msgs = match params.offer_asset.info.clone() {
        AssetInfo::Token { contract_addr } => vec![get_cw20_transfer_from_msg(
            &info.sender,
            &env.contract.address.clone(),
            &contract_addr,
            net_input_amount,
        )?],
        AssetInfo::NativeToken { denom } => vec![get_bank_transfer_to_msg(
            &info.sender.clone(),
            &denom,
            net_input_amount,
        )],
    };

    msgs.push(get_fee_transfer_msg(
        &info.sender,
        &fees.protocol_fee_recipient,
        Asset {
            info: offer_pool.info.clone(),
            amount: protocol_fee_amount,
        },
    )?);

    LIQUIDITY.update(deps.storage, |mut liquidity| -> Result<_, ContractError> {
        if liquidity.token_a.info == offer_pool.info {
            liquidity
                .token_a
                .amount
                .checked_add(net_input_amount)
                .map_err(StdError::overflow)?;
            Ok(liquidity)
        } else if liquidity.token_b.info == offer_pool.info {
            liquidity
                .token_a
                .amount
                .checked_add(net_input_amount)
                .map_err(StdError::overflow)?;
            Ok(liquidity)
        } else if liquidity.token_a.info == ask_pool.info {
            liquidity
                .token_b
                .amount
                .checked_add(net_input_amount)
                .map_err(StdError::overflow)?;
            Ok(liquidity)
        } else if liquidity.token_b.info == ask_pool.info {
            liquidity
                .token_b
                .amount
                .checked_add(net_input_amount)
                .map_err(StdError::overflow)?;
            Ok(liquidity)
        } else {
            Ok(liquidity)
        }
    })?;

    // Add liquidity with lp_fee
    // add_liquidity(input_token_fee_amount, output_token_fee_amount);

    Ok(Response::new()
        .add_attributes(vec![
            attr("action", "swap"),
            attr("token_in_amount", params.offer_asset.amount.to_string()),
            attr("token_out_amount", net_token_output_amount.to_string()),
        ])
        .add_messages(msgs))
}

fn get_lp_token_amount_to_mint(
    token1_amount: Uint128,
    liquidity_supply: Uint128,
    token1_reserve: Uint128,
) -> Result<Uint128, ContractError> {
    if liquidity_supply == Uint128::zero() {
        Ok(token1_amount)
    } else {
        Ok(token1_amount
            .checked_mul(liquidity_supply)
            .map_err(StdError::overflow)?
            .checked_div(token1_reserve)
            .map_err(StdError::divide_by_zero)?)
    }
}

fn get_token2_amount_required(
    max_token: Uint128,
    token1_amount: Uint128,
    liquidity_supply: Uint128,
    token2_reserve: Uint128,
    token1_reserve: Uint128,
) -> Result<Uint128, StdError> {
    if liquidity_supply == Uint128::zero() {
        Ok(max_token)
    } else {
        Ok(token1_amount
            .checked_mul(token2_reserve)
            .map_err(StdError::overflow)?
            .checked_div(token1_reserve)
            .map_err(StdError::divide_by_zero)?
            .checked_add(Uint128::new(1))
            .map_err(StdError::overflow)?)
    }
}

// fn provide_liquidity(
//     deps: DepsMut,
//     info: MessageInfo,
//     env: Env,
//     assets: [Asset; 2],
// ) -> Result<Response, ContractError> {
//     assets[0].info.check_is_valid(deps.api)?;
//     assets[1].info.check_is_valid(deps.api)?;
//
//     for asset in assets {
//         asset.assert_sent_native_token_balance(&info)?;
//     }
//
//     let config = CONFIG.load(deps.storage)?;
//
//     let pools: [Asset; 2] = config
//         .pair_info
//         .query_pools(&deps.querier, env.contract.address)?;
//
//     let mut deposits: [Uint128; 2] = [
//         assets
//             .iter()
//             .find(|a| a.info.equal(&pools[0].info))
//             .map(|a| a.amount)
//             .expect("Wrong asset info is given"),
//         assets
//             .iter()
//             .find(|a| a.info.equal(&pools[1].info))
//             .map(|a| a.amount)
//             .expect("Wrong asset info is given"),
//     ];
//
//     if deposits[0].is_zero() && deposits[1].is_zero() {
//         return Err(ContractError::InvalidZeroAmount {});
//     }
//
//     let lp_token_supply = get_lp_token_supply(deps.as_ref(), &lp_token_addr)?;
//     let liquidity_amount = get_lp_token_amount_to_mint(
//         deposits[0],
//         lp_token_supply,
//         assets[0].amount);
//
//     let token2_amount = get_token2_amount_required(
//         deposits[1],
//         deposits[0],
//         lp_token_supply,
//         assets[1].amount,
//         assets[0].amount,
//     )?;
//
//     let mut transfer_msgs: Vec<CosmosMsg> = vec![];
//     if let AssetInfo::Token(addr) = assets[0].clone().info {
//         transfer_msgs.push(get_cw20_transfer_from_msg(
//             &info.sender,
//             &env.contract.address,
//             &addr,
//             deposits[0],
//         )?)
//     }
//
//     if let AssetInfo::Token(addr) = assets[1].clone().info {
//         transfer_msgs.push(get_cw20_transfer_from_msg(
//             &info.sender,
//             &env.contract.address,
//             &addr,
//             deposits[1],
//         )?)
//     }
//
//     if let AssetInfo::NativeToken(denom) = assets[1].clone().info {
//         if token2_amount < max_token2 {
//             transfer_msgs.push(get_bank_transfer_to_msg(
//                 &info.sender,
//                 &denom,
//                 max_token2 - token2_amount,
//             ))
//         }
//     }
//
//     // provide_liquidity business logic unimplemented
//
//     Ok(Response::new())
// }
