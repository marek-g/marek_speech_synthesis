use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, Stream, StreamError,
};
use futures::stream::TryStreamExt;
use futures_util::pin_mut;
use marek_tts_client_rs::TtsClient;
use std::io::Write;
use std::{error::Error, sync::mpsc::channel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Connecting to the server...");

    let mut tts_client = TtsClient::connect("127.0.0.1:9999").await?;

    println!("Enumerating voices...");

    let voices = tts_client.enumerate_voices().await?;
    println!("{:?}", voices);

    //let voice_name = "Claribel Dervla";
    //let voice_name = "Daisy Studious";
    //let voice_name = "Gracie Wise";
    //let voice_name = "Tammie Ema";
    let voice_name = "Marcos Rudaski";
    let sample_rate = voices
        .iter()
        .find(|voice| voice.voice == voice_name)
        .map(|voice| voice.sample_rate)
        .unwrap();

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no audio output device available");

    let supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");

    let config = supported_configs_range
        .into_iter()
        .find(|config| config.channels() == 1 && config.sample_format() == SampleFormat::I16)
        .expect("no supported config?!")
        .with_sample_rate(cpal::SampleRate(sample_rate))
        .config();

    let (tx, rx) = channel();

    let error_callback =
        |err: StreamError| eprintln!("an error occurred on the output audio stream: {}", err);
    let mut data_callback = Some(move |data: &mut [i16], _info: &cpal::OutputCallbackInfo| {
        println!("New audio buffer: {} {:?}", data.len(), _info);
        std::io::stdout().flush().unwrap();
        for sample in data.iter_mut() {
            if let Ok(s) = rx.try_recv() {
                *sample = s;
            } else {
                *sample = 0i16;

                // TODO: if "Has no more data!" then signal end of the playback
                // in _info.playback - info.callback (+ position in data * samples_per_sec?)
                //println!("No new data!");
            }
        }
    });
    let mut stream: Option<Stream> = None;

    let audio = tts_client.tts_stream(
        "Dzień dobry, witaj Rust! Co za miłe spotkanie!",
        voice_name,
        "XTTS2",
        "pl",
    );

    pin_mut!(audio);
    while let Some(chunk) = audio.try_next().await? {
        println!("Has data!");
        std::io::stdout().flush().unwrap();
        for sample in chunk.iter() {
            //println!("Send data!");
            std::io::stdout().flush().unwrap();
            tx.send(*sample)?;
        }

        if let Some(data_callback) = data_callback.take() {
            println!("Open stream!");
            stream =
                Some(device.build_output_stream(&config, data_callback, error_callback, None)?);
            stream.as_ref().unwrap().play()?;
        }
    }

    println!("Has no more data!");

    // TODO: wait for the signal from data_callback
    std::thread::sleep(std::time::Duration::from_millis(3000));

    drop(stream);
    println!("Dropped stream!");

    Ok(())
}
