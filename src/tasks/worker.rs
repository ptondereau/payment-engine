use std::collections::HashMap;

use tokio::sync::mpsc;

use crate::{
    account::Account,
    errors::{AccountOperationError::WrongAccountId, PaymentEngineError, Result},
    transaction::{Transaction, TransactionId},
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
                    TransactionCommandAction::Deposit => todo!(),
                    TransactionCommandAction::Withdraw => todo!(),
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
}
