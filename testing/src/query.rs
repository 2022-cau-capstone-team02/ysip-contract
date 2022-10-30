use cosmwasm_std::{Addr, Uint128};
use cw20::{BalanceResponse, Cw20QueryMsg};
use cw_multi_test::BasicApp;

pub fn query_cw20_balance(
    app: &BasicApp,
    contract_addr: &Addr,
    addr: &str,
) -> Uint128 {
    let balance_response: BalanceResponse = app.wrap().query_wasm_smart(
        contract_addr,
        &Cw20QueryMsg::Balance {
            address: addr.to_string()
        }
    ).unwrap();

    balance_response.balance
}