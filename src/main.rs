use payment_engine::errors::{PaymentEngineError, Result};
use tokio::{
    fs::File,
    io::stdout,
    io::{AsyncWrite, AsyncWriteExt},
};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let csv_path = std::env::args().nth(1).ok_or_else(|| {
        PaymentEngineError::CommandLineError(format!(
            "Missing input file name. Usage: {} <filename>.csv",
            std::env::args().next().unwrap()
        ))
    })?;

    let csv_file = File::open(csv_path).await?;

    let mut stdout = stdout();
    send_accounts_csv_to_stdout(csv_file, &mut stdout).await?;

    Ok(())
}

pub async fn send_accounts_csv_to_stdout<T: AsyncWrite + Unpin>(
    file: File,
    mut output: T,
) -> Result<()> {
    let mut csv_reader = csv_async::AsyncReaderBuilder::new()
        .trim(csv_async::Trim::All)
        .flexible(true)
        .create_reader(file);

    let mut record = csv_async::ByteRecord::new();
    while csv_reader.read_byte_record(&mut record).await? {
        output.write_all(record.as_slice()).await?;
    }

    Ok(())
}
