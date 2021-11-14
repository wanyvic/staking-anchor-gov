use crate::contract::{execute, instantiate, query, reply};
use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse,
    UserStateResponse, WITHDRAW_REPLY_ID,
};
use crate::state::{
    config_read, feerate_read, temp_send_store, total_shares_read, total_shares_store,
    user_states_read, user_states_store, Config, TempSendData,
};

use crate::mock_querier::mock_dependencies;
use anchor_token::gov::Cw20HookMsg as GovCw20HookMsg;
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, to_binary, Api, ContractResult, CosmosMsg, Decimal, DepsMut, Reply, Response,
    StdError, SubMsg, SubMsgExecutionResponse, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

use anchor_token::gov::{ExecuteMsg as GovExcuteMsg, StakerResponse, VoteOption, VoterInfo};
const DEFAULT_FEERATE: u64 = 2;
const TEST_NEW_FEERATE: u64 = 5;
const TEST_DEV: &str = "dev";
const TEST_DEV2: &str = "dev2";
const TEST_ANCHOR_TOKEN: &str = "anchor_token";
const TEST_ANCHOR_GOV: &str = "anchor_gov";
const TEST_CREATOR: &str = "creator";
const TEST_ALICE: &str = "alice";
const TEST_BOB: &str = "bob";

//
fn mock_instantiate(deps: DepsMut) {
    let msg = InstantiateMsg {
        feerate: Decimal::percent(DEFAULT_FEERATE),
        dev: TEST_DEV.to_string(),
        anchor_token: TEST_ANCHOR_TOKEN.to_string(),
        anchor_gov: TEST_ANCHOR_GOV.to_string(),
    };

    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps, mock_env(), info, msg)
        .expect("contract successfully handles InstantiateMsg");
}

fn set_pedding_owner(deps: DepsMut, new_owner: String) {
    let msg = ExecuteMsg::TransferOwnerShip {
        new_owner: new_owner,
    };

    let info = mock_info(TEST_CREATOR, &[]);

    execute(deps, mock_env(), info.clone(), msg)
        .expect("contract successfully handles RegisterContracts");
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        feerate: Decimal::percent(DEFAULT_FEERATE),
        dev: TEST_DEV.to_string(),
        anchor_token: TEST_ANCHOR_TOKEN.to_string(),
        anchor_gov: TEST_ANCHOR_GOV.to_string(),
    };

    let info = mock_info(TEST_CREATOR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    //1. checkout config setting
    let config: Config = config_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config,
        Config {
            anchor_token: deps.api.addr_canonicalize(TEST_ANCHOR_TOKEN).unwrap(),
            anchor_gov: deps.api.addr_canonicalize(TEST_ANCHOR_GOV).unwrap(),
            dev: deps.api.addr_canonicalize(TEST_DEV).unwrap(),
            owner: deps.api.addr_canonicalize(TEST_CREATOR).unwrap(),
            pendding_owner: None,
        }
    );
    //2. checkout fee rate setting
    let feerate: Decimal = feerate_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(Decimal::percent(DEFAULT_FEERATE), feerate)
}

/// execute
#[test]
fn fails_update_dev_with_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let msg = ExecuteMsg::UpdateDev {
        new_dev: TEST_DEV2.to_string(),
    };
    let info = mock_info(TEST_DEV, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return error"),
    }
}

// TODO:
#[test]
fn fails_update_dev_without_validated_address() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let msg = ExecuteMsg::UpdateDev {
        new_dev: "12".to_string(),
    };
    let info = mock_info(TEST_CREATOR, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Std(StdError::GenericErr { msg, .. })) => {
            assert_eq!(msg, "Invalid input: human address too short")
        }
        _ => panic!("Must return error"),
    }
}

#[test]
fn proper_update_dev() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let msg = ExecuteMsg::UpdateDev {
        new_dev: TEST_DEV2.to_string(),
    };
    let info = mock_info(TEST_CREATOR, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let config: Config = config_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config,
        Config {
            dev: deps.api.addr_canonicalize(TEST_DEV2).unwrap(),
            ..config.clone()
        }
    );
}

