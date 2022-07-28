use payment_engine::{
    csv::send_accounts_csv_to_stdout,
    engine::PaymentEngine,
    errors::{PaymentEngineError, Result},
    tasks::producer::TransactionProducer,
};

use tokio::{fs::File, io::stdout, sync::mpsc};

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

    let (engine_sender, engine_receiver) = mpsc::channel(512);
    let mut engine = PaymentEngine::new(engine_receiver);
    let engine_join = tokio::spawn(async move {
        while let Some(command) = engine.receiver.recv().await {
            // Do not abort engine on command processing errors
            if let Err(e) = engine.handle(command.clone()).await {
                log::error!(
                    "PaymentEngine: Failed to process command {:?}: {}",
                    command,
                    e
                );
            };
        }

        // Clean shutdown
        // see: https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html#clean-shutdown
        engine.shutdown().await;

        Ok(())
    });

    let producer = TransactionProducer::new(csv_file, engine_sender.clone());
    producer.start().await?;

    let mut stdout = stdout();
    send_accounts_csv_to_stdout(engine_sender, &mut stdout).await?;

    engine_join.await?
}
