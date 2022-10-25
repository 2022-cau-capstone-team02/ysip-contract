use cosmwasm_std::Empty;
use cw_multi_test::{Contract, ContractWrapper};
use token::{instantiate, execute, query};

pub fn mock_cw20_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(execute, instantiate, query);
    Box::new(contract)
}