// checkout set_pedding_owner ACL
#[test]
fn fails_transfer_ownership_with_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let msg = ExecuteMsg::TransferOwnerShip {
        new_owner: TEST_DEV2.to_string(),
    };

    let info = mock_info(TEST_DEV, &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return error"),
    }
}

/// TODO:
#[test]
fn fails_transfer_ownership_without_validated_address() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let msg = ExecuteMsg::TransferOwnerShip {
        new_owner: "12".to_string(),
    };

    let info = mock_info(TEST_CREATOR, &[]);

    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg);
    match res {
        Err(ContractError::Std(StdError::GenericErr { msg, .. })) => {
            assert_eq!(msg, "Invalid input: human address too short")
        }
        _ => panic!("Must return error"),
    }
}
// checkout pendding_owner value
#[test]
fn proper_transfer_ownership() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let msg = ExecuteMsg::TransferOwnerShip {
        new_owner: TEST_DEV2.to_string(),
    };

    let info = mock_info(TEST_CREATOR, &[]);

    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    let config: Config = config_read(deps.as_ref().storage).load().unwrap();

    assert_eq!(
        config,
        Config {
            pendding_owner: Some(deps.api.addr_canonicalize(TEST_DEV2).unwrap()),
            ..config.clone()
        }
    );
}
/// TODO:
#[test]
fn proper_transfer_ownership_with_none() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let msg = ExecuteMsg::TransferOwnerShip {
        new_owner: TEST_DEV2.to_string(),
    };

    let info = mock_info(TEST_CREATOR, &[]);

    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    let config: Config = config_read(deps.as_ref().storage).load().unwrap();

    assert_eq!(
        config,
        Config {
            pendding_owner: Some(deps.api.addr_canonicalize(TEST_DEV2).unwrap()),
            ..config.clone()
        }
    );
    // reset
    let msg = ExecuteMsg::TransferOwnerShip {
        new_owner: String::default(),
    };

    let info = mock_info(TEST_CREATOR, &[]);

    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    let config: Config = config_read(deps.as_ref().storage).load().unwrap();

    assert_eq!(
        config,
        Config {
            pendding_owner: None,
            ..config.clone()
        }
    );
}

#[test]
fn fails_accepted_owner_with_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let msg = ExecuteMsg::AcceptOwner {};
    let info = mock_info(TEST_DEV, &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return error"),
    }
    set_pedding_owner(deps.as_mut(), TEST_DEV2.to_string());
    let msg = ExecuteMsg::AcceptOwner {};
    let info = mock_info(TEST_DEV, &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return error"),
    }
}

#[test]
fn proper_accepted_owner() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    set_pedding_owner(deps.as_mut(), TEST_DEV2.to_string());
    let msg = ExecuteMsg::AcceptOwner {};
    let info = mock_info(TEST_DEV2, &[]);

    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    let config: Config = config_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config,
        Config {
            owner: deps.api.addr_canonicalize(TEST_DEV2).unwrap(),
            pendding_owner: None,
            ..config.clone()
        }
    );
}

#[test]
fn fails_update_feerate_with_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let info = mock_info(TEST_DEV, &[]);
    let msg = ExecuteMsg::UpdateFeeRate {
        new_feerate: Decimal::percent(TEST_NEW_FEERATE),
    };
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return error"),
    }
}
#[test]
fn fails_update_feerate_out_of_limits() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let msg = ExecuteMsg::UpdateFeeRate {
        new_feerate: Decimal::percent(101),
    };
    let info = mock_info(TEST_CREATOR, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::FeeRateOutOfLimits {}) => (),
        _ => panic!("Must return error"),
    }
}

#[test]
fn proper_update_feerate() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let msg = ExecuteMsg::UpdateFeeRate {
        new_feerate: Decimal::percent(TEST_NEW_FEERATE),
    };
    let info = mock_info(TEST_CREATOR, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let feerate: Decimal = feerate_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(feerate, Decimal::percent(TEST_NEW_FEERATE));
}

#[test]
fn fails_receive_with_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_ALICE.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::StakingTokens {}).unwrap(),
    });
    let info = mock_info(&(TEST_ANCHOR_TOKEN.to_string() + "2"), &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return error"),
    }
}

