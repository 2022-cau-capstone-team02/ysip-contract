use cosmwasm_std::{Addr, Coin, CosmosMsg, Uint128};

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