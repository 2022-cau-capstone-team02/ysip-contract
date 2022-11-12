use crate::error::ContractError;
use crate::math::{get_lp_fee_amount, get_protocol_fee_amount, get_swap_output_amount};
use crate::state::{Config, Fees, Liquidity, CONFIG, LIQUIDITY};
use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Reply, ReplyOn, Response, StdError, StdResult, SubMsg, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::MinterResponse;
use cw20_base::msg::InstantiateMsg as Cw20InstantiateMsg;
use std::str::FromStr;
use ysip::asset::{format_lp_token_name, Asset, AssetInfo};
use ysip::pair::{
    ExecuteMsg, InstantiateMsg, LiquidityResponse, PairInfo, PairInfoResponse, QueryMsg, SwapParams,
};
use ysip::querier::{query_lp_token_supply, query_token_balance};
use ysip::utils::{
    get_bank_transfer_to_msg, get_burn_from_msg, get_cw20_mint_msg, get_cw20_transfer_from_msg,
    get_cw20_transfer_msg, get_fee_transfer_msg,
};

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
                name: lp_token_name.clone(),
                symbol: "uLp".to_string(),
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
        ExecuteMsg::ProvideLiquidity { assets } => {
            execute_provide_liquidity(deps, env, info, assets)
        }
        ExecuteMsg::Swap {
            offer_asset,
            min_output_amount,
            max_spread,
            to,
        } => execute_swap(
            deps,
            env,
            info,
            offer_asset,
            min_output_amount,
            max_spread,
            to,
        ),
        ExecuteMsg::RemoveLiquidity { amount } => execute_remove_liquidity(deps, env, info, amount),
    }
}

fn get_reserve(deps: Deps, assets: [Asset; 2]) -> StdResult<[Asset; 2]> {
    let reserve = LIQUIDITY.load(deps.storage)?;

    let token1_reserve = [&reserve.token_a, &reserve.token_b]
        .iter()
        .find(|i| i.info.eq(&assets[0].info))
        .expect("reserve not found")
        .clone();

    let token2_reserve = [&reserve.token_a, &reserve.token_b]
        .iter()
        .find(|i| i.info.eq(&assets[1].info))
        .expect("reserve not found")
        .clone();

    Ok([token1_reserve.clone(), token2_reserve.clone()])
}

