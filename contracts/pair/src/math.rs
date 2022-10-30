use crate::state::FEE_SCALE_FACTOR;
use crate::utils::fee_decimal_to_uint128;
use cosmwasm_std::{Decimal, StdError, StdResult, Uint128, Uint256};

pub fn get_swap_output_amount(
    input_amount: Uint128,
    input_reserve: Uint128,
    output_reserve: Uint128,
    fee_percent: Decimal,
) -> StdResult<Uint128> {
    if input_reserve == Uint128::zero() || output_reserve == Uint128::zero() {
        return Err(StdError::generic_err("No liquidity"));
    };

    let k = input_reserve * output_reserve;

    // 1% -> 100
    let fee = fee_decimal_to_uint128(fee_percent)?;
    let net_input_percent = FEE_SCALE_FACTOR - fee;
    let net_input_amount = input_amount
        .checked_multiply_ratio(net_input_percent, FEE_SCALE_FACTOR)
        .map_err(|_| StdError::generic_err("multiply ratio error"))?;
    let input_reserve_after_swap = input_reserve + net_input_amount;
    let output_reserve_after_swap = k / input_reserve_after_swap;
    let net_output_amount = output_reserve - output_reserve_after_swap;

    Ok(net_output_amount)
}

pub fn get_protocol_fee_amount(input_amount: Uint128, fee_percent: Decimal) -> StdResult<Uint128> {
    if fee_percent.is_zero() {
        return Ok(Uint128::zero());
    }

    let fee_percent = fee_decimal_to_uint128(fee_percent)?;

    Ok(input_amount
        .full_mul(fee_percent)
        .checked_div(Uint256::from(FEE_SCALE_FACTOR))?
        .try_into()?)
}

pub fn get_lp_fee_amount(
    input_token_amount: Uint128,
    output_token_amount: Uint128,
    lp_fee_percent: Decimal,
) -> StdResult<(Uint128, Uint128)> {
    if lp_fee_percent.is_zero() {
        return Ok((Uint128::zero(), Uint128::zero()));
    }

    let fee_percent = fee_decimal_to_uint128(lp_fee_percent)?;
    let input_token_fee_amount = input_token_amount.multiply_ratio(fee_percent, FEE_SCALE_FACTOR);
    let output_token_fee_amount = output_token_amount.multiply_ratio(fee_percent, FEE_SCALE_FACTOR);

    Ok((input_token_fee_amount, output_token_fee_amount))
}

#[cfg(test)]
mod test_input_price {
    use crate::math::get_swap_output_amount;
    use cosmwasm_std::{Decimal, Uint128};
    use std::str::FromStr;

    const EXP: u128 = 1000000;

    #[test]
    fn test_swap_1() {
        let pool_x_reserve = Uint128::new(100 * EXP);
        let pool_y_reserve = Uint128::new(3000 * EXP);
        let input_x = Uint128::new(10 * EXP);
        let res = get_swap_output_amount(
            input_x,
            pool_x_reserve,
            pool_y_reserve,
            Decimal::from_str("0.3").unwrap(),
        )
        .unwrap();
        assert_eq!(res.u128(), 271983269);
    }

    #[test]
    fn test_swap_2() {
        let pool_x_reserve = Uint128::new(100 * EXP);
        let pool_y_reserve = Uint128::new(4000 * EXP);
        let input_x = Uint128::new(20 * EXP);
        let res = get_swap_output_amount(
            input_x,
            pool_x_reserve,
            pool_y_reserve,
            Decimal::from_str("0.3").unwrap(),
        )
        .unwrap();
        assert_eq!(res.u128(), 664999167);
    }

    #[test]
    fn test_swap_3() {
        let pool_x_reserve = Uint128::new(100 * EXP);
        let pool_y_reserve = Uint128::new(5000 * EXP);
        let input_x = Uint128::new(40 * EXP);
        let res = get_swap_output_amount(
            input_x,
            pool_x_reserve,
            pool_y_reserve,
            Decimal::from_str("0.3").unwrap(),
        )
        .unwrap();
        assert_eq!(res.u128(), 1425507578);
    }
}