#[test]
fn fails_receive_with_zero_amount() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_ALICE.to_string(),
        amount: Uint128::zero(),
        msg: to_binary(&Cw20HookMsg::StakingTokens {}).unwrap(),
    });
    let info = mock_info(&(TEST_ANCHOR_TOKEN.to_string()), &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::InsufficientFunds {}) => (),
        _ => panic!("Must return error"),
    }
}

#[test]
pub fn proper_receive_with_dev_fee() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let transfer_amount_alice = Uint128::from(1_000_000u128);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_ALICE.to_string(),
        amount: transfer_amount_alice,
        msg: to_binary(&Cw20HookMsg::StakingTokens {}).unwrap(),
    });
    let info = mock_info(&(TEST_ANCHOR_TOKEN.to_string()), &[]);

    // deposit MOCK_CONTRACT_ADDR some tokens.
    let transfer_contract_amount1 = Uint128::from(2_000_000u128);
    deps.querier.with_token_balances(&[(
        &TEST_ANCHOR_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &transfer_contract_amount1)],
    )]);

    deps.querier.with_gov_stakers(&[(
        &TEST_ANCHOR_GOV.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &StakerResponse {
                balance: Uint128::zero(),
                share: Uint128::zero(),
                locked_balance: vec![],
            },
        )],
    )]);

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let msg = res.messages.get(0).expect("no message");
    // assert for Send to TEST_ANCHOR_GOV SubMsg
    assert_eq!(
        msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_ANCHOR_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: TEST_ANCHOR_GOV.to_string(),
                amount: transfer_contract_amount1,
                msg: to_binary(&GovCw20HookMsg::StakeVotingTokens {}).unwrap(),
            })
            .unwrap(),
            funds: vec![],
        }))
    );

    //dev check
    let key = deps.api.addr_canonicalize(TEST_DEV).unwrap();
    let dev_shares = user_states_read(deps.as_ref().storage)
        .may_load(&key.as_slice())
        .unwrap_or_default()
        .unwrap_or_default();

    let feerate = feerate_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(dev_shares, feerate * transfer_amount_alice);

    // user check
    let key = deps.api.addr_canonicalize(TEST_ALICE).unwrap();
    let user_shares = user_states_read(deps.as_ref().storage)
        .may_load(&key.as_slice())
        .unwrap_or_default()
        .unwrap_or_default();

    let total_shares = total_shares_read(deps.as_ref().storage).load().unwrap();

    // first deposit, the same.

    assert_eq!(
        user_shares,
        transfer_amount_alice - (feerate * transfer_amount_alice)
    );

    assert_eq!(total_shares, user_shares + dev_shares);
}

#[test]
pub fn proper_receive_with_dev_fee_same_account() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let transfer_amount_alice = Uint128::from(1_000_000u128);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_DEV.to_string(),
        amount: transfer_amount_alice,
        msg: to_binary(&Cw20HookMsg::StakingTokens {}).unwrap(),
    });
    let info = mock_info(&(TEST_ANCHOR_TOKEN.to_string()), &[]);

    // deposit MOCK_CONTRACT_ADDR some tokens.
    let transfer_contract_amount1 = Uint128::from(2_000_000u128);
    deps.querier.with_token_balances(&[(
        &TEST_ANCHOR_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &transfer_contract_amount1)],
    )]);

    deps.querier.with_gov_stakers(&[(
        &TEST_ANCHOR_GOV.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &StakerResponse {
                balance: Uint128::zero(),
                share: Uint128::zero(),
                locked_balance: vec![],
            },
        )],
    )]);

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let msg = res.messages.get(0).expect("no message");
    // assert for Send to TEST_ANCHOR_GOV SubMsg
    assert_eq!(
        msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_ANCHOR_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: TEST_ANCHOR_GOV.to_string(),
                amount: transfer_contract_amount1,
                msg: to_binary(&GovCw20HookMsg::StakeVotingTokens {}).unwrap(),
            })
            .unwrap(),
            funds: vec![],
        }))
    );

    // user check
    let key = deps.api.addr_canonicalize(TEST_DEV).unwrap();
    let user_shares = user_states_read(deps.as_ref().storage)
        .may_load(&key.as_slice())
        .unwrap_or_default()
        .unwrap_or_default();

    let total_shares = total_shares_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(total_shares, user_shares);
}

