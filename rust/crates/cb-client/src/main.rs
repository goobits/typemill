//! cb-client main binary

use cb_client::run_cli;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_cli().await?;
    Ok(())
}