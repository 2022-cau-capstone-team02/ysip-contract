use cosmwasm_std::{Addr, StdResult, Api, QuerierWrapper};
use crate::querier::query_token_symbol;

const TOKEN_SYMBOL_MAX_LENGTH: usize = 10;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetInfo {
    /// Non-native Token
    Token { contract_addr: Addr },

    /// Native Token
    NativeToken { denom: String },
}

impl AssetInfo {
    pub fn check_is_valid(&self, api: &dyn Api) -> StdResult<()> {
        match self {
            AssetInfo::Token { contract_addr } => api.addr_validate(contract_addr.as_str())?,
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
}

pub fn format_lp_token_name(
    asset_infos: [AssetInfo; 2],
    querier: &QuerierWrapper,
) -> StdResult<String> {
    let mut short_symbols: Vec<String> = vec![];
    asset_infos.into_iter().for_each(|asset_info| {
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
    });

    Ok(format!("{}-{}-LP", short_symbols[0], short_symbols[1]).to_uppercase())
}