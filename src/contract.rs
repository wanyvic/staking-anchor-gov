#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, from_binary, to_binary, Addr, Attribute, Binary, CanonicalAddr, ContractResult,
    CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, QuerierWrapper, QueryRequest, Reply,
    Response, StdError, StdResult, SubMsg, Uint128, WasmMsg, WasmQuery,
};

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, ReplySendData,
    StateResponse, UserStateResponse, WITHDRAW_REPLY_ID,
};
use crate::state::{
    config_read, config_store, feerate_read, feerate_store, total_shares_read, total_shares_store,
    user_states_read, user_states_store, Config,
};

use anchor_token::gov::{
    Cw20HookMsg as GovCw20HookMsg, ExecuteMsg as GovExcuteMsg, QueryMsg as GovQueryMsg,
    StakerResponse,
};
use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg, Cw20ReceiveMsg};
// -> wasmd_18 tx wasm store target/wasm32-unknown-unknown/release/staking_anchor_gov.wasm --from main --node tcp://localhost:26657 --chain-id localnet --gas-prices 0.01ucosm --gas 1289204 --gas-adjustment 1.3 --keyring-backend test --home ~/.wasmd_test_keys
// -> wasmd_18 tx wasm instantiate 1 '{"fee_rate": "0.02","anchor_gov":"wasm1ytw7vjnt7qeduxa2s7u98uqau4p9f096javn6c", "anchor_token": "wasm166hsz0p746sr99qxdx66xglck85cqgjld2dxyl"}' --from main --label "test" --node tcp://localhost:26657 --chain-id localnet --gas-prices 0.01ucosm --gas auto --gas-adjustment 1.3 --keyring-backend test --home ~/.wasmd_test_keys
// <- {"height":"444","txhash":"758208312EF43C93E57E4EC56C99C06CD9CB7BA0877492D97E32C7FDD5F697DF","data":"0A3C0A0B696E7374616E7469617465122D0A2B7761736D3134686A32746176713866706573647778786375343472747933686839307668756A6771776733","raw_log":"[{\"events\":[{\"type\":\"instantiate\",\"attributes\":[{\"key\":\"_contract_address\",\"value\":\"wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujgqwg3\"},{\"key\":\"code_id\",\"value\":\"1\"}]},{\"type\":\"message\",\"attributes\":[{\"key\":\"action\",\"value\":\"instantiate\"},{\"key\":\"module\",\"value\":\"wasm\"},{\"key\":\"sender\",\"value\":\"wasm1ytw7vjnt7qeduxa2s7u98uqau4p9f096javn6c\"}]},{\"type\":\"wasm\",\"attributes\":[{\"key\":\"_contract_address\",\"value\":\"wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujgqwg3\"},{\"key\":\"method\",\"value\":\"instantiate\"},{\"key\":\"owner\",\"value\":\"wasm1ytw7vjnt7qeduxa2s7u98uqau4p9f096javn6c\"},{\"key\":\"fee_rate\",\"value\":\"0.02\"},{\"key\":\"anchor_token\",\"value\":\"wasm1ytw7vjnt7qeduxa2s7u98uqau4p9f096javn6c\"},{\"key\":\"anchor_gov\",\"value\":\"wasm1ytw7vjnt7qeduxa2s7u98uqau4p9f096javn6c\"}]}]}]","logs":[{"events":[{"type":"instantiate","attributes":[{"key":"_contract_address","value":"wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujgqwg3"},{"key":"code_id","value":"1"}]},{"type":"message","attributes":[{"key":"action","value":"instantiate"},{"key":"module","value":"wasm"},{"key":"sender","value":"wasm1ytw7vjnt7qeduxa2s7u98uqau4p9f096javn6c"}]},{"type":"wasm","attributes":[{"key":"_contract_address","value":"wasm14hj2tavq8fpesdwxxcu44rty3hh90vhujgqwg3"},{"key":"method","value":"instantiate"},{"key":"owner","value":"wasm1ytw7vjnt7qeduxa2s7u98uqau4p9f096javn6c"},{"key":"fee_rate","value":"0.02"},{"key":"anchor_token","value":"wasm1ytw7vjnt7qeduxa2s7u98uqau4p9f096javn6c"},{"key":"anchor_gov","value":"wasm1ytw7vjnt7qeduxa2s7u98uqau4p9f096javn6c"}]}]}],"gas_wanted":"149730","gas_used":"124816"}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        pendding_owner: None,
        dev: deps.api.addr_canonicalize(msg.dev.as_str())?,
        anchor_token: deps.api.addr_canonicalize(msg.anchor_token.as_str())?,
        anchor_gov: deps.api.addr_canonicalize(msg.anchor_gov.as_str())?,
    };

    // store value
    config_store(deps.storage).save(&config)?;

    feerate_store(deps.storage).save(&msg.feerate)?;

    total_shares_store(deps.storage).save(&Uint128::zero())?;

    // add event
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("feerate", msg.feerate.to_string())
        .add_attribute("anchor_token", msg.anchor_token)
        .add_attribute("anchor_gov", msg.anchor_gov)
        .add_attribute("dev", msg.dev))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, _env, info, msg),
        ExecuteMsg::TransferOwnerShip { new_owner } => set_pedding_owner(deps, info, new_owner),
        ExecuteMsg::AcceptOwner {} => accept_owner(deps, info),
        ExecuteMsg::UpdateDev { new_dev } => update_dev(deps, info, new_dev),
        ExecuteMsg::UpdateFeeRate { new_feerate } => update_feerate(deps, info, new_feerate),
        // TODO:
        ExecuteMsg::WithdrawToken { amount } => withdraw_token(deps, _env, info, amount),
    }
}

