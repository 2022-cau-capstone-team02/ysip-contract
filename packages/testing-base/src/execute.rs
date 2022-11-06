use anyhow::Error;
use cosmwasm_std::{Addr, Attribute, Coin};
use cw_multi_test::{BasicApp, Executor};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

pub fn execute_contract<T>(
    app: &mut BasicApp,
    contract_addr: &Addr,
    msg: &T,
    send_funds: &[Coin],
    sender: &str,
) -> Result<Vec<Attribute>, Error>
where
    T: Serialize + DeserializeOwned + Clone + Debug,
{
    let execute_res = app.execute_contract(
        Addr::unchecked(sender),
        contract_addr.clone(),
        &msg,
        send_funds,
    );
    match execute_res {
        Ok(res) => Ok(res.events[1].clone().attributes),
        Err(err) => Err(err),
    }
}
