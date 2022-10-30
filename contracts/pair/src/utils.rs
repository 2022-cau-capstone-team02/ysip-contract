use crate::state::{FEE_DECIMAL_PRECISION, FEE_SCALE_FACTOR};
use cosmwasm_std::{to_binary, Coin, CosmosMsg, Decimal, StdError, StdResult, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use ysip::asset::Asset;
use ysip::pair::ExecuteMsg;

pub fn fee_decimal_to_uint128(fee_percent: Decimal) -> StdResult<Uint128> {
    let result = fee_percent
        .atomics()
        .checked_mul(FEE_SCALE_FACTOR)
        .map_err(StdError::overflow)?;

    Ok(result / FEE_DECIMAL_PRECISION)
}

pub fn get_provide_liquidity_msg(
    offer_pool: Asset,
    input_token_fee_amount: Uint128,
    ask_pool: Asset,
    output_token_fee_amount: Uint128,
    contract_addr: &str,
    funds: Vec<Coin>,
) -> StdResult<CosmosMsg> {
    let provide_liquidity_msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: offer_pool.info,
                amount: input_token_fee_amount,
            },
            Asset {
                info: ask_pool.info,
                amount: output_token_fee_amount,
            },
        ],
    };

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&provide_liquidity_msg)?,
        funds,
    }))
}

pub fn get_increase_allowance_msg(
    contract_addr: &str,
    spender: &str,
    amount: Uint128,
) -> StdResult<CosmosMsg> {
    let increae_allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: spender.to_string(),
        amount,
        expires: None,
    };

    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&increae_allowance_msg)?,
        funds: vec![],
    }))
}