pub fn set_pedding_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let mut config: Config = config_read(deps.storage).load()?;
    // return Unauthorized, if sender not owner
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // if new_owner is empty, reset pendding owner.
    let mut new_pendding_owner: Option<CanonicalAddr> = None;
    if !new_owner.is_empty() {
        new_pendding_owner = Some(deps.api.addr_canonicalize(new_owner.as_str())?);
    }

    let old_pendding = if let Some(x) = config.pendding_owner {
        x.to_string()
    } else {
        String::default()
    };

    // store config
    config.pendding_owner = new_pendding_owner;
    config_store(deps.storage).save(&config)?;

    Ok(Response::new()
        .add_attribute("method", "set_pedding_owner")
        .add_attribute("old_pendding", old_pendding)
        .add_attribute("new_pendding", config.owner.to_string()))
}
pub fn accept_owner(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config: Config = config_read(deps.storage).load()?;

    let pedding_owner = config
        .pendding_owner
        .ok_or(ContractError::Unauthorized {})?;

    if deps.api.addr_canonicalize(info.sender.as_str())? != pedding_owner {
        return Err(ContractError::Unauthorized {});
    }

    let old_owner = config.owner;

    // store config
    config.owner = pedding_owner;
    config.pendding_owner = None;
    config_store(deps.storage).save(&config)?;

    Ok(Response::new()
        .add_attribute("method", "ownership_transferred")
        .add_attribute("old_owner", old_owner.to_string())
        .add_attribute("new_owner", config.owner.to_string()))
}

pub fn update_dev(
    deps: DepsMut,
    info: MessageInfo,
    new_dev: String,
) -> Result<Response, ContractError> {
    let mut config: Config = config_read(deps.storage).load()?;

    if deps.api.addr_canonicalize(info.sender.as_str())? != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let response = Response::new()
        .add_attribute("method", "update_dev")
        .add_attribute("old_dev", config.dev.to_string());

    // store config
    config.dev = deps.api.addr_canonicalize(new_dev.as_str())?;
    config_store(deps.storage).save(&config)?;

    Ok(response.add_attribute("new_dev", new_dev))
}

