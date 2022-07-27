/// The main role of having sub-command is just to a clear split of action foreach transaction type.
/// For example, when we encounter a dispute, we can open/cancel/chargeback.
use tokio::sync::mpsc;

use crate::transaction::{Dispute, Transaction, TransactionKind};

#[derive(Debug, Clone)]
pub enum PaymentEngineCommand {
    TransactionCommand(TransactionCommandData),
    DisputeCommand(DisputeCommandData),
    SendAccountsToCSV(mpsc::Sender<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionCommandData {
    action: TransactionCommandAction,
    pub tx: Transaction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionCommandAction {
    Deposit,
    Withdraw,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisputeCommandData {
    action: DisputeCommandAction,
    dispute: Dispute,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DisputeCommandAction {
    OpenDispute,
    CancelDispute,
    ChargebackDispute,
}

impl From<Transaction> for TransactionCommandData {
    fn from(transaction: Transaction) -> Self {
        let action = match transaction.kind() {
            TransactionKind::Deposit => TransactionCommandAction::Deposit,
            TransactionKind::Withdrawal => TransactionCommandAction::Withdraw,
        };
        Self {
            action,
            tx: transaction,
        }
    }
}
