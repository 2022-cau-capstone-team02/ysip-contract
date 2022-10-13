use crate::state::{FEE_DECIMAL_PRECISION, FEE_SCALE_FACTOR};
use cosmwasm_std::{Decimal, StdError, StdResult, Uint128};

pub fn fee_decimal_to_uint128(fee_percent: Decimal) -> StdResult<Uint128> {
    let result = fee_percent
        .atomics()
        .checked_mul(FEE_SCALE_FACTOR)
        .map_err(StdError::overflow)?;

    Ok(result / FEE_DECIMAL_PRECISION)
}

