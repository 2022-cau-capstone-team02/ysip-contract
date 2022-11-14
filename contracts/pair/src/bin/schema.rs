use std::env::current_dir;
use std::fs::create_dir_all;
use schemars::schema_for;
use cosmwasm_schema::{export_schema, remove_schemas};

use ysip::pair::{InstantiateMsg, ExecuteMsg, QueryMsg, PairInfoResponse, LiquidityResponse};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("contracts/pair/schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(PairInfoResponse), &out_dir);
    export_schema(&schema_for!(LiquidityResponse), &out_dir);
}