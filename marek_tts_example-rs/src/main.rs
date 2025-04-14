use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, Stream, StreamError,
};
use futures::{channel::oneshot, stream::TryStreamExt};
use futures_util::pin_mut;
use marek_tts_client_rs::TtsClient;
use std::{
    error::Error,
    sync::{mpsc::channel, Arc},
    time::Duration,
};
use std::{io::Write, sync::atomic::AtomicBool, sync::atomic::Ordering};
use tokio::time::sleep;

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

    let (sample_tx, sample_rx) = channel();
    let (finished_tx, finished_rx) = oneshot::channel();
    let mut finished_tx = Some(finished_tx);
    let no_more_data = Arc::new(AtomicBool::new(false));
    let no_more_data_clone = no_more_data.clone();

    let error_callback =
        |err: StreamError| eprintln!("an error occurred on the output audio stream: {}", err);
    let mut data_callback = Some(move |data: &mut [i16], info: &cpal::OutputCallbackInfo| {
        println!("New audio buffer: {} {:?}", data.len(), info);
        std::io::stdout().flush().unwrap();
        for (idx, sample) in data.iter_mut().enumerate() {
            if let Ok(s) = sample_rx.try_recv() {
                *sample = s;
            } else {
                *sample = 0i16;

                if no_more_data_clone.load(Ordering::Relaxed) {
                    if let Some(finished_tx) = finished_tx.take() {
                        let duration = info
                            .timestamp()
                            .playback
                            .duration_since(&info.timestamp().callback)
                            .unwrap()
                            + Duration::from_secs_f64((idx as f64) / (sample_rate as f64));
                        println!("Duration: {:?}", duration);
                        finished_tx.send(duration).unwrap();
                    }
                }
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
            sample_tx.send(*sample)?;
        }

        if let Some(data_callback) = data_callback.take() {
            println!("Open stream!");
            stream =
                Some(device.build_output_stream(&config, data_callback, error_callback, None)?);
            stream.as_ref().unwrap().play()?;
        }
    }

    println!("Has no more data!");
    no_more_data.store(true, Ordering::Relaxed);

    // wait for the signal from data_callback
    let playback_duration = finished_rx.await.unwrap();
    sleep(playback_duration).await;

    drop(stream);
    println!("Dropped stream!");

    Ok(())
}
