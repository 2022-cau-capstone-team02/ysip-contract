use cosmwasm_std::{Attribute, Uint128};
use cw20::Cw20ExecuteMsg;
use cw_multi_test::BasicApp;
use testing_base::execute::execute_contract;

pub fn execute_mint(
    app: &mut BasicApp,
    contract_addr: &str,
    admin: &str,
    recipient: &str,
    amount: u128,
) -> Vec<Attribute> {
    let mint_msg = Cw20ExecuteMsg::Mint {
        recipient: recipient.to_string(),
        amount: Uint128::new(amount)
    };

    execute_contract(
        app,
        contract_addr,
        &mint_msg,
        &[],
        admin
    ).unwrap()
}