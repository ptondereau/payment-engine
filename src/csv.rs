use rust_decimal::Decimal;
use serde::Deserialize;
use tokio::{
    fs::File,
    io::{AsyncWrite, AsyncWriteExt},
};

use crate::{account::AccountId, errors::Result, transaction::TransactionId};

#[derive(Debug, Clone, Deserialize)]
pub struct TransactionRecord {
    /// `type` is a foreign keyword.
    #[serde(rename = "type")]
    type_: TransactionRecordType,
    client: AccountId,
    tx: TransactionId,
    amount: Option<Decimal>,
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

pub async fn send_accounts_csv_to_stdout<T: AsyncWrite + Unpin>(
    file: File,
    mut output: T,
) -> Result<()> {
    let mut csv_reader = csv_async::AsyncReaderBuilder::new()
        .trim(csv_async::Trim::All)
        .flexible(true)
        .create_deserializer(file);

    // pop out the headers.
    let headers = csv_reader.byte_headers().await?.clone();
    let mut record = csv_async::ByteRecord::new();
    while csv_reader.read_byte_record(&mut record).await? {
        let deserialize_record = record.deserialize::<TransactionRecord>(Some(&headers))?;

        output.write_all(record.as_slice()).await?;
    }

    Ok(())
}
