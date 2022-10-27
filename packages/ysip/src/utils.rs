use crate::asset::{Asset, AssetInfo};
use cosmwasm_std::{to_binary, Addr, Coin, CosmosMsg, StdResult, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

pub fn get_cw20_transfer_from_msg(
    owner: &Addr,
    recipient: &Addr,
    token_addr: &Addr,
    token_amount: Uint128,
) -> StdResult<CosmosMsg> {
    // create transfer cw20 msg
    let transfer_cw20_msg = Cw20ExecuteMsg::TransferFrom {
        owner: owner.into(),
        recipient: recipient.into(),
        amount: token_amount,
    };
    let exec_cw20_transfer = WasmMsg::Execute {
        contract_addr: token_addr.into(),
        msg: to_binary(&transfer_cw20_msg)?,
        funds: vec![],
    };

    Ok(exec_cw20_transfer.into())
}

pub fn get_bank_transfer_to_msg(
    recipient: &Addr,
    denom: &str,
    native_amount: Uint128,
) -> CosmosMsg {
    let transfer_bank_msg = cosmwasm_std::BankMsg::Send {
        to_address: recipient.to_string(),
        amount: vec![Coin {
            denom: denom.to_string(),
            amount: native_amount,
        }],
    };

    transfer_bank_msg.into()
}

pub fn get_cw20_mint_msg(
    recipient: &Addr,
    amount: Uint128,
    contract_addr: &Addr,
) -> StdResult<CosmosMsg> {
    let mint_msg = Cw20ExecuteMsg::Mint {
        recipient: recipient.to_string(),
        amount,
    };

    Ok(WasmMsg::Execute {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    }
    .into())
}

pub fn get_fee_transfer_msg(sender: &Addr, recipient: &Addr, fee: Asset) -> StdResult<CosmosMsg> {
    match fee.info {
        AssetInfo::Token { contract_addr } => {
            get_cw20_transfer_from_msg(sender, recipient, &contract_addr, fee.amount)
        }
        AssetInfo::NativeToken { denom } => {
            Ok(get_bank_transfer_to_msg(recipient, &denom, fee.amount))
        }
    }
}
