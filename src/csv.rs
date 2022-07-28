use rust_decimal::Decimal;
use serde::Deserialize;
use tokio::{
    io::{AsyncWrite, AsyncWriteExt},
    sync::mpsc,
};

use crate::{
    account::AccountId,
    errors::{PaymentEngineError, Result},
    tasks::command::{DisputeCommandAction, DisputeCommandData, PaymentEngineCommand},
    transaction::{Dispute, Transaction, TransactionId, TransactionKind},
};

#[derive(Debug, Clone, Deserialize)]
pub struct TransactionRecord {
    /// `type` is a foreign keyword.
    #[serde(rename = "type")]
    type_: TransactionRecordType,
    client: AccountId,
    tx: TransactionId,
    amount: Decimal,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionRecordType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl TryInto<PaymentEngineCommand> for TransactionRecord {
    type Error = PaymentEngineError;

    fn try_into(self) -> std::result::Result<PaymentEngineCommand, Self::Error> {
        match self.type_ {
            TransactionRecordType::Deposit => {
                let tx =
                    Transaction::new(TransactionKind::Deposit, self.tx, self.client, self.amount);
                Ok(PaymentEngineCommand::TransactionCommand(tx.into()))
            }
            TransactionRecordType::Withdrawal => {
                let tx = Transaction::new(
                    TransactionKind::Withdrawal,
                    self.tx,
                    self.client,
                    self.amount,
                );
                Ok(PaymentEngineCommand::TransactionCommand(tx.into()))
            }
            TransactionRecordType::Dispute => {
                let d = Dispute::new(self.client, self.tx);
                let cmd = DisputeCommandData::new(DisputeCommandAction::OpenDispute, d);
                Ok(PaymentEngineCommand::DisputeCommand(cmd))
            }
            TransactionRecordType::Resolve => {
                let d = Dispute::new(self.client, self.tx);
                let cmd = DisputeCommandData::new(DisputeCommandAction::CancelDispute, d);
                Ok(PaymentEngineCommand::DisputeCommand(cmd))
            }
            TransactionRecordType::Chargeback => {
                let d = Dispute::new(self.client, self.tx);
                let cmd = DisputeCommandData::new(DisputeCommandAction::ChargebackDispute, d);
                Ok(PaymentEngineCommand::DisputeCommand(cmd))
            }
        }
    }
}

pub async fn send_accounts_csv_to_stdout<T: AsyncWrite + Unpin>(
    engine_sender: mpsc::Sender<PaymentEngineCommand>,
    mut output: T,
) -> Result<()> {
    let (csv_sender, mut csv_receiver) = mpsc::channel(12);
    engine_sender
        .send(PaymentEngineCommand::SendAccountsToCSV(csv_sender))
        .await?;

    while let Some(record) = csv_receiver.recv().await {
        output.write_all(record.as_bytes()).await?;
    }

    output.flush().await?;

    Ok(())
}
