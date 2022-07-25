use payment_engine::errors::{PaymentEngineError, Result};
use tokio::fs::File;

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

    println!("file: {:?}", csv_file);

    Ok(())
}
