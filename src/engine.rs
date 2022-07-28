use std::collections::{HashMap, HashSet};

use tokio::{sync::mpsc, task::JoinHandle};

use crate::{
    account::{Account, AccountId},
    errors::Result,
    tasks::{
        command::{DisputeCommandData, PaymentEngineCommand, TransactionCommandData},
        worker::AccountWorker,
    },
    transaction::TransactionId,
};

#[derive(Debug)]
pub struct PaymentEngine {
    pub receiver: mpsc::Receiver<PaymentEngineCommand>,
    account_workers: HashMap<AccountId, mpsc::Sender<PaymentEngineCommand>>,
    worker_joins: Vec<(AccountId, JoinHandle<Result<()>>)>,
    processed_transaction_ids: HashSet<TransactionId>,
}

impl PaymentEngine {
    pub fn new(receiver: mpsc::Receiver<PaymentEngineCommand>) -> Self {
        Self {
            receiver,
            account_workers: HashMap::new(),
            worker_joins: Vec::new(),
            processed_transaction_ids: HashSet::new(),
        }
    }

    pub async fn handle(&mut self, cmd: PaymentEngineCommand) -> Result<()> {
        log::debug!("command received: {:?}", cmd);
        match cmd {
            PaymentEngineCommand::TransactionCommand(tx) => self.handle_transaction(tx).await,
            PaymentEngineCommand::DisputeCommand(d) => self.handle_dispute(d).await,
            PaymentEngineCommand::SendAccountsToCSV(sender) => {
                self.handle_send_accounts_to_csv(sender).await
            }
        }?;

        Ok(())
    }

    async fn handle_send_accounts_to_csv(&self, chan: mpsc::Sender<String>) -> Result<()> {
        chan.send(String::from("client,available,held,total,locked\n"))
            .await?;
        for (_, worker_sender) in self.account_workers.iter() {
            worker_sender
                .send(PaymentEngineCommand::SendAccountsToCSV(chan.clone()))
                .await?;
        }
        Ok(())
    }

    async fn handle_transaction(&mut self, cmd: TransactionCommandData) -> Result<()> {
        let transaction_id = cmd.tx.id();

        if self.processed_transaction_ids.contains(&transaction_id) {
            panic!("duplicated tx")
        }

        let account_id = cmd.tx.account_id();
        let send_cmd = PaymentEngineCommand::TransactionCommand(cmd);
        match self.account_workers.get(&account_id) {
            Some(s) => s.send(send_cmd).await?,
            None => self.create_account_worker(account_id, send_cmd).await?,
        }

        self.processed_transaction_ids.insert(transaction_id);

        Ok(())
    }

    async fn create_account_worker(
        &mut self,
        account_id: AccountId,
        cmd: PaymentEngineCommand,
    ) -> Result<()> {
        let (sender, receiver) = mpsc::channel(32);
        let mut account_worker = AccountWorker::new(receiver, Account::new(account_id));
        let join = tokio::spawn(async move {
            while let Some(cmd) = account_worker.receiver.recv().await {
                // Do not abort worker on command handling errors
                if let Err(e) = account_worker.handle(&cmd).await {
                    log::error!(
                        "AccountWorker with id: {} failed to handle command {:?}: {}",
                        account_worker.get_id(),
                        cmd,
                        e
                    );
                };
            }

            Ok(())
        });
        sender.send(cmd).await?;
        self.account_workers.insert(account_id, sender);
        self.worker_joins.push((account_id, join));
        Ok(())
    }

    async fn handle_dispute(&mut self, cmd: DisputeCommandData) -> Result<()> {
        let account_id = cmd.dispute.account_id();
        let send_cmd = PaymentEngineCommand::DisputeCommand(cmd);
        match self.account_workers.get(&account_id) {
            Some(s) => s.send(send_cmd).await?,
            None => self.create_account_worker(account_id, send_cmd).await?,
        }
        Ok(())
    }

    pub async fn shutdown(&mut self) {
        self.account_workers = HashMap::new();
        // Wait until all workers terminate gracefully
        while let Some((acc_id, join)) = self.worker_joins.pop() {
            match join.await {
                Ok(result) => {
                    if let Err(result_e) = result {
                        log::error!("Account worker {} tokio task failed: {}", acc_id, result_e);
                    }
                }
                Err(e) => log::error!("await Account worker {} failed: {}", acc_id, e),
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Result;
    use crate::transaction::{Transaction, TransactionKind};
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_engine_send_accounts_to_csv() -> Result<()> {
        let cmd = PaymentEngineCommand::TransactionCommand(
            Transaction::new(TransactionKind::Deposit, 0, 0, dec!(0)).try_into()?,
        );

        let (_, receiver) = mpsc::channel(2);
        let mut engine = PaymentEngine::new(receiver);

        engine.handle(cmd).await?;
        assert_eq!(engine.worker_joins.len(), 1);
        assert!(engine.account_workers.contains_key(&0));

        Ok(())
    }
}