pub fn update_feerate(
    deps: DepsMut,
    info: MessageInfo,
    new_feerate: Decimal,
) -> Result<Response, ContractError> {
    let config: Config = config_read(deps.storage).load()?;
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.owner {
        return Err(ContractError::Unauthorized {});
    }
    let old_feerate = feerate_read(deps.storage).load()?;
    // feerate range check.
    let new_feerate = feerate_limits(new_feerate)?;

    // store feerate to state value.
    feerate_store(deps.storage).save(&new_feerate)?;

    Ok(Response::new()
        .add_attribute("method", "update_feerate")
        .add_attribute("old_feerate", old_feerate.to_string())
        .add_attribute("new_feerate", new_feerate.to_string()))
}
// feerate_limits to check new fee rate in range
fn feerate_limits(feerate: Decimal) -> Result<Decimal, ContractError> {
    if feerate > Decimal::one() || feerate < Decimal::zero() {
        return Err(ContractError::FeeRateOutOfLimits {});
    }
    Ok(feerate)
}

/// TODO: withdraw token from gov to user.
pub fn withdraw_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let user_raw = deps.api.addr_canonicalize(info.sender.as_str())?;
    let key = user_raw.as_slice();
    if let Some(mut user_shares) = user_states_read(deps.storage).may_load(key)? {
        let config: Config = config_read(deps.storage).load()?;
        let mut total_shares = total_shares_read(deps.storage).load()?;
        let (available_balance, locked_balance, _) = query_balance_from_gov(
            &deps.querier,
            deps.api.addr_humanize(&config.anchor_gov)?,
            env.contract.address,
        )?;
        let total_balance = available_balance + locked_balance;
        let withdraw_shares = amount
            .map(|v| {
                std::cmp::max(
                    v.multiply_ratio(total_shares, total_balance),
                    Uint128::from(1u128),
                )
            })
            .unwrap_or(user_shares);

        let available_user_shares = total_shares
            .multiply_ratio(available_balance, total_balance)
            .multiply_ratio(user_shares, total_shares);
        if withdraw_shares > available_user_shares {
            return Err(ContractError::InsufficientFunds {});
        }

        let withdraw_amount: Uint128 = withdraw_shares.multiply_ratio(total_balance, total_shares);
        user_shares -= withdraw_shares;
        total_shares -= withdraw_shares;

        user_states_store(deps.storage).save(&key, &user_shares)?;
        total_shares_store(deps.storage).save(&total_shares)?;

        Ok(Response::new()
            .add_submessage(SubMsg::reply_on_success(
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: deps.api.addr_humanize(&config.anchor_gov)?.to_string(),
                    msg: to_binary(&GovExcuteMsg::WithdrawVotingTokens {
                        amount: Some(withdraw_amount),
                    })?,
                    funds: vec![],
                }),
                WITHDRAW_REPLY_ID,
            ))
            .set_data(to_binary(&ReplySendData {
                recipient: info.sender.to_string(),
                amount: withdraw_amount,
            })?))
    } else {
        return Err(ContractError::NothingStaked {});
    }
}
/// when receive token, we will record it and re-invest to gov.
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let config: Config = config_read(deps.storage).load()?;
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.anchor_token {
        return Err(ContractError::Unauthorized {});
    }
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::StakingTokens {}) => {
            let api = deps.api;
            stake_tokens(
                deps,
                env,
                api.addr_validate(&cw20_msg.sender)?,
                cw20_msg.amount,
            )
        }
        _ => Err(ContractError::DataShouldBeGiven {}),
    }
}

