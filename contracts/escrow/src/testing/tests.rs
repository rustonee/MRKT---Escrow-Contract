use crate::contract::{execute, instantiate, query};
use crate::msg::{ConfigResponse, EscrowInfo, ExecuteMsg, InstantiateMsg, NftReceiveMsg, QueryMsg};
use crate::ContractError;
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    coins, from_binary, from_slice, to_binary, Addr, CosmosMsg, OwnedDeps, ReplyOn, SubMsg,
    Uint128, WasmMsg,
};
use cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg};
use std::marker::PhantomData;

pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::default(),
        custom_query_type: PhantomData,
    }
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        denom: "usei".to_string(),
    };
    let info = mock_info("addr0000", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap()).unwrap();
    assert_eq!("usei", config.denom.as_str());
}

#[test]
fn create_escrow() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        denom: "usei".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // beneficiary can release it
    let info = mock_info("nft0001", &[]);
    let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
        sender: "addr0000".to_string(),
        token_id: "0001".to_string(),
        msg: to_binary(&NftReceiveMsg::CreateEscrow {
            cw721_address: Addr::unchecked("nft0001"),
            price: Uint128::from(1000u128),
            buyer: Addr::unchecked("addr0001"),
        })
        .unwrap(),
    });
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap()).unwrap();
    assert_eq!(1, config.escrow_id);
}

#[test]
fn cancel_escrow() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        denom: "usei".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // beneficiary can release it
    let info = mock_info("nft0001", &[]);
    let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
        sender: "addr0000".to_string(),
        token_id: "0001".to_string(),
        msg: to_binary(&NftReceiveMsg::CreateEscrow {
            cw721_address: Addr::unchecked("nft0001"),
            price: Uint128::from(1000u128),
            buyer: Addr::unchecked("addr0001"),
        })
        .unwrap(),
    });
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let msg = QueryMsg::GetEscrow { id: 0u64 };
    let res: EscrowInfo = from_binary(&query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();

    let info = mock_info("addr0000", &[]);
    let msg = ExecuteMsg::CancelEscrow { escrow_id: 0u64 };
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "nft0001".to_string(),
                msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                    recipient: "addr0000".to_string(),
                    token_id: "0001".to_string()
                })
                .unwrap(),
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );
}

#[test]
fn send_funds() {
    let mut deps = mock_dependencies();
    let msg = InstantiateMsg {
        denom: "usei".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    // beneficiary can release it
    let info = mock_info("nft0001", &[]);
    let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
        sender: "addr0000".to_string(),
        token_id: "0001".to_string(),
        msg: to_binary(&NftReceiveMsg::CreateEscrow {
            cw721_address: Addr::unchecked("nft0001"),
            price: Uint128::from(1000u128),
            buyer: Addr::unchecked("addr0001"),
        })
        .unwrap(),
    });
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info("addr0001", &coins(500, "usei"));
    let msg = ExecuteMsg::SendFunds { escrow_id: 0 };

    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap_err();
    assert_eq!(res, ContractError::InsufficientFund {});

    let info = mock_info("addr0001", &coins(1000, "usei"));
    let res = execute(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
    assert_eq!(
        res.messages[0],
        SubMsg {
            id: 0,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "nft0001".to_string(),
                msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                    recipient: "addr0001".to_string(),
                    token_id: "0001".to_string()
                })
                .unwrap(),
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        }
    );
}
