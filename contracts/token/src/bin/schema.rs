use std::env::current_dir;
use std::fs::create_dir_all;
use schemars::schema_for;
use cosmwasm_schema::{export_schema, remove_schemas};

use cw20_base::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};
use cw20::{
    BalanceResponse, Cw20ReceiveMsg, DownloadLogoResponse, MarketingInfoResponse, MinterResponse, TokenInfoResponse,
    AllowanceResponse, AllAllowancesResponse, AllAccountsResponse,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("contracts/token/schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(BalanceResponse), &out_dir);
    export_schema(&schema_for!(TokenInfoResponse), &out_dir);
    export_schema(&schema_for!(MinterResponse), &out_dir);
    export_schema(&schema_for!(AllowanceResponse), &out_dir);
    export_schema(&schema_for!(AllAllowancesResponse), &out_dir);
    export_schema(&schema_for!(AllAccountsResponse), &out_dir);
    export_schema(&schema_for!(MarketingInfoResponse), &out_dir);
    export_schema(&schema_for!(DownloadLogoResponse), &out_dir);
}