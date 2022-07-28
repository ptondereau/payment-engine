use std::convert::Infallible;

use crate::account::AccountId;
use thiserror::Error;
use tokio::sync::mpsc;

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

    #[error("CSV reader error: {0}")]
    CSVReaderError(String),

    #[error("TokioMpscError: {0}")]
    TokioMpscError(String),
}

impl From<std::io::Error> for PaymentEngineError {
    fn from(e: std::io::Error) -> Self {
        Self::InputOutpoutError(format!("{}", e))
    }
}

impl From<csv_async::Error> for PaymentEngineError {
    fn from(e: csv_async::Error) -> Self {
        Self::CSVReaderError(format!("{}", e))
    }
}

impl<T> From<mpsc::error::SendError<T>> for PaymentEngineError {
    fn from(e: mpsc::error::SendError<T>) -> Self {
        Self::TokioMpscError(format!("Error with PaymentsEngineCommand: {}", e))
    }
}

impl From<Infallible> for PaymentEngineError {
    fn from(e: Infallible) -> Self {
        Self::AccountProcessError(AccountOperationError::InfallibleError(format!(
            "Infaillible error: {}",
            e
        )))
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

    #[error("Infallible error: {0}")]
    InfallibleError(String),

    #[error("Wrong account id in worker's command: {0} {1}")]
    WrongAccountId(AccountId, AccountId),
}
