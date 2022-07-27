use payment_engine::{
    csv::send_accounts_csv_to_stdout,
    errors::{PaymentEngineError, Result},
};

use tokio::{fs::File, io::stdout};

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
