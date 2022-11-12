use cosmwasm_std::{Addr, Coin};
use cw20::{Cw20Coin, MinterResponse};
use cw20_base::msg::InstantiateMsg;
use cw_multi_test::BasicApp;
use testing_base::instantiate::instantiate_contract;
use ysip::asset::AssetInfo;
use ysip::pair::InstantiateMsg as PairInstantiateMsg;

pub fn instantiate_cw20_contract(
    app: &mut BasicApp,
    code_id: u64,
    funds: &[Coin],
    sender: &str,
    admin: &str,
    name: &str,
    symbol: &str,
    initial_balances: Vec<Cw20Coin>,
    label: &str,
) -> Addr {
    let cw20_init_msg = InstantiateMsg {
        name: name.to_string(),
        symbol: symbol.to_string(),
        decimals: 6,
        initial_balances,
        mint: Some(MinterResponse {
            minter: admin.to_string(),
            cap: None,
        }),
        marketing: None,
    };
    instantiate_contract(app, cw20_init_msg, funds, code_id, sender, admin, label)
}

pub fn instantiate_pair_contract(
    app: &mut BasicApp,
    pair_code_id: u64,
    token_code_id: u64,
    funds: &[Coin],
    sender: &str,
    admin: &str,
    token_addr: &Addr,
    native_token_denom: &str,
    label: &str,
) -> Addr {
    let pair_init_msg = PairInstantiateMsg {
        asset_infos: [
            AssetInfo::Token {
                contract_addr: Addr::unchecked(token_addr),
            },
            AssetInfo::NativeToken {
                denom: native_token_denom.to_string(),
            },
        ],
        token_code_id,
        protocol_fee_recipient: admin.to_string(),
        protocol_fee_percent: "0.15".to_string(),
        lp_fee_percent: "0.15".to_string(),
    };

    instantiate_contract(
        app,
        pair_init_msg,
        funds,
        pair_code_id,
        sender,
        admin,
        label,
    )
}
