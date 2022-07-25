use crate::account::AccountId;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PaymentEngineError>;

#[derive(Debug, Error, PartialEq)]
pub enum PaymentEngineError {
    /// Domain errors from an account process.
    #[error("Failed to process account operation: {0}")]
    AccountProcessError(#[from] AccountOperationError),

    /// Command line errors.
    #[error("Command line failed: {0}")]
    CommandLineError(String),

    #[error("Input/Output error: {0}")]
    InputOutpoutError(String),
}

impl From<std::io::Error> for PaymentEngineError {
    fn from(e: std::io::Error) -> Self {
        Self::InputOutpoutError(format!("{}", e))
    }
}

#[derive(Debug, Clone, Error, PartialEq)]
pub enum AccountOperationError {
    #[error("Insufficient funds in the wallet")]
    InsufficientFunds,

    #[error("Amount should be positive only")]
    NonPositiveAmount,

    #[error("Account {0} is locked")]
    AccountLocked(AccountId),

    #[error("Account's wallet has too much money")]
    OverflowInWallet,
}
