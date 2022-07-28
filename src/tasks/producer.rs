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
