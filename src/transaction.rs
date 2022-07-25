/// This module contains transaction business encapsulation.
/// This will help us to have a clean code and intention.
use crate::account::AccountId;
use rust_decimal::Decimal;
use std::fmt::{Display, Formatter, Result};

pub type TransactionId = u32;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TransactionKind {
    Deposit,
    Withdrawal,
}

impl Display for TransactionKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            TransactionKind::Deposit => write!(f, "Deposit"),
            TransactionKind::Withdrawal => write!(f, "Withdrawal"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionStatus {
    ChargedBack,
    Created,
    DisputeInProgress,
    Processed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    kind: TransactionKind,
    id: TransactionId,
    account_id: AccountId,
    amount: Decimal,
    pub status: TransactionStatus,
}

impl Transaction {
    pub fn new(
        kind: TransactionKind,
        id: TransactionId,
        account_id: AccountId,
        amount: Decimal,
    ) -> Self {
        Self {
            kind,
            id,
            account_id,
            amount,
            status: TransactionStatus::Created,
        }
    }

    pub fn kind(&self) -> TransactionKind {
        self.kind
    }

    pub fn id(&self) -> TransactionId {
        self.id
    }

    pub fn account_id(&self) -> AccountId {
        self.account_id
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DisputeResolution {
    Cancelled,
    ChargedBack,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DisputeStatus {
    Created,
    InProgress,
    Resolved(DisputeResolution),
}

/// Represents a line as a business case of a dispute.
#[derive(Debug, Clone, PartialEq)]
pub struct Dispute {
    account_id: AccountId,
    tx_id: TransactionId,
    pub status: DisputeStatus,
}

impl Dispute {
    pub fn new(account_id: AccountId, tx_id: TransactionId) -> Self {
        Self {
            account_id,
            tx_id,
            status: DisputeStatus::Created,
        }
    }

    pub fn account_id(&self) -> AccountId {
        self.account_id
    }

    pub fn tx_id(&self) -> TransactionId {
        self.tx_id
    }
}
