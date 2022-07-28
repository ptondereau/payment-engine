use tokio::{io::AsyncRead, sync::mpsc};

use crate::{csv::TransactionRecord, errors::Result};

use super::command::PaymentEngineCommand;

pub struct TransactionProducer<R: AsyncRead + Unpin + Send> {
    reader: R,
    engine_sender: mpsc::Sender<PaymentEngineCommand>,
}

impl<R: AsyncRead + Unpin + Send> TransactionProducer<R> {
    pub fn new(reader: R, engine_sender: mpsc::Sender<PaymentEngineCommand>) -> Self {
        Self {
            reader,
            engine_sender,
        }
    }

    pub async fn start(self) -> Result<()> {
        let mut rdr = csv_async::AsyncReaderBuilder::new()
            .trim(csv_async::Trim::All)
            .create_reader(self.reader);

        let headers = rdr.byte_headers().await?.clone();
        let mut record = csv_async::ByteRecord::new();
        while rdr.read_byte_record(&mut record).await? {
            let tx_record: TransactionRecord = record.deserialize(Some(&headers))?;
            match tx_record.clone().try_into() {
                Ok(cmd) => self.engine_sender.send(cmd).await?,
                Err(e) => {
                    // Do not abort producer on parsing errors
                    log::error!("Failed to process record {:?}: {}", tx_record, e);
                }
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::transaction::{Transaction, TransactionKind};

    use super::*;

    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_deserialize_csv() -> Result<()> {
        let tests = vec![
            b"\
type,client,tx,amount
deposit,1,1,1.664
"
            .as_slice(),
            b"\
type,       client,    tx,  amount
deposit,         1,     1,  1.664
"
            .as_slice(),
        ];

        let expected_tx_cmd =
            Transaction::new(TransactionKind::Deposit, 1, 1, dec!(1.664)).try_into()?;
        for data in tests.into_iter() {
            let (sender, mut receiver) = mpsc::channel(1);
            let producer = TransactionProducer::new(data, sender);

            producer.start().await?;

            let cmd = receiver.recv().await.expect("cmd has not been received");
            match cmd {
                PaymentEngineCommand::TransactionCommand(tx_cmd) => {
                    assert_eq!(tx_cmd, expected_tx_cmd)
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }
}
