use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::{
    account::{Account, AccountId},
    errors::{
        AccountOperationError::{self, DuplicatedTransaction, WrongAccountId},
        PaymentEngineError, Result,
    },
    transaction::{
        Dispute, DisputeResolution, DisputeStatus, Transaction, TransactionId, TransactionKind,
        TransactionStatus,
    },
};

use super::command::{DisputeCommandAction, PaymentEngineCommand, TransactionCommandAction};

pub struct AccountWorker {
    pub receiver: mpsc::Receiver<PaymentEngineCommand>,
    account: Account,
    transactions: HashMap<TransactionId, Transaction>,
    disputes: HashMap<TransactionId, Dispute>,
}

impl AccountWorker {
    pub fn new(receiver: mpsc::Receiver<PaymentEngineCommand>, account: Account) -> Self {
        Self {
            receiver,
            account,
            transactions: HashMap::new(),
            disputes: HashMap::new(),
        }
    }

    pub fn get_id(&self) -> AccountId {
        self.account.get_id()
    }

    pub async fn handle(&mut self, command: &PaymentEngineCommand) -> Result<()> {
        let result = match command {
            PaymentEngineCommand::TransactionCommand(ref sub_command) => {
                if sub_command.tx.account_id() != self.account.get_id() {
                    return Err(
                        WrongAccountId(sub_command.tx.account_id(), self.account.get_id()).into(),
                    );
                }

                match sub_command.action {
                    TransactionCommandAction::Deposit => self.handle_deposit(&sub_command.tx),
                    TransactionCommandAction::Withdraw => self.handle_withdrawal(&sub_command.tx),
                }
            }
            PaymentEngineCommand::DisputeCommand(ref sub_command) => {
                if sub_command.dispute.account_id() != self.account.get_id() {
                    return Err(WrongAccountId(
                        sub_command.dispute.account_id(),
                        self.account.get_id(),
                    )
                    .into());
                }

                match sub_command.action {
                    DisputeCommandAction::OpenDispute => {
                        self.handle_new_dispute(&sub_command.dispute)
                    }
                    DisputeCommandAction::CancelDispute => self
                        .handle_close_dispute(&sub_command.dispute, DisputeResolution::Cancelled),
                    DisputeCommandAction::ChargebackDispute => self
                        .handle_close_dispute(&sub_command.dispute, DisputeResolution::ChargedBack),
                }
            }
            PaymentEngineCommand::SendAccountsToCSV(sender) => {
                sender.send(format!("{}", self.account)).await?;
                Ok(())
            }
        };

        result.map_err(PaymentEngineError::from)
    }

    pub fn handle_deposit(&mut self, transaction: &Transaction) -> Result<()> {
        if self.transactions.get(&transaction.id()).is_some() {
            return Err(DuplicatedTransaction(transaction.id()).into());
        }

        self.account.deposit(transaction.amount())?;
        let mut tx = transaction.clone();
        tx.status = TransactionStatus::Processed;
        self.transactions.insert(tx.id(), tx);

        Ok(())
    }

    pub fn handle_withdrawal(&mut self, transaction: &Transaction) -> Result<()> {
        if self.transactions.get(&transaction.id()).is_some() {
            return Err(DuplicatedTransaction(transaction.id()).into());
        }

        self.account.withdraw(transaction.amount())?;
        let mut tx: Transaction = transaction.clone();
        tx.status = TransactionStatus::Processed;
        self.transactions.insert(tx.id(), tx);
        Ok(())
    }

    pub fn handle_new_dispute(&mut self, d: &Dispute) -> Result<()> {
        let disputed_tx = self
            .transactions
            .get_mut(&d.tx_id())
            .ok_or_else(|| AccountOperationError::TransactionNotFound(d.tx_id()))?;

        if disputed_tx.kind() != TransactionKind::Deposit {
            return Err(AccountOperationError::DisputeIsNotDeposit(disputed_tx.kind()).into());
        }

        if disputed_tx.status != TransactionStatus::Processed {
            let reason = match disputed_tx.status {
                TransactionStatus::Created => "has not been processed yet",
                TransactionStatus::DisputeInProgress => "has another dispute in progress",
                TransactionStatus::ChargedBack => "has been charged back",
                // Use empty str instead of `unreachable!()` macro to avoid panics that might lead
                // to inconsistent state or crash loops. In the worst case we can tolerate non-expressive
                // error message.
                TransactionStatus::Processed => "",
            };
            return Err(
                AccountOperationError::TransactionStateMismatch(disputed_tx.id(), reason).into(),
            );
        }

        self.account.hold(disputed_tx.amount())?;

        disputed_tx.status = TransactionStatus::DisputeInProgress;

        let mut updated_d: Dispute = d.clone();
        updated_d.status = DisputeStatus::InProgress;
        self.disputes.insert(disputed_tx.id(), updated_d);

        Ok(())
    }

    pub fn handle_close_dispute(
        &mut self,
        d: &Dispute,
        resolution: DisputeResolution,
    ) -> Result<()> {
        let disputed_tx = self
            .transactions
            .get_mut(&d.tx_id())
            .ok_or_else(|| AccountOperationError::TransactionNotFound(d.tx_id()))?;

        let stored_dispute = self
            .disputes
            .get_mut(&d.tx_id())
            .ok_or_else(|| AccountOperationError::TransactionDisputeNotFound(d.tx_id()))?;

        if stored_dispute.status != DisputeStatus::InProgress {
            let reason = match stored_dispute.status {
                DisputeStatus::Created => "has not been processed yet",
                DisputeStatus::Resolved(_) => "has already been resolved",
                // Use empty str instead of `unreachable!()` macro to avoid panics that might lead
                // to inconsistent state or crash loops. In the worst case we can tolerate non-expressive
                // error message.
                DisputeStatus::InProgress => "",
            };
            return Err(AccountOperationError::TransactionStateMismatch(
                stored_dispute.tx_id(),
                reason,
            )
            .into());
        }

        self.account.unhold(disputed_tx.amount())?;

        match resolution {
            DisputeResolution::Cancelled => {
                disputed_tx.status = TransactionStatus::Processed;
            }
            DisputeResolution::ChargedBack => {
                self.account.withdraw(disputed_tx.amount())?;
                disputed_tx.status = TransactionStatus::ChargedBack;
                self.account.locked = true;
            }
        }

        stored_dispute.status = DisputeStatus::Resolved(resolution);

        Ok(())
    }
}
