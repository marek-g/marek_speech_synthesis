use std::error::Error;
use std::future::Future;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamError};
use futures::channel::oneshot;
use futures::{pin_mut, TryStreamExt};
use marek_tts_client::TtsClient;
use tokio::time::sleep;

pub trait TtsAudioOutput {
    fn say(
        &mut self,
        text: &str,
        voice: &str,
        engine: &str,
        language: &str,
    ) -> impl Future<Output = Result<(), Box<dyn Error>>>;
}

impl TtsAudioOutput for TtsClient {
    async fn say(
        &mut self,
        text: &str,
        voice: &str,
        engine: &str,
        language: &str,
    ) -> Result<(), Box<dyn Error>> {
        // start generating speech data
        let audio = self.tts_stream(text, voice, engine, language);
        pin_mut!(audio);

        // start initializing audio output
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("no audio output device available");

        let supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");

        // retrieve first audio chunk to read sample rate
        let chunk = audio.try_next().await?;
        if chunk.is_none() {
            return Ok(());
        }
        let mut chunk = chunk.unwrap();
        let sample_rate = chunk.sample_rate;

        // continue initializing audio output
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
            //println!("New audio buffer: {} {:?}", data.len(), info);
            std::io::stdout().flush().unwrap();
            for (idx, sample) in data.iter_mut().enumerate() {
                if let Ok(s) = sample_rx.try_recv() {
                    *sample = s;
                } else {
                    *sample = 0i16;

                    if no_more_data_clone.load(Ordering::Relaxed) {
                        if let Some(finished_tx) = finished_tx.take() {
                            let mut presentation_duration = info
                                .timestamp()
                                .playback
                                .duration_since(&info.timestamp().callback)
                                .unwrap();

                            // from my experiments the latency on Alsa is about 23 ms
                            // less than reported (cpal 0.15)
                            presentation_duration =
                                presentation_duration.saturating_sub(Duration::from_millis(23));

                            let presentation_duration = presentation_duration
                                + Duration::from_secs_f64((idx as f64) / (sample_rate as f64));
                            //println!("Presentation duration: {:?}", presentation_duration);
                            finished_tx.send(presentation_duration).unwrap();
                        }
                    }
                }
            }
        });
        let mut stream: Option<Stream> = None;

        loop {
            //println!("Has data!");
            std::io::stdout().flush().unwrap();
            for sample in chunk.samples.iter() {
                //println!("Send data!");
                std::io::stdout().flush().unwrap();
                sample_tx.send(*sample)?;
            }

            if let Some(data_callback) = data_callback.take() {
                //println!("Open stream!");
                stream = Some(device.build_output_stream(
                    &config,
                    data_callback,
                    error_callback,
                    None,
                )?);
                stream.as_ref().unwrap().play()?;
            }

            if let Some(next_chunk) = audio.try_next().await? {
                chunk = next_chunk;
            } else {
                break;
            }
        }

        //println!("Has no more data!");
        no_more_data.store(true, Ordering::Relaxed);

        // wait for the signal from data_callback
        let playback_duration = finished_rx.await.unwrap();
        sleep(playback_duration).await;

        drop(stream);
        //println!("Dropped stream!");

        Ok(())
    }
}
