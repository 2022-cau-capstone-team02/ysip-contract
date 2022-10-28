use cosmwasm_std::{coin, to_binary, Addr, Attribute, Coin, CosmosMsg, Uint128};
use cw20::Cw20ExecuteMsg;
use cw20::Cw20ReceiveMsg;
use cw_multi_test::BasicApp;
use testing_base::execute::execute_contract;
use ysip::asset::{Asset, AssetInfo};
use ysip::pair::{Cw20HookMsg, ExecuteMsg};
use ysip::querier::query_token_balance;

pub fn execute_mint(
    app: &mut BasicApp,
    contract_addr: &str,
    admin: &str,
    recipient: &str,
    amount: u128,
) -> Vec<Attribute> {
    let mint_msg = Cw20ExecuteMsg::Mint {
        recipient: recipient.to_string(),
        amount: Uint128::new(amount),
    };

    execute_contract(app, contract_addr, &mint_msg, &[], admin).unwrap()
}

pub fn increase_allowance(
    app: &mut BasicApp,
    owner: &str,
    pair_addr: &str,
    token_addr: &str,
    amount: u128,
) -> Vec<Attribute> {
    let increase_allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
        spender: pair_addr.to_string(),
        amount: Uint128::new(amount),
        expires: None,
    };

    execute_contract(app, token_addr, &increase_allowance_msg, &[], owner).unwrap()
}

pub fn execute_transfer_from(
    app: &mut BasicApp,
    owner: &str,
    recipient: &str,
    token_addr: &str,
    amount: u128,
) -> Vec<Attribute> {
    let transfer_from_msg = Cw20ExecuteMsg::TransferFrom {
        owner: owner.to_string(),
        recipient: recipient.to_string(),
        amount: Uint128::new(amount),
    };

    execute_contract(app, token_addr, &transfer_from_msg, &[], owner).unwrap()
}

pub fn execute_provide_liquidity(
    app: &mut BasicApp,
    native_token_denom: &str,
    native_token_amount: u128,
    token_contract_addr: &str,
    token_amount: u128,
    pair_contract_addr: &str,
    sender: &str,
) -> Vec<Attribute> {
    let provide_liquidity_msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::Token {
                    contract_addr: Addr::unchecked(token_contract_addr),
                },
                amount: Uint128::new(token_amount),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: native_token_denom.to_string(),
                },
                amount: Uint128::new(native_token_amount),
            },
        ],
    };

    execute_contract(
        app,
        pair_contract_addr,
        &provide_liquidity_msg,
        &[coin(native_token_amount, native_token_denom)],
        sender,
    )
    .unwrap()
}

pub fn execute_swap_token_in(
    app: &mut BasicApp,
    contract_addr: &str,
    token_addr: &str,
    sender: &str,
    swap_amount_in: u128,
) -> Vec<Attribute> {
    let swap_msg = ExecuteMsg::Swap {
        offer_asset: Asset {
            info: AssetInfo::Token {
                contract_addr: Addr::unchecked(token_addr),
            },
            amount: Uint128::new(swap_amount_in),
        },
        min_output_amount: Some(String::from("50")),
        max_spread: Some(String::from("100")),
        to: Some(String::from(sender)),
    };

    execute_contract(app, contract_addr, &swap_msg, &[], sender).unwrap()
}

pub fn execute_swap_coin_in(
    app: &mut BasicApp,
    contract_addr: &str,
    sender: &str,
    swap_amount_in: u128,
) -> Vec<Attribute> {
    let swap_msg = ExecuteMsg::Swap {
        offer_asset: Asset {
            info: AssetInfo::NativeToken {
                denom: "ukrw".to_string(),
            },
            amount: Uint128::new(swap_amount_in),
        },
        min_output_amount: Some(String::from("100")),
        max_spread: Some(String::from("5")),
        to: Some(String::from(sender)),
    };

    execute_contract(
        app,
        contract_addr,
        &swap_msg,
        &[coin(swap_amount_in, "ukrw")],
        sender,
    )
    .unwrap()
}
