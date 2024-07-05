use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Already on Escrow")]
    AlreadyOnEscrow {},

    #[error("Cannot cancel escrow because of not seller")]
    CannotCancelEscrowNotSeller {},

    #[error("Cannot cancel escrow because of no funds")]
    CannotCancelEscrowNoFunds {},

    #[error("NotOnEscrow")]
    NotOnEscrow {},

    #[error("NotStarted")]
    NotStarted {},

    #[error("Disabled")]
    Disabled {},

    #[error("Amount of the native coin inputed is zero")]
    NativeInputZero {},

    #[error("InvalidCw20Token")]
    InvalidCw20Token {},

    #[error("InvalidNativeToken")]
    InvalidNativeToken {},

    #[error("InvalidCw721Token")]
    InvalidCw721Token {},

    #[error("InvalidBuyer")]
    InvalidBuyer {},

    #[error("WrongPaymentAmount")]
    WrongPaymentAmount {},

    #[error("InvalidTokenReplyId")]
    InvalidTokenReplyId {},

    #[error("Cw20InputZero")]
    Cw20InputZero {},

    #[error("Cw721AlreadyLinked")]
    Cw721AlreadyLinked {},

    #[error("Incorrect funds")]
    IncorrectFunds {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("Insufficient Tokens")]
    InsufficientFund {},

    #[error("AlreadyCanceled")]
    AlreadyCanceled {},

    #[error("Wrong length")]
    WrongLength {},

    #[error("TokenTypeMismatch")]
    TokenTypeMismatch {},
}
