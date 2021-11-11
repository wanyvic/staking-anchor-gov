use cosmwasm_std::{CanonicalAddr, Decimal, Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static KEY_CONFIG: &[u8] = b"config";
static KEY_FEERATE: &[u8] = b"feerate";
static KEY_TOTAL_SHARES: &[u8] = b"total_shares";
static KEY_USER_STATES: &[u8] = b"user_states";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub pendding_owner: Option<CanonicalAddr>,
    pub dev: CanonicalAddr,
    pub anchor_token: CanonicalAddr,
    pub anchor_gov: CanonicalAddr,
}

pub fn config_store(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, KEY_CONFIG)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, KEY_CONFIG)
}
pub fn feerate_store(storage: &mut dyn Storage) -> Singleton<Decimal> {
    singleton(storage, KEY_FEERATE)
}

pub fn feerate_read(storage: &dyn Storage) -> ReadonlySingleton<Decimal> {
    singleton_read(storage, KEY_FEERATE)
}

pub fn total_shares_store(storage: &mut dyn Storage) -> Singleton<Uint128> {
    singleton(storage, KEY_TOTAL_SHARES)
}

pub fn total_shares_read(storage: &dyn Storage) -> ReadonlySingleton<Uint128> {
    singleton_read(storage, KEY_TOTAL_SHARES)
}

pub fn user_states_read(storage: &dyn Storage) -> ReadonlyBucket<Uint128> {
    bucket_read(storage, KEY_USER_STATES)
}

pub fn user_states_store(storage: &mut dyn Storage) -> Bucket<Uint128> {
    bucket(storage, KEY_USER_STATES)
}
