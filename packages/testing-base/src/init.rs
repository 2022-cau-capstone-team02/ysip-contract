use cosmwasm_std::{coin, Addr, Empty};
use cw_multi_test::{custom_app, BasicApp};

pub fn init_app(admin: &str) -> BasicApp {
    let init_funds = vec![coin(30000, "ukrw")];
    custom_app::<Empty, Empty, _>(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &Addr::unchecked(admin), init_funds)
            .unwrap();
    })
}

// fn next_block(block: &mut BlockInfo) {
//     block.time = block.time.plus_seconds(5);
//     block.height += 1;
// }
