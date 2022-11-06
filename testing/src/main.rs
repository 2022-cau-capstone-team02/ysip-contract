use cosmwasm_std::Uint128;
use testing::execute::{execute_mint, execute_provide_liquidity, execute_remove_liquidity, execute_swap_token_in, increase_allowance};
use testing::init::{mock_cw20_contract, mock_pair_contract};
use testing::instantiate::{instantiate_cw20_contract, instantiate_pair_contract};
use testing::query::{query_cw20_balance, query_pair_info};
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
        &channel_a_contract_addr,
        ADDR1,
        ADDR1,
        500000,
    );

    let pair_contract_addr = instantiate_pair_contract(
        &mut app,
        pair_code_id,
        channel_a_code_id,
        &[],
        ADDR1,
        ADDR2,
        &channel_a_contract_addr,
        "ukrw",
        "pair",
    );

    let _increase_allowance_res = increase_allowance(
        &mut app,
        ADDR1,
        &pair_contract_addr,
        &channel_a_contract_addr,
        500000,
    );

    let token_balance_before_provide_liquidity =
        query_cw20_balance(&app, &channel_a_contract_addr, ADDR1);
    println!(
        "token_balance_before_provide_liquidity: {}",
        token_balance_before_provide_liquidity
    );

    let coin_balance_before_provide_liquidity = app.wrap().query_balance(ADDR1, "ukrw").unwrap();
    println!(
        "coin_balance_before_provide_liquidity: {}",
        coin_balance_before_provide_liquidity
    );

    let _liquidity_res = execute_provide_liquidity(
        &mut app,
        "ukrw",
        100000,
        &channel_a_contract_addr,
        200000,
        &pair_contract_addr,
        ADDR1,
    );

    let balance_after_provide_liquidity = query_cw20_balance(&app, &channel_a_contract_addr, ADDR1);
    println!(
        "token_balance_after_provide_liquidity: {}",
        balance_after_provide_liquidity
    );

    let coin_balance_before_provide_liquidity = app.wrap().query_balance(ADDR1, "ukrw").unwrap();
    println!(
        "coin_balance_after_provide_liquidity: {}",
        coin_balance_before_provide_liquidity
    );

    let swap_res = execute_swap_token_in(
        &mut app,
        &pair_contract_addr,
        &channel_a_contract_addr,
        ADDR1,
        30000,
    );

    swap_res.iter().for_each(|i| println!("{:?}", i));

    let token_balance_after_swap1 = query_cw20_balance(&app, &channel_a_contract_addr, ADDR1);
    println!("token_balance_after_swap1: {}", token_balance_after_swap1);

    let coin_balance_after_swap1 = app.wrap().query_balance(ADDR1, "ukrw").unwrap();
    println!("coin_balance_after_swap1: {}", coin_balance_after_swap1);

    let token_balance_after_swap1 = query_cw20_balance(&app, &channel_a_contract_addr, ADDR2);
    println!(
        "token_balance_of_admin_after_swap1: {}",
        token_balance_after_swap1
    );

    let coin_balance_after_swap1 = app.wrap().query_balance(ADDR2, "ukrw").unwrap();
    println!(
        "coin_balance_of_admin_after_swap1: {}",
        coin_balance_after_swap1
    );

    let pair_info_response =  query_pair_info(&app, &pair_contract_addr);

    increase_allowance(
        &mut app,
        ADDR1,
        &pair_contract_addr,
        &pair_info_response.liquidity_token,
        500000,
    );

    let a= query_cw20_balance(&app, &pair_info_response.liquidity_token, ADDR1);
    println!("{}", a);

    let remove_liquidity_res = execute_remove_liquidity(&mut app, &pair_contract_addr, ADDR1, Uint128::new(100000));
    remove_liquidity_res.iter().for_each(|a| println!("{:?}", a));

}
