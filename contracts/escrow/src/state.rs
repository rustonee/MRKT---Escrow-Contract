use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use cw_storage_plus::Map;
use crate::msg::EscrowInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub escrow_id: u64,
    pub denom: String,
    pub enabled: bool,
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

pub const ESCROWS_KEY: &str = "escrows";
pub const ESCROWS: Map<u64, EscrowInfo> = Map::new(ESCROWS_KEY);

pub const CREATORS_KEY: &str = "creators";
pub const CREATORS: Map<Addr, Vec<EscrowInfo>> = Map::new(CREATORS_KEY);