#[test]
fn proper_receive_without_dev_fee_double() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        feerate: Decimal::zero(),
        dev: TEST_DEV.to_string(),
        anchor_token: TEST_ANCHOR_TOKEN.to_string(),
        anchor_gov: TEST_ANCHOR_GOV.to_string(),
    };

    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg)
        .expect("contract successfully handles InstantiateMsg");

    let transfer_amount_alice = Uint128::from(1_000_000u128);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_ALICE.to_string(),
        amount: transfer_amount_alice,
        msg: to_binary(&Cw20HookMsg::StakingTokens {}).unwrap(),
    });

    // let a: Binary = to_binary(&Cw20HookMsg::StakingTokens {}).unwrap();
    // let b = String::from_utf8(to_vec(&a).unwrap()).unwrap();
    // let c = String::from_utf8(to_vec(&Cw20HookMsg::StakingTokens {}).unwrap()).unwrap();
    // println!("{},{},{}", a.to_base64(), b, c);
    let info = mock_info(&(TEST_ANCHOR_TOKEN.to_string()), &[]);

    // deposit MOCK_CONTRACT_ADDR some tokens.
    let transfer_contract_amount1 = Uint128::from(2_000_000u128);
    deps.querier.with_token_balances(&[(
        &TEST_ANCHOR_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &transfer_contract_amount1)],
    )]);

    deps.querier.with_gov_stakers(&[(
        &TEST_ANCHOR_GOV.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &StakerResponse {
                balance: Uint128::zero(),
                share: Uint128::zero(),
                locked_balance: vec![],
            },
        )],
    )]);

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let msg = res.messages.get(0).expect("no message");
    // assert for Send to TEST_ANCHOR_GOV SubMsg
    assert_eq!(
        msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_ANCHOR_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: TEST_ANCHOR_GOV.to_string(),
                amount: transfer_contract_amount1,
                msg: to_binary(&GovCw20HookMsg::StakeVotingTokens {}).unwrap(),
            })
            .unwrap(),
            funds: vec![],
        }))
    );

    // user check
    let key = deps.api.addr_canonicalize(TEST_ALICE).unwrap();
    let user_shares_alice = user_states_read(deps.as_ref().storage)
        .may_load(&key.as_slice())
        .unwrap_or_default()
        .unwrap_or_default();

    let total_shares1 = total_shares_read(deps.as_ref().storage).load().unwrap();

    // first deposit, the same.
    assert_eq!(total_shares1, user_shares_alice);

    assert_eq!(user_shares_alice, transfer_amount_alice);

    // double transfer
    let transfer_contract_amount2 = Uint128::from(3_000_000u128);
    let second_contract_total_balance = transfer_contract_amount1.multiply_ratio(7u128, 3u128);
    let second_contract_total_shares = transfer_contract_amount1.multiply_ratio(7u128, 3u128);
    deps.querier.with_token_balances(&[(
        &TEST_ANCHOR_TOKEN.to_string(),
        &[
            (&TEST_ANCHOR_GOV.to_string(), &transfer_contract_amount1),
            (&MOCK_CONTRACT_ADDR.to_string(), &transfer_contract_amount2),
        ],
    )]);
    deps.querier.with_gov_stakers(&[(
        &TEST_ANCHOR_GOV.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &StakerResponse {
                balance: second_contract_total_balance,
                share: second_contract_total_shares,
                locked_balance: vec![],
            },
        )],
    )]);

    let transfer_amount_bob = Uint128::from(1_000_000u128);
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_BOB.to_string(),
        amount: transfer_amount_bob,
        msg: to_binary(&Cw20HookMsg::StakingTokens {}).unwrap(),
    });

    let info = mock_info(&(TEST_ANCHOR_TOKEN.to_string()), &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let msg = res.messages.get(0).expect("no message");

    assert_eq!(
        msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_ANCHOR_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: TEST_ANCHOR_GOV.to_string(),
                amount: transfer_contract_amount2,
                msg: to_binary(&GovCw20HookMsg::StakeVotingTokens {}).unwrap(),
            })
            .unwrap(),
            funds: vec![],
        }))
    );

    let key = deps.api.addr_canonicalize(TEST_BOB).unwrap();
    let user_shares_bob = user_states_read(deps.as_ref().storage)
        .may_load(&key.as_slice())
        .unwrap_or_default()
        .unwrap_or_default();

    let total_shares2 = total_shares_read(deps.as_ref().storage).load().unwrap();
    // second deposit, the same.
    assert_eq!(total_shares2, user_shares_bob + user_shares_alice);
    assert_eq!(
        user_shares_bob,
        transfer_amount_bob.multiply_ratio(total_shares1, second_contract_total_balance)
    );
}

