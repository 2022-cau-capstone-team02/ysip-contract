use cosmwasm_std::coin;
use cw20::{BalanceResponse, Cw20QueryMsg};
use testing::execute::{
    execute_mint, execute_provide_liquidity, execute_swap_coin_in, execute_swap_token_in,
    execute_transfer_from, increase_allowance,
};
use testing::init::{mock_cw20_contract, mock_pair_contract};
use testing::instantiate::{instantiate_cw20_contract, instantiate_pair_contract};
use testing::query::query_cw20_balance;
use testing_base::consts::{ADDR1, ADDR2};
use testing_base::init::init_app;

fn main() {
    let mut app = init_app(ADDR1);
    let channel_a_code_id = app.store_code(mock_cw20_contract());
    let pair_code_id = app.store_code(mock_pair_contract());

    let channel_a_contract_addr = instantiate_cw20_contract(
        &mut app,
        channel_a_code_id,
        &[],
        ADDR1,
        ADDR1,
        "channel_a",
        "channel-a",
        vec![],
        "channel_a",
    );

    let _mint_res = execute_mint(
        &mut app,
        channel_a_contract_addr.as_ref(),
        ADDR1,
        ADDR1,
        300000,
    );

    let pair_contract_addr = instantiate_pair_contract(
        &mut app,
        pair_code_id,
        channel_a_code_id,
        &[],
        ADDR1,
        ADDR2,
        channel_a_contract_addr.as_ref(),
        "ukrw",
        "pair",
    );

    let _increase_allowance_res = increase_allowance(
        &mut app,
        ADDR1,
        pair_contract_addr.as_ref(),
        channel_a_contract_addr.as_ref(),
        300000,
    );

    let token_balance_before_provide_liquidity = query_cw20_balance(
        &app,
        &channel_a_contract_addr,
        ADDR1
    );
    println!("token_balance_before_provide_liquidity: {}", token_balance_before_provide_liquidity);

    let coin_balance_before_provide_liquidity = app.wrap().query_balance(ADDR1, "ukrw").unwrap();
    println!("coin_balance_before_provide_liquidity: {}", coin_balance_before_provide_liquidity);

    let _liquidity_res = execute_provide_liquidity(
        &mut app,
        "ukrw",
        100000,
        channel_a_contract_addr.as_ref(),
        200000,
        pair_contract_addr.as_ref(),
        ADDR1,
    );

    let balance_after_provide_liquidity = query_cw20_balance(
        &app,
        &channel_a_contract_addr,
        ADDR1
    );
    println!("token_balance_after_provide_liquidity: {}", balance_after_provide_liquidity);

    let coin_balance_before_provide_liquidity = app.wrap().query_balance(ADDR1, "ukrw").unwrap();
    println!("coin_balance_after_provide_liquidity: {}", coin_balance_before_provide_liquidity);


    let swap_res = execute_swap_token_in(
        &mut app,
        pair_contract_addr.as_ref(),
        channel_a_contract_addr.as_ref(),
        ADDR1,
        30000,
    );

    swap_res.iter().for_each(|i| println!("{:?}", i));

    let token_balance_after_swap1 = query_cw20_balance(
        &app,
        &channel_a_contract_addr,
        ADDR1
    );
    println!("token_balance_after_swap1: {}", token_balance_after_swap1);

    let coin_balance_after_swap1 = app.wrap().query_balance(ADDR1, "ukrw").unwrap();
    println!("coin_balance_after_swap1: {}", coin_balance_after_swap1);

    let token_balance_after_swap1 = query_cw20_balance(
        &app,
        &channel_a_contract_addr,
        ADDR2
    );
    println!("token_balance_of_admin_after_swap1: {}", token_balance_after_swap1);

    let coin_balance_after_swap1 = app.wrap().query_balance(ADDR2, "ukrw").unwrap();
    println!("coin_balance_of_admin_after_swap1: {}", coin_balance_after_swap1);


}
