use marek_tts_client_rs::TtsClient;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Connecting to the server...");

    let mut tts_client = TtsClient::connect("127.0.0.1:9999").await?;

    Ok(())
}
