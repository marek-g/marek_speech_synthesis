use futures::stream::TryStreamExt;
use futures_util::pin_mut;
use marek_tts_client_rs::TtsClient;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Connecting to the server...");

    let mut tts_client = TtsClient::connect("127.0.0.1:9999").await?;

    println!("Enumerating voices...");

    let voices = tts_client.enumerate_voices().await?;
    println!("{:?}", voices);

    let audio = tts_client.tts_stream("Dzień dobry!", "Claribel Dervla", "XTTS2", "pl");
    pin_mut!(audio);
    while let Some(chunk) = audio.try_next().await? {
        println!("Has data!");
        //println!("{:?}", chunk);
    }

    Ok(())
}
