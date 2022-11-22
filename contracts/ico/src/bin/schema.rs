use cosmwasm_schema::{export_schema, remove_schemas};
use ico::msg::{
    ExecuteMsg, FundingAmountResponse, InstantiateMsg, IsFundingFinishedResponse,
    PairAddressResponse, QueryMsg, TokenAddressResponse,
};
use schemars::schema_for;
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("contracts/ico/schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(FundingAmountResponse), &out_dir);
    export_schema(&schema_for!(IsFundingFinishedResponse), &out_dir);
    export_schema(&schema_for!(TokenAddressResponse), &out_dir);
    export_schema(&schema_for!(PairAddressResponse), &out_dir);
}
