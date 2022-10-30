use crate::asset::{Asset, AssetInfo};
use cosmwasm_std::{Addr, Decimal, QuerierWrapper, StdError, StdResult, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Information about the two assets in the pool
    pub asset_infos: [AssetInfo; 2],
    /// The token contract code ID used for the tokens in the pool
    pub token_code_id: u64,
    /// The factory contract address
    pub factory_addr: String,
    /// Address that recieves protocol fee
    pub protocol_fee_recipient: String,
    pub protocol_fee_percent: String,
    pub lp_fee_percent: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PairInfo {
    pub asset_infos: [AssetInfo; 2],
    pub contract_addr: Addr,
    pub liquidity_token: Addr,
}

impl PairInfo {
    pub fn init(contract_addr: Addr, asset_infos: [AssetInfo; 2]) -> PairInfo {
        Self {
            asset_infos,
            contract_addr,
            liquidity_token: Addr::unchecked(""),
        }
    }

    pub fn query_pools(
        &self,
        querier: &QuerierWrapper,
        contract_addr: Addr,
    ) -> StdResult<[Asset; 2]> {
        Ok([
            Asset {
                amount: self.asset_infos[0].query_pool(querier, contract_addr.clone())?,
                info: self.asset_infos[0].clone(),
            },
            Asset {
                amount: self.asset_infos[1].query_pool(querier, contract_addr)?,
                info: self.asset_infos[1].clone(),
            },
        ])
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// ProvideLiquidity allows someone to provide liquidity in the pool
    ProvideLiquidity {
        /// The assets available in the pool
        assets: [Asset; 2],
    },
    /// Swap operation
    Swap {
        offer_asset: Asset,
        min_output_amount: Option<String>,
        max_spread: Option<String>,
        to: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}

pub struct SwapParams {
    pub offer_asset: Asset,
    pub min_output_amount: Option<String>,
    pub max_spread: Option<String>,
    pub to: Option<Addr>,
}

fn to_decimal(input: &Option<String>) -> StdResult<Decimal> {
    Ok(Decimal::from_str(
        &input.clone().unwrap_or("0".to_string()),
    )?)
}

impl SwapParams {
    pub fn assert_min_token_bought(&self, true_amount: Uint128) -> StdResult<()> {
        let min_output_amount = to_decimal(&self.min_output_amount)?;
        let max_spread = to_decimal(&self.max_spread)?;

        let max_diff = min_output_amount
            * Decimal::from_ratio(max_spread.atomics(), Uint128::new(10u128.pow(20)));

        let true_amount = Decimal::new(true_amount * Uint128::new(10u128.pow(18)));

        if true_amount >= max_diff {
            Ok(())
        } else {
            Err(StdError::generic_err("not enough token bought"))
        }
    }
}

#[cfg(test)]
pub mod test_swap {
    use crate::asset::{Asset, AssetInfo};
    use crate::pair::SwapParams;
    use cosmwasm_std::{Addr, Decimal, Uint128};
    use std::str::FromStr;

    #[test]
    fn test() {
        let sp = SwapParams {
            offer_asset: Asset {
                info: AssetInfo::Token {
                    contract_addr: Addr::unchecked(""),
                },
                amount: Uint128::new(10000),
            },
            min_output_amount: Some("100".to_string()),
            max_spread: Some("5".to_string()),
            to: Some(Addr::unchecked("")),
        };

        // success
        sp.assert_min_token_bought(Decimal::from_str("105").unwrap())
            .unwrap();

        // fail
        sp.assert_min_token_bought(Decimal::from_str("106").unwrap())
            .unwrap_err();

        // success
        sp.assert_min_token_sold(Decimal::from_str("95").unwrap())
            .unwrap();

        // fail
        sp.assert_min_token_sold(Decimal::from_str("94").unwrap())
            .unwrap_err();
    }
}
