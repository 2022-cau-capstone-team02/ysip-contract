use testing::execute::execute_mint;
use testing::init::mock_cw20_contract;
use testing::instantiate::instantiate_cw20_contract;
use testing_base::consts::ADDR1;
use testing_base::init::init_app;

fn main() {
    let mut app = init_app(ADDR1);
    let channel_a = app.store_code(mock_cw20_contract());

    let channel_a_contract_addr = instantiate_cw20_contract(
        &mut app,
        channel_a,
        ADDR1,
        ADDR1,
        "channel_a",
        "channel-a",
        vec![],
        "channel_a"
    );

    println!("{}", channel_a_contract_addr);

    let res = execute_mint(
        &mut app,
        channel_a_contract_addr.as_ref(),
        ADDR1,
        ADDR1,
        100
    );

    println!("{:?}", res);
}