#[test]
fn fails_withdraw_token_out_of_amount() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let user_shares = Uint128::from(1000u128);
    let key = deps.api.addr_canonicalize(TEST_ALICE).unwrap();
    let balance = Uint128::from(100_000_000u128);
    let share = Uint128::from(50_000_000u128);
    user_states_store(deps.as_mut().storage)
        .save(&key, &user_shares)
        .unwrap();
    total_shares_store(deps.as_mut().storage)
        .save(&(balance / Uint128::from(2u128)))
        .unwrap();

    deps.querier.with_gov_stakers(&[(
        &TEST_ANCHOR_GOV.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &StakerResponse {
                balance: balance,
                share: share,
                locked_balance: vec![],
            },
        )],
    )]);

    let msg = QueryMsg::UserState {
        user: TEST_ALICE.to_string(),
    };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let user_state_response: UserStateResponse = from_binary(&res).unwrap();
    let user_balance = user_state_response.available_balance;

    // checkout user share and amount
    assert_eq!(user_state_response.shares, user_shares);
    assert_eq!(
        user_state_response.available_balance,
        user_shares * Uint128::from(2u128)
    );
    // try to withdraw out of balance
    let msg = ExecuteMsg::WithdrawToken {
        amount: Some(user_balance + Uint128::from(2u128)),
    };

    let info = mock_info(TEST_ALICE, &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg);
    match res {
        Err(ContractError::InsufficientFunds {}) => (),
        _ => panic!("Must return error"),
    }
}
#[test]
fn fails_withdraw_token_without_deposit() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    total_shares_store(deps.as_mut().storage)
        .save(&Uint128::from(1000u128))
        .unwrap();

    // try to withdraw out of balance
    let msg = ExecuteMsg::WithdrawToken {
        amount: Some(Uint128::from(1u128)),
    };

    let info = mock_info(TEST_ALICE, &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg);
    match res {
        Err(ContractError::NothingStaked {}) => (),
        _ => panic!("Must return error"),
    }
}

#[test]
fn proper_withdraw_token() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let user_shares = Uint128::from(1000u128);
    let key = deps.api.addr_canonicalize(TEST_ALICE).unwrap();
    user_states_store(deps.as_mut().storage)
        .save(&key, &user_shares)
        .unwrap();
    total_shares_store(deps.as_mut().storage)
        .save(&user_shares)
        .unwrap();

    let balance = Uint128::from(100_000_000u128);
    let share = Uint128::from(50_000_000u128);
    deps.querier.with_gov_stakers(&[(
        &TEST_ANCHOR_GOV.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &StakerResponse {
                balance: balance,
                share: share,
                locked_balance: vec![],
            },
        )],
    )]);

    let msg = QueryMsg::UserState {
        user: TEST_ALICE.to_string(),
    };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let user_state_response: UserStateResponse = from_binary(&res).unwrap();

    // checkout user share and amount
    assert_eq!(user_state_response.shares, user_shares);
    assert_eq!(user_state_response.available_balance, balance);

    // try to withdraw a few
    let withdraw_amount = balance / Uint128::from(2u128);
    let msg = ExecuteMsg::WithdrawToken {
        amount: Some(withdraw_amount),
    };

    let info = mock_info(TEST_ALICE, &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    // assert for Send to TEST_ANCHOR_GOV SubMsg
    assert_eq!(
        res,
        Response::new().add_submessage(SubMsg::reply_on_success(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_ANCHOR_GOV.to_string(),
                msg: to_binary(&GovExcuteMsg::WithdrawVotingTokens {
                    amount: Some(withdraw_amount)
                })
                .unwrap(),
                funds: vec![],
            }),
            WITHDRAW_REPLY_ID
        ))
    )
}

#[test]
fn proper_withdraw_token_all() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let user_shares = Uint128::from(1000u128);
    let key = deps.api.addr_canonicalize(TEST_ALICE).unwrap();
    user_states_store(deps.as_mut().storage)
        .save(&key, &user_shares)
        .unwrap();
    total_shares_store(deps.as_mut().storage)
        .save(&user_shares)
        .unwrap();

    let balance = Uint128::from(100_000_000u128);
    let share = Uint128::from(50_000_000u128);
    deps.querier.with_gov_stakers(&[(
        &TEST_ANCHOR_GOV.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &StakerResponse {
                balance: balance,
                share: share,
                locked_balance: vec![],
            },
        )],
    )]);

    let msg = QueryMsg::UserState {
        user: TEST_ALICE.to_string(),
    };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let user_state_response: UserStateResponse = from_binary(&res).unwrap();

    // checkout user share and amount
    assert_eq!(user_state_response.shares, user_shares);
    assert_eq!(user_state_response.available_balance, balance);

    // try to withdraw a few
    let msg = ExecuteMsg::WithdrawToken { amount: None };

    let info = mock_info(TEST_ALICE, &[]);
    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    let msg = res.messages.get(0).expect("no message");
    // assert for Send to TEST_ANCHOR_GOV SubMsg
    assert_eq!(
        msg,
        &SubMsg::reply_on_success(
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_ANCHOR_GOV.to_string(),
                msg: to_binary(&GovExcuteMsg::WithdrawVotingTokens {
                    amount: Some(balance)
                })
                .unwrap(),
                funds: vec![],
            }),
            WITHDRAW_REPLY_ID
        )
    );
}

#[test]
fn fails_reply_temp_send_data_not_found() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let reply_msg = Reply {
        id: WITHDRAW_REPLY_ID,
        result: ContractResult::Ok(SubMsgExecutionResponse {
            events: vec![],
            data: None,
        }),
    };
    let res = reply(deps.as_mut(), mock_env(), reply_msg);
    match res {
        Err(ContractError::Std(StdError::NotFound { kind, .. })) => {
            assert_eq!(kind, "staking_anchor_gov::state::TempSendData")
        }
        _ => panic!("Must return error"),
    }
}

#[test]
fn proper_reply() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    temp_send_store(&mut deps.storage)
        .save(&TempSendData {
            recipient: TEST_CREATOR.to_string(),
            amount: Uint128::from(100u128),
        })
        .unwrap();
    let reply_msg = Reply {
        id: WITHDRAW_REPLY_ID,
        result: ContractResult::Ok(SubMsgExecutionResponse {
            events: vec![],
            data: None,
        }),
    };
    let res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();
    let msg = res.messages.get(0).expect("no message");
    assert_eq!(
        msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: TEST_ANCHOR_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: TEST_CREATOR.to_string(),
                amount: Uint128::from(100u128),
            })
            .unwrap(),
            funds: vec![],
        }))
    )
}

// query

