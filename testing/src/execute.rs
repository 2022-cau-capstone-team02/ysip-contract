use cosmwasm_std::{Addr, Attribute, Coin, coin, to_binary, Uint128};
use cw20::{Cw20ExecuteMsg};
use cw20::Cw20ReceiveMsg;
use cw_multi_test::BasicApp;
use testing_base::execute::execute_contract;
use ysip::asset::{Asset, AssetInfo};
use ysip::pair::{Cw20HookMsg, ExecuteMsg};

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

    execute_contract(
        app,
        contract_addr,
        &mint_msg,
        &[],
        admin,
    ).unwrap()
}

pub fn execute_provide_liquidity(
    app: &mut BasicApp,
    native_token_denom: &str,
    token_contract_addr: &str,
    sender: &str,
) -> Vec<Attribute> {
    let provide_liquidity_msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::Token {
                    contract_addr: Addr::unchecked(token_contract_addr)
                },
                amount: Uint128::new(10),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: native_token_denom.to_string()
                },
                amount: Uint128::new(5)
            }
        ],
    };


}

pub fn execute_swap(
    app: &mut BasicApp,
    contract_addr: &str,
    sender: &str,
    swap_amount_in: u128,
) -> Vec<Attribute> {
    let receive_msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: sender.to_string(),
        amount: Uint128::new(swap_amount_in),
        msg: to_binary(&Cw20HookMsg::Swap {
            min_output_amount: Some("0".to_string()),
            max_spread: Some("100".to_string()),
            to: Some(sender.to_string()),
        }).unwrap(),
    });

    execute_contract(
        app,
        contract_addr,
        &receive_msg,
        &[coin(700, "ukrw")],
        sender,
    ).unwrap()
}