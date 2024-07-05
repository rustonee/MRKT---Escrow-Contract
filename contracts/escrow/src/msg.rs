use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw721::Cw721ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateOwner {
        owner: Addr,
    },
    ReceiveNft(Cw721ReceiveMsg),
    
    CancelEscrow {
        escrow_id: u64
    },
    SendFunds {
        escrow_id: u64
    }    
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NftReceiveMsg {
    CreateEscrow {
        cw721_address: Addr,
        price: Uint128,
        buyer: Addr
    },
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetEscrow {
        id: u64
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
    pub escrow_id: u64,
    pub denom: String,
    pub enabled: bool
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct EscrowInfo {
    pub token_id: String,
    pub cw721_address: Addr,
    pub price: Uint128,
    pub amount: Uint128,
    pub seller: Addr,
    pub buyer: Addr,
    pub created_timestamp: u64,
    pub canceled_timestamp: u64,
}