#[test]
fn query_config() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let msg = QueryMsg::Config {};
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();

    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config_response,
        ConfigResponse {
            anchor_token: TEST_ANCHOR_TOKEN.to_string(),
            anchor_gov: TEST_ANCHOR_GOV.to_string(),
            dev: TEST_DEV.to_string(),
            owner: TEST_CREATOR.to_string(),
            pendding_owner: String::default(),
        }
    );

    // checkout pedding_owner
    set_pedding_owner(deps.as_mut(), TEST_DEV2.to_string());
    let msg = QueryMsg::Config {};
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let config_response: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!(
        config_response,
        ConfigResponse {
            anchor_token: TEST_ANCHOR_TOKEN.to_string(),
            anchor_gov: TEST_ANCHOR_GOV.to_string(),
            dev: TEST_DEV.to_string(),
            owner: TEST_CREATOR.to_string(),
            pendding_owner: TEST_DEV2.to_string(),
        }
    );
}
#[test]
fn query_user_state() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let transfer_amount_alice = Uint128::from(1_000_000u128);
    let info = mock_info(&(TEST_ANCHOR_TOKEN.to_string()), &[]);

    // query empty UserState
    let msg = QueryMsg::UserState {
        user: TEST_ALICE.to_string(),
    };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let user_state_response: UserStateResponse = from_binary(&res).unwrap();

    assert_eq!(
        user_state_response,
        UserStateResponse {
            available_balance: Uint128::zero(),
            shares: Uint128::zero(),
            locked_balance: Uint128::zero(),
        }
    );

    // deposit MOCK_CONTRACT_ADDR some tokens.
    let transfer_contract_amount1 = Uint128::from(2_000_000u128);
    deps.querier.with_token_balances(&[(
        &TEST_ANCHOR_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &transfer_contract_amount1)],
    )]);

    deps.querier.with_gov_stakers(&[(
        &TEST_ANCHOR_GOV.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &StakerResponse {
                balance: Uint128::zero(),
                share: Uint128::zero(),
                locked_balance: vec![],
            },
        )],
    )]);

    let balance = Uint128::from(100_000_000u128);
    let share = Uint128::from(50_000_000u128);
    let locked_balance = vec![
        (
            1u64,
            VoterInfo {
                vote: VoteOption::Yes,
                balance: Uint128::from(100u128),
            },
        ),
        (
            2u64,
            VoterInfo {
                vote: VoteOption::Yes,
                balance: Uint128::from(100u128),
            },
        ),
        (
            3u64,
            VoterInfo {
                vote: VoteOption::No,
                balance: Uint128::from(100u128),
            },
        ),
    ];
    deps.querier.with_gov_stakers(&[(
        &TEST_ANCHOR_GOV.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &StakerResponse {
                balance,
                share,
                locked_balance,
            },
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_ALICE.to_string(),
        amount: transfer_amount_alice,
        msg: to_binary(&Cw20HookMsg::StakingTokens {}).unwrap(),
    });
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // user check
    let feerate = feerate_read(deps.as_ref().storage).load().unwrap();
    let total_shares = total_shares_read(deps.as_ref().storage).load().unwrap();
    let key = deps.api.addr_canonicalize(TEST_ALICE).unwrap();
    let user_shares = user_states_read(deps.as_ref().storage)
        .may_load(&key.as_slice())
        .unwrap_or_default()
        .unwrap_or_default();

    assert_eq!(
        user_shares,
        transfer_amount_alice - (feerate * transfer_amount_alice)
    );

    let msg = QueryMsg::UserState {
        user: TEST_ALICE.to_string(),
    };
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();
    let user_state_response: UserStateResponse = from_binary(&res).unwrap();

    assert_eq!(
        user_state_response,
        UserStateResponse {
            available_balance: (balance - Uint128::from(300u128)) * user_shares / total_shares,
            shares: user_shares,
            locked_balance: Uint128::from(300u128) * user_shares / total_shares
        }
    )
}

#[test]
fn query_state() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let balance = Uint128::from(100_000_000u128);
    let share = Uint128::from(50_000_000u128);
    let locked_balance = vec![
        (
            1u64,
            VoterInfo {
                vote: VoteOption::Yes,
                balance: Uint128::from(100u128),
            },
        ),
        (
            2u64,
            VoterInfo {
                vote: VoteOption::Yes,
                balance: Uint128::from(100u128),
            },
        ),
        (
            3u64,
            VoterInfo {
                vote: VoteOption::No,
                balance: Uint128::from(100u128),
            },
        ),
    ];
    deps.querier.with_gov_stakers(&[(
        &TEST_ANCHOR_GOV.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &StakerResponse {
                balance,
                share,
                locked_balance,
            },
        )],
    )]);

    let msg = QueryMsg::State {};
    let res = query(deps.as_ref(), mock_env(), msg).unwrap();

    let state_response: StateResponse = from_binary(&res).unwrap();

    assert_eq!(
        state_response,
        StateResponse {
            feerate: Decimal::percent(DEFAULT_FEERATE),
            locked_balance: Uint128::from(300u128),
            available_balance: balance - Uint128::from(300u128),
            total_shares: Uint128::zero(),
        }
    );
}
