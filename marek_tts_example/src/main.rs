use marek_tts_audio_output::TtsAudioOutput;
use marek_tts_client::TtsClient;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //println!("Connecting to the server...");

    let mut tts_client = TtsClient::connect("127.0.0.1:9999").await?;

    //println!("Enumerating voices...");
    //let voices = tts_client.enumerate_voices().await?;
    //println!("{:?}", voices);

    //let voice_name = "Claribel Dervla";
    //let voice_name = "Daisy Studious";
    //let voice_name = "Gracie Wise";
    //let voice_name = "Tammie Ema";
    let voice_name = "Marcos Rudaski";

    tts_client
        .say(
            "Dzień dobry, witaj Rust! Co za miłe spotkanie!",
            voice_name,
            "XTTS2",
            "pl",
        )
        .await
        .unwrap();

    Ok(())
}
