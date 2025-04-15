use std::error::Error;

use async_stream::try_stream;
use byteorder::{ByteOrder, LittleEndian};
use futures_core::stream::Stream;
use hex::FromHex;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream, ToSocketAddrs};

pub struct TtsClient {
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
}

#[derive(Debug, Deserialize)]
pub struct Voice {
    pub voice: String,
    pub engine: String,
    pub languages: Vec<String>,
    pub sample_rate: u32,
}

#[derive(Debug)]
pub struct AudioChunk {
    pub sample_rate: u32,
    pub samples: Vec<i16>,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error_code: i32,
    error_description: String,
}

#[derive(Serialize)]
struct TtsStreamRequest<'a> {
    method: &'a str,
    text: &'a str,
    voice: &'a str,
    engine: &'a str,
    language: &'a str,
}

#[derive(Debug, Deserialize)]
struct TtsStreamResponse {
    sample_rate: u32,
    chunk_size: usize,
    data: Option<String>,
}

impl TtsClient {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self, Box<dyn Error>> {
        let stream = TcpStream::connect(addr).await?;
        let (read, write) = stream.into_split();
        let reader = BufReader::new(read);
        let writer = BufWriter::new(write);
        Ok(TtsClient { reader, writer })
    }

    pub async fn enumerate_voices(&mut self) -> Result<Vec<Voice>, Box<dyn Error>> {
        self.writer
            .write_all(b"{ \"method\": \"enumerate_voices\" }\n")
            .await?;
        self.writer.flush().await?;

        let mut line = String::new();
        self.reader.read_line(&mut line).await?;

        if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&line) {
            Err(error_response.error_description.into())
        } else {
            Ok(serde_json::from_str(&line)?)
        }
    }

    pub fn tts_stream(
        &mut self,
        text: &str,
        voice: &str,
        engine: &str,
        language: &str,
    ) -> impl Stream<Item = Result<AudioChunk, Box<dyn Error + 'static>>> + '_ {
        let request = TtsStreamRequest {
            method: "tts_stream",
            text,
            voice,
            engine,
            language,
        };
        let request = serde_json::to_string(&request);

        try_stream! {
                let request = request?;
                //println!("{}", request);
                self.writer.write_all((request + "\n").as_bytes()).await?;
                self.writer.flush().await?;

                loop {
            let mut line = String::new();
            self.reader.read_line(&mut line).await?;

            if let Ok(error_response) = serde_json::from_str::<ErrorResponse>(&line) {
                        let err: Box<dyn Error + 'static> = error_response.error_description.into();
                        Err(err)?;
            }

            let response: TtsStreamResponse = serde_json::from_str(&line)?;
            let chunk_size = response.chunk_size;
            if let Some(data) = response.data {
                let buffer_u8 = <Vec<u8>>::from_hex(data).unwrap();
            let mut buffer_i16 = vec![0; buffer_u8.len() / 2];
            LittleEndian::read_i16_into(&buffer_u8, &mut buffer_i16);

        yield AudioChunk { sample_rate: response.sample_rate, samples: buffer_i16 };

            self.writer
                .write_all(b"y\n").await?;
            self.writer.flush().await?;
            } else {
            return;
        }
                }
            }
    }
}
