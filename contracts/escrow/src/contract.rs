use crate::state::{Config, CONFIG, CREATORS, ESCROWS};
#[cfg(not(feature = "library"))]
use crate::{util, ContractError};
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::{Balance, Denom};
use cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg};

use crate::msg::{
    ConfigResponse, EscrowInfo, ExecuteMsg, InstantiateMsg, MigrateMsg, NftReceiveMsg, QueryMsg,
};
use cw2::get_contract_version;

// version info for migration info
const CONTRACT_NAME: &str = "escrow";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, crate::ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender.clone(),
        escrow_id: 0u64,
        denom: msg.denom,
        enabled: true,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetEscrow { id } => to_binary(&query_get_escrow(deps, id)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner,
        escrow_id: config.escrow_id,
        denom: config.denom,
        enabled: config.enabled,
    })
}

fn query_get_escrow(deps: Deps, id: u64) -> StdResult<EscrowInfo> {
    let escrow = ESCROWS.load(deps.storage, id).unwrap();
    Ok(escrow)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, crate::ContractError> {
    match msg {
        ExecuteMsg::UpdateOwner { owner } => {
            util::execute_update_owner(deps.storage, info.sender, owner)
        }
        ExecuteMsg::ReceiveNft(msg) => execute_receive_nft(deps, env, info, msg),
        ExecuteMsg::CancelEscrow { escrow_id } => execute_cancel_escrow(deps, env, info, escrow_id),
        ExecuteMsg::SendFunds { escrow_id } => execute_send_funds(deps, info, escrow_id),
    }
}

pub fn execute_receive_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wrapper: Cw721ReceiveMsg,
) -> Result<Response, crate::ContractError> {
    util::check_enabled(deps.storage)?;
    let mut config = CONFIG.load(deps.storage)?;
    let token_id = wrapper.token_id.clone();
    let user_addr = deps.api.addr_validate(wrapper.sender.as_str())?;
    let msg: NftReceiveMsg = from_binary(&wrapper.msg)?;
    match msg {
        NftReceiveMsg::CreateEscrow {
            cw721_address,
            price,
            buyer,
        } => {
            if info.sender.clone() != cw721_address.clone() {
                return Err(ContractError::InvalidCw721Token {});
            }
            let escrow = EscrowInfo {
                token_id: token_id.clone(),
                cw721_address: cw721_address.clone(),
                price: price.clone(),
                amount: Uint128::zero(),
                seller: user_addr.clone(),
                buyer: buyer.clone(),
                created_timestamp: env.block.time.seconds(),
                canceled_timestamp: 0u64,
            };
            let mut escrows: Vec<EscrowInfo> = vec![];
            if CREATORS.has(deps.storage, user_addr.clone()) {
                escrows = CREATORS.load(deps.storage, user_addr.clone())?;
            }
            escrows.push(escrow.clone());
            CREATORS.save(deps.storage, user_addr.clone(), &escrows)?;
            ESCROWS.save(deps.storage, config.escrow_id.clone(), &escrow)?;
            config.escrow_id += 1;
            CONFIG.save(deps.storage, &config)?;

            Ok(Response::new()
                .add_attribute("action", "execute_create_escrow")
                .add_attribute("token_id", token_id.clone())
                .add_attribute("price", price.clone())
                .add_attribute("buyer", buyer.clone()))
        }
    }
}

pub fn execute_cancel_escrow(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
) -> Result<Response, crate::ContractError> {
    util::check_enabled(deps.storage)?;
    let mut escrow = ESCROWS.load(deps.storage, id.clone())?;
    if escrow.seller.clone() != info.sender.clone() {
        return Err(ContractError::CannotCancelEscrowNotSeller {});
    }
    if !escrow.amount.is_zero() {
        return Err(ContractError::CannotCancelEscrowNoFunds {});
    }

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: escrow.cw721_address.clone().to_string(),
        msg: to_binary(&Cw721ExecuteMsg::TransferNft {
            token_id: escrow.token_id.clone(),
            recipient: info.sender.clone().into(),
        })?,
        funds: vec![],
    }));

    escrow.canceled_timestamp = env.block.time.seconds();
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "execute_cancel_escrow")
        .add_attribute("escrow_id", id.to_string()))
}

pub fn execute_send_funds(
    deps: DepsMut,
    info: MessageInfo,
    escorw_id: u64,
) -> Result<Response, crate::ContractError> {
    util::check_enabled(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let escrow = ESCROWS.load(deps.storage, escorw_id.clone())?;

    if escrow.buyer.clone() != info.sender.clone() {
        return Err(ContractError::InvalidBuyer {});
    }

    let amount = util::get_amount_of_denom(
        Balance::from(info.funds),
        Denom::Native(config.denom.clone()),
    )?;
    if amount < escrow.price {
        return Err(ContractError::InsufficientFund {});
    }

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: escrow.cw721_address.clone().to_string(),
        msg: to_binary(&Cw721ExecuteMsg::TransferNft {
            token_id: escrow.token_id.clone(),
            recipient: info.sender.clone().into(),
        })?,
        funds: vec![],
    }));

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "execute_send_funds")
        .add_attribute("token_id", escrow.token_id.to_string())
        .add_attribute("buyer", info.sender.clone()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<Response, crate::ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(crate::ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}
