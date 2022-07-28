use std::collections::HashMap;
use tokio::sync::mpsc;

use crate::{
    account::Account,
    errors::{
        AccountOperationError::{DuplicatedTransaction, WrongAccountId},
        PaymentEngineError, Result,
    },
    transaction::{Transaction, TransactionId, TransactionStatus},
};

use super::command::{DisputeCommandAction, PaymentEngineCommand, TransactionCommandAction};

pub struct AccountWorker {
    receiver: mpsc::Receiver<PaymentEngineCommand>,
    account: Account,
    transactions: HashMap<TransactionId, Transaction>,
}

impl AccountWorker {
    pub fn new(receiver: mpsc::Receiver<PaymentEngineCommand>, account: Account) -> Self {
        Self {
            receiver,
            account,
            transactions: HashMap::new(),
        }
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
                    DisputeCommandAction::OpenDispute => todo!(),
                    DisputeCommandAction::CancelDispute => todo!(),
                    DisputeCommandAction::ChargebackDispute => todo!(),
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
}