fn stake_tokens(
    deps: DepsMut,
    env: Env,
    sender: Addr,
    mut amount: Uint128,
) -> Result<Response, ContractError> {
    if amount.is_zero() {
        return Err(ContractError::InsufficientFunds {});
    }
    let config: Config = config_read(deps.storage).load()?;
    let (available_balance, locked_balance, _) = query_balance_from_gov(
        &deps.querier,
        deps.api.addr_humanize(&config.anchor_gov)?,
        env.contract.address.clone(),
    )?;
    let deposited_balance = available_balance + locked_balance;
    let sender_address_raw = deps.api.addr_canonicalize(sender.as_str())?;
    let key = &sender_address_raw.as_slice();

    let mut user_shares = user_states_read(deps.storage)
        .may_load(key)?
        .unwrap_or_default();

    let mut total_shares = total_shares_read(deps.storage).load()?;

    let feerate = feerate_read(deps.storage).load()?;
    let mut dev_increase_share = Uint128::zero();
    let dev_amount = amount * feerate;
    if !dev_amount.is_zero() {
        amount = amount - dev_amount;
        dev_increase_share = deposit(dev_amount, deposited_balance, total_shares);
        let dev_key = &config.dev.as_slice();
        let mut dev_shares = user_states_read(deps.storage)
            .may_load(dev_key)?
            .unwrap_or_default();
        dev_shares += dev_increase_share;
        user_states_store(deps.storage).save(dev_key, &dev_shares)?;
    }
    let user_increase_share = deposit(amount, deposited_balance, total_shares);
    user_shares += user_increase_share;
    total_shares += dev_increase_share + user_increase_share;
    total_shares_store(deps.storage).save(&total_shares)?;
    user_states_store(deps.storage).save(key, &user_shares)?;
    let balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&config.anchor_token)?,
        env.contract.address,
    )?;
    let dev = deps.api.addr_humanize(&config.dev)?;
    send_tokens(
        deps,
        &config.anchor_token,
        &config.anchor_gov,
        balance,
        to_binary(&GovCw20HookMsg::StakeVotingTokens {}).unwrap(),
        vec![
            attr("method", "StakingTokens"),
            attr(sender.to_string(), amount.to_string()),
            attr(dev.to_string(), dev_amount.to_string()),
        ],
    )
}

fn deposit(amount: Uint128, total_balance: Uint128, total_shares: Uint128) -> Uint128 {
    if total_balance.is_zero() || total_shares.is_zero() {
        return amount;
    }
    amount.multiply_ratio(total_shares, total_balance)
}
fn send_tokens(
    deps: DepsMut,
    asset_token: &CanonicalAddr,
    recipient: &CanonicalAddr,
    amount: Uint128,
    msg: Binary,
    attr: Vec<Attribute>,
) -> Result<Response, ContractError> {
    let contract_human = deps.api.addr_humanize(asset_token)?.to_string();
    let recipient_human = deps.api.addr_humanize(recipient)?.to_string();
    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_human,
            msg: to_binary(&Cw20ExecuteMsg::Send {
                contract: recipient_human.clone(),
                amount: amount,
                msg: msg,
            })?,
            funds: vec![],
        })])
        .add_attributes(attr))
}
fn transfer_tokens(
    deps: DepsMut,
    asset_token: &CanonicalAddr,
    recipient: &CanonicalAddr,
    amount: Uint128,
    action: &str,
) -> Result<Response, ContractError> {
    let contract_human = deps.api.addr_humanize(asset_token)?.to_string();
    let recipient_human = deps.api.addr_humanize(recipient)?.to_string();

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_human,
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient_human.clone(),
                amount: amount,
            })?,
            funds: vec![],
        })])
        .add_attributes(vec![
            ("action", action),
            ("recipient", recipient_human.as_str()),
            ("amount", amount.to_string().as_str()),
        ]))
}
pub fn query_token_balance(
    querier: &QuerierWrapper,
    contract_addr: Addr,
    account_addr: Addr,
) -> StdResult<Uint128> {
    let res: Cw20BalanceResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&Cw20QueryMsg::Balance {
            address: account_addr.to_string(),
        })?,
    }))?;

    // load balance form the token contract
    Ok(res.balance)
}

