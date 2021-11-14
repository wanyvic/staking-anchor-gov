use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    OverflowError(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("FeeRate out of limits")]
    FeeRateOutOfLimits {},

    #[error("Insufficient funds sent")]
    InsufficientFunds {},

    #[error("Data should be given")]
    DataShouldBeGiven {},

    #[error("Nothing staked")]
    NothingStaked {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
