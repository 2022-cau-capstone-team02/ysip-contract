use cosmwasm_std::{Addr, Coin};
use cw_multi_test::{BasicApp, Executor};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn instantiate_contract<T>(
    app: &mut BasicApp,
    msg: T,
    funds: &[Coin],
    code_id: u64,
    sender: &str,
    admin: &str,
    label: &str,
) -> Addr
where
    T: Serialize + DeserializeOwned + Clone,
{
    app.instantiate_contract(
        code_id,
        Addr::unchecked(sender),
        &msg,
        funds,
        label,
        Some(admin.to_string()),
    )
    .unwrap()
}