pub fn query_balance_from_gov(
    querier: &QuerierWrapper,
    gov_addr: Addr,
    contract_addr: Addr,
) -> StdResult<(Uint128, Uint128, Uint128)> {
    let response: StakerResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: gov_addr.to_string(),
        msg: to_binary(&GovQueryMsg::Staker {
            address: contract_addr.to_string(),
        })?,
    }))?;
    let mut available_balance = response.balance;
    let mut locked_balance = Uint128::zero();
    for (_, x) in response.locked_balance.iter() {
        available_balance -= x.balance;
        locked_balance += x.balance;
    }
    return Ok((available_balance, locked_balance, response.share));
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&query_config(deps)?)?),
        QueryMsg::State {} => Ok(to_binary(&query_state(deps, _env)?)?),
        QueryMsg::UserState { user } => Ok(to_binary(&query_user_state(deps, _env, user)?)?),
    }
}
fn query_config(deps: Deps) -> Result<ConfigResponse, ContractError> {
    let config: Config = config_read(deps.storage).load()?;
    let pedding_owner = if let Some(x) = config.pendding_owner {
        deps.api.addr_humanize(&x)?.to_string()
    } else {
        String::default()
    };
    Ok(ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        pendding_owner: pedding_owner,
        dev: deps.api.addr_humanize(&config.dev)?.to_string(),
        anchor_token: deps.api.addr_humanize(&config.anchor_token)?.to_string(),
        anchor_gov: deps.api.addr_humanize(&config.anchor_gov)?.to_string(),
    })
}
/// query state of contract
fn query_state(deps: Deps, env: Env) -> Result<StateResponse, ContractError> {
    let config: Config = config_read(deps.storage).load()?;
    let total_shares = total_shares_read(deps.storage).load()?;
    let feerate = feerate_read(deps.storage).load()?;

    let (available_balance, locked_balance, _) = query_balance_from_gov(
        &deps.querier,
        deps.api.addr_humanize(&config.anchor_gov)?,
        env.contract.address,
    )?;

    Ok(StateResponse {
        total_shares: total_shares,
        feerate: feerate,
        locked_balance: locked_balance,
        available_balance: available_balance,
    })
}

fn query_user_state(
    deps: Deps,
    env: Env,
    user: String,
) -> Result<UserStateResponse, ContractError> {
    let total_shares = total_shares_read(deps.storage).load()?;
    if total_shares.is_zero() {
        return Ok(UserStateResponse {
            locked_balance: Uint128::zero(),
            available_balance: Uint128::zero(),
            shares: Uint128::zero(),
        });
    }
    let key = deps.api.addr_canonicalize(&user)?;
    let user_shares = user_states_read(deps.storage)
        .may_load(&key.as_slice())?
        .unwrap_or_default();

    let config: Config = config_read(deps.storage).load()?;
    let (available_balance, locked_balance, _) = query_balance_from_gov(
        &deps.querier,
        deps.api.addr_humanize(&config.anchor_gov)?,
        env.contract.address,
    )?;

    Ok(UserStateResponse {
        locked_balance: locked_balance * user_shares / total_shares,
        available_balance: available_balance * user_shares / total_shares,
        shares: user_shares,
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id == WITHDRAW_REPLY_ID {
        let res = msg.result.unwrap();
        match from_binary(&res.data.unwrap()) {
            Ok(ReplySendData { recipient, amount }) => {
                let config: Config = config_read(deps.storage).load()?;
                let recipient_addr = deps.api.addr_canonicalize(&recipient.as_str())?;
                return transfer_tokens(
                    deps,
                    &config.anchor_token,
                    &recipient_addr,
                    amount,
                    "transfer",
                );
            }
            _ => {}
        }
    }
    Err(ContractError::Std(StdError::generic_err(
        "not supported reply",
    )))
}