fn execute_swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    offer_asset: Asset,
    min_output_amount: Option<String>,
    max_spread: Option<String>,
    to: Option<String>,
) -> Result<Response, ContractError> {
    let to_addr = if let Some(to_addr) = to {
        Some(deps.api.addr_validate(to_addr.as_str())?)
    } else {
        None
    };

    swap(
        deps,
        env,
        info,
        SwapParams {
            offer_asset,
            min_output_amount,
            max_spread,
            to: to_addr,
        },
    )
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
        .query_pools(&deps.querier, &env.contract.address)?;

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

    let reserves = get_reserve(deps.as_ref(), pools.clone())?;

    let offer_pool_reserve = reserves
        .iter()
        .find(|a| a.info == offer_pool.info)
        .expect("reserve not found");

    let ask_pool_reserve = reserves
        .iter()
        .find(|a| a.info == ask_pool.info)
        .expect("reserve not found");

    let fees = config.fees;

    let protocol_fee_amount =
        get_protocol_fee_amount(params.offer_asset.amount, fees.protocol_fee_percent)?;

    let token_bought_amount = get_swap_output_amount(
        params.offer_asset.amount - protocol_fee_amount,
        offer_pool_reserve.amount,
        ask_pool_reserve.amount,
        fees.lp_fee_percent,
    )?;

    let (input_token_fee_amount, output_token_fee_amount) = get_lp_fee_amount(
        params.offer_asset.amount - protocol_fee_amount,
        token_bought_amount,
        fees.lp_fee_percent,
    )?;

    // amount of token into the pool
    let net_input_amount = params.offer_asset.amount - protocol_fee_amount - input_token_fee_amount;

    //amount of token out of the pool
    let net_token_output_amount = token_bought_amount - output_token_fee_amount;

    params.assert_min_token_bought(net_token_output_amount)?;

    let mut msgs = vec![];

    // transfer token from owner to pair contract
    if let AssetInfo::Token { contract_addr } = offer_pool.info.clone() {
        let msg = get_cw20_transfer_from_msg(
            &info.sender,
            &env.contract.address,
            &contract_addr,
            params.offer_asset.amount,
        )?;
        msgs.push(msg)
    }

    // send output token or coin to sender
    match ask_pool.info.clone() {
        AssetInfo::Token { contract_addr } => msgs.push(get_cw20_transfer_msg(
            &info.sender,
            &contract_addr,
            net_token_output_amount,
        )?),
        AssetInfo::NativeToken { denom } => msgs.push(get_bank_transfer_to_msg(
            &info.sender.clone(),
            &denom,
            net_token_output_amount,
        )),
    };

    msgs.push(get_fee_transfer_msg(
        &fees.protocol_fee_recipient,
        Asset {
            info: offer_pool.info.clone(),
            amount: protocol_fee_amount,
        },
    )?);

    println!(
        "liquidity before swap: {:?}",
        LIQUIDITY.load(deps.storage).unwrap()
    );

    LIQUIDITY.update(deps.storage, |mut liquidity| -> Result<_, ContractError> {
        if liquidity.token_a.info == offer_pool.info && liquidity.token_b.info == ask_pool.info {
            liquidity.token_a.amount = liquidity
                .token_a
                .amount
                .checked_add(net_input_amount + input_token_fee_amount)
                .map_err(StdError::overflow)?;
            liquidity.token_b.amount = liquidity
                .token_b
                .amount
                .checked_sub(net_token_output_amount)
                .map_err(StdError::overflow)?;
        } else if liquidity.token_b.info == offer_pool.info
            && liquidity.token_a.info == ask_pool.info
        {
            liquidity.token_b.amount = liquidity
                .token_b
                .amount
                .checked_add(net_input_amount + input_token_fee_amount)
                .map_err(StdError::overflow)?;
            liquidity.token_a.amount = liquidity
                .token_a
                .amount
                .checked_sub(net_token_output_amount)
                .map_err(StdError::overflow)?;
        }
        Ok(liquidity)
    })?;

    println!(
        "liquidity after swap: {:?}",
        LIQUIDITY.load(deps.storage).unwrap()
    );

    Ok(Response::new()
        .add_attributes(vec![
            attr("action", "swap"),
            attr("token_in_amount", params.offer_asset.amount),
            attr("token_out_amount", net_token_output_amount),
            attr("protocol_fee_amount", protocol_fee_amount),
            attr("protocol_fee_recipient", &fees.protocol_fee_recipient),
            attr("input_token_fee_amount", input_token_fee_amount),
            attr("output_token_fee_amount", output_token_fee_amount),
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
        let temp_token2_amount = token1_amount
            .checked_mul(token2_reserve)
            .map_err(StdError::overflow)?
            .checked_div(token1_reserve)
            .map_err(StdError::divide_by_zero)?;

        if temp_token2_amount.eq(&Uint128::zero()) {
            Ok(Uint128::one())
        } else {
            Ok(temp_token2_amount)
        }
    }
}

fn execute_provide_liquidity(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assets: [Asset; 2],
) -> Result<Response, ContractError> {
    assets[0].info.check_is_valid(deps.api)?;
    assets[1].info.check_is_valid(deps.api)?;

    for asset in &assets {
        asset.assert_sent_native_token_balance(&info)?;
    }

    let config = CONFIG.load(deps.storage)?;

    let pools: [Asset; 2] = config
        .pair_info
        .query_pools(&deps.querier, &env.contract.address)?;

    let deposits: [Uint128; 2] = [
        assets
            .iter()
            .find(|a| a.info.eq(&pools[0].info))
            .map(|a| a.amount)
            .expect("Wrong asset info is given"),
        assets
            .iter()
            .find(|a| a.info.eq(&pools[1].info))
            .map(|a| a.amount)
            .expect("Wrong asset info is given"),
    ];

    if deposits[0].is_zero() && deposits[1].is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    let [token1_reserve, token2_reserve] = get_reserve(deps.as_ref(), assets.clone())?;

    let lp_token_supply =
        query_lp_token_supply(&deps.as_ref().querier, &config.pair_info.liquidity_token)?;

    let liquidity_amount =
        get_lp_token_amount_to_mint(deposits[0], lp_token_supply, token1_reserve.amount)?;

    let token2_amount = get_token2_amount_required(
        deposits[1],
        deposits[0],
        lp_token_supply,
        token2_reserve.amount,
        token1_reserve.amount,
    )?;

    if deposits[1] < token2_amount {
        return Err(ContractError::NotEnoughTokenAmount {
            need: token2_amount,
            supplied: deposits[1],
        });
    }

    let mut transfer_msgs: Vec<CosmosMsg> = vec![];
    if let AssetInfo::Token { contract_addr } = assets[0].clone().info {
        transfer_msgs.push(get_cw20_transfer_from_msg(
            &info.sender,
            &env.contract.address,
            &contract_addr,
            deposits[0],
        )?)
    }

    if let AssetInfo::Token { contract_addr } = assets[1].clone().info {
        transfer_msgs.push(get_cw20_transfer_from_msg(
            &info.sender,
            &env.contract.address,
            &contract_addr,
            token2_amount,
        )?)
    }

    // refund needed
    if deposits[1] > token2_amount {
        if let AssetInfo::NativeToken { denom } = assets[1].clone().info {
            transfer_msgs.push(get_bank_transfer_to_msg(
                &info.sender,
                &denom,
                deposits[1] - token2_amount,
            ))
        }
    }

    LIQUIDITY.update(deps.storage, |mut liq| -> Result<_, ContractError> {
        assets
            .iter()
            .find(|a| a.info.eq(&liq.token_a.info))
            .map(|_| {
                liq.token_a.amount = liq
                    .token_a
                    .amount
                    .checked_add(deposits[0])
                    .expect("overflow");
            });

        assets
            .iter()
            .find(|a| a.info.eq(&liq.token_b.info))
            .map(|_| {
                liq.token_b.amount = liq
                    .token_b
                    .amount
                    .checked_add(token2_amount)
                    .expect("overflow");
            });

        Ok(liq)
    })?;

    let mint_lp_tokens_msg = get_cw20_mint_msg(
        &info.sender,
        liquidity_amount,
        &config.pair_info.liquidity_token,
    )?;

    Ok(Response::new()
        .add_attribute("action", "provide_liquidity")
        .add_attribute("token_1_amount", deposits[0])
        .add_attribute("token_2_amount", token2_amount)
        .add_messages(transfer_msgs)
        .add_message(mint_lp_tokens_msg))
}

fn execute_remove_liquidity(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let lp_token_addr = config.pair_info.liquidity_token;
    let lp_token_balance = query_token_balance(&deps.querier, &lp_token_addr, &info.sender)?;
    let lp_token_supply = query_lp_token_supply(&deps.querier, &lp_token_addr)?;

    if amount > lp_token_balance {
        return Err(ContractError::NotEnoughBalance {
            avaiable: lp_token_balance,
            requested: amount,
        });
    }

    let liquidity = LIQUIDITY.load(deps.storage)?;

    let token1_amount = amount
        .checked_mul(liquidity.token_a.amount)
        .map_err(StdError::overflow)?
        .checked_div(lp_token_supply)
        .map_err(StdError::divide_by_zero)?;

    let token2_amount = amount
        .checked_mul(liquidity.token_b.amount)
        .map_err(StdError::overflow)?
        .checked_div(lp_token_supply)
        .map_err(StdError::divide_by_zero)?;

    LIQUIDITY.update(deps.storage, |mut liquidity| -> Result<_, ContractError> {
        liquidity.token_a.amount = liquidity
            .token_a
            .amount
            .checked_sub(token1_amount)
            .map_err(StdError::overflow)?;

        liquidity.token_b.amount = liquidity
            .token_b
            .amount
            .checked_sub(token2_amount)
            .map_err(StdError::overflow)?;

        Ok(liquidity)
    })?;

    let token1_transfer_msg = match liquidity.token_a.info {
        AssetInfo::Token { contract_addr } => {
            get_cw20_transfer_msg(&info.sender, &contract_addr, token1_amount)?
        }
        AssetInfo::NativeToken { denom } => {
            get_bank_transfer_to_msg(&info.sender, &denom, token1_amount)
        }
    };

    let token2_transfer_msg = match liquidity.token_b.info {
        AssetInfo::Token { contract_addr } => {
            get_cw20_transfer_msg(&info.sender, &contract_addr, token2_amount)?
        }
        AssetInfo::NativeToken { denom } => {
            get_bank_transfer_to_msg(&info.sender, &denom, token2_amount)
        }
    };

    let lp_token_burn_msg = get_burn_from_msg(&lp_token_addr, &info.sender, amount)?;

    Ok(Response::new()
        .add_message(token1_transfer_msg)
        .add_message(token2_transfer_msg)
        .add_message(lp_token_burn_msg)
        .add_attribute("liquidity_burned", amount)
        .add_attribute("token1_returned", token1_amount)
        .add_attribute("token2_returned", token2_amount))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::PairInfo {} => query_pair_info(deps),
        QueryMsg::Liquidity {} => query_liquidity(deps),
    }
}

fn query_pair_info(deps: Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;
    let res = PairInfoResponse {
        assets: config.pair_info.asset_infos,
        contract_addr: config.pair_info.contract_addr,
        liquidity_token: config.pair_info.liquidity_token,
    };

    Ok(to_binary(&res)?)
}

fn query_liquidity(deps: Deps) -> StdResult<Binary> {
    let liquidity = LIQUIDITY.load(deps.storage)?;
    let res = LiquidityResponse {
        liquidity: [liquidity.token_a, liquidity.token_b],
    };

    Ok(to_binary(&res)?)
}
