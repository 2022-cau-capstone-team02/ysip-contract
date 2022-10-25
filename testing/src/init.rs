use cosmwasm_std::Empty;
use cw_multi_test::{Contract, ContractWrapper};

pub fn mock_cw20_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(token::execute, token::instantiate, token::query);
    Box::new(contract)
}

pub fn mock_pair_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(pair::contract::execute, pair::contract::instantiate, pair::contract::query).with_reply(pair::contract::reply);
    Box::new(contract)
}