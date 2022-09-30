use cosmwasm_std::{Addr, QuerierWrapper, QueryRequest, StdResult, WasmQuery};
use cw20::{Cw20QueryMsg, TokenInfoResponse};

pub fn query_token_symbol(querier: &QuerierWrapper, contract_addr: Addr) -> StdResult<String> {
    let res: TokenInfoResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: String::from(contract_addr),
        msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;

    Ok(res.symbol)
}