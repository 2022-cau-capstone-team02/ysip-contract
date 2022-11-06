use crate::querier::{query_balance, query_token_balance, query_token_symbol};
use cosmwasm_std::{Addr, Api, MessageInfo, QuerierWrapper, StdError, StdResult, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const TOKEN_SYMBOL_MAX_LENGTH: usize = 10;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Asset {
    /// Information about an asset stored in a [`AssetInfo`] struct
    pub info: AssetInfo,
    /// A token amount
    pub amount: Uint128,
}

impl Asset {
    pub fn is_cw20_token(&self) -> bool {
        match &self.info {
            AssetInfo::NativeToken { .. } => false,
            AssetInfo::Token { .. } => true,
        }
    }

    pub fn assert_sent_native_token_balance(&self, message_info: &MessageInfo) -> StdResult<()> {
        if let AssetInfo::NativeToken { denom } = &self.info {
            match message_info.funds.iter().find(|fund| fund.denom == *denom) {
                Some(coin) => {
                    if self.amount == coin.amount {
                        Ok(())
                    } else {
                        Err(StdError::generic_err(
                            "Native token balance transffered mismatch with the argument",
                        ))
                    }
                }
                None => {
                    if self.amount.is_zero() {
                        Ok(())
                    } else {
                        Err(StdError::generic_err(
                            "Native token balance transffered mismatch with the argument",
                        ))
                    }
                }
            }
        } else {
            Ok(())
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetInfo {
    /// Non-native Token
    Token { contract_addr: Addr },

    /// Native Token
    NativeToken { denom: String },
}

impl PartialEq for AssetInfo {
    fn eq(&self, other: &Self) -> bool {
        match self {
            AssetInfo::Token { contract_addr } => {
                let self_contract_addr = contract_addr;
                match other {
                    AssetInfo::Token { contract_addr } => self_contract_addr == contract_addr,
                    AssetInfo::NativeToken { .. } => false,
                }
            }
            AssetInfo::NativeToken { denom } => {
                let self_denom = denom;
                match other {
                    AssetInfo::Token { .. } => false,
                    AssetInfo::NativeToken { denom } => self_denom == denom,
                }
            }
        }
    }
}

impl AssetInfo {
    pub fn check_is_valid(&self, api: &dyn Api) -> StdResult<()> {
        match self {
            AssetInfo::Token { contract_addr } => {
                api.addr_validate(contract_addr.as_str())?;
            }
            AssetInfo::NativeToken { denom } => {
                if !denom.starts_with("ibc/") && denom != &denom.to_lowercase() {
                    return Err(StdError::generic_err(format!(
                        "Non-IBC token denom {} should be lowercase",
                        denom
                    )));
                }
            }
        }
        Ok(())
    }

    pub fn query_pool(&self, querier: &QuerierWrapper, pool_addr: &Addr) -> StdResult<Uint128> {
        match self {
            AssetInfo::Token { contract_addr } => {
                query_token_balance(querier, contract_addr, pool_addr)
            }
            AssetInfo::NativeToken { denom } => {
                query_balance(querier, pool_addr, denom.to_string())
            }
        }
    }
}

pub fn format_lp_token_name(
    asset_infos: [AssetInfo; 2],
    querier: &QuerierWrapper,
) -> StdResult<String> {
    let mut short_symbols: Vec<String> = vec![];

    for asset_info in asset_infos {
        let short_symbol = match asset_info {
            AssetInfo::NativeToken { denom } => {
                denom.chars().take(TOKEN_SYMBOL_MAX_LENGTH).collect()
            }
            AssetInfo::Token { contract_addr } => {
                let token_symbol = query_token_symbol(querier, contract_addr)?;
                token_symbol.chars().take(TOKEN_SYMBOL_MAX_LENGTH).collect()
            }
        };
        short_symbols.push(short_symbol);
    }

    Ok(format!("{}-{}-LP", short_symbols[0], short_symbols[1]).to_uppercase())
}
