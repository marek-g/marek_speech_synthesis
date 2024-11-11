use std::error::Error;

use async_stream::try_stream;
use byteorder::{ByteOrder, LittleEndian};
use futures_core::stream::Stream;
use hex::FromHex;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream, ToSocketAddrs};

pub struct TtsClient {
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
}

#[derive(Debug, Deserialize)]
pub struct Voice {
    voice: String,
    engine: String,
    languages: Vec<String>,
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
    result_code: i32,
    description: Option<String>,
    sample_rate: Option<u32>,
    chunk_size: Option<usize>,
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
        Ok(serde_json::from_str(&line)?)
    }

    pub fn tts_stream(
        &mut self,
        text: &str,
        voice: &str,
        engine: &str,
        language: &str,
    ) -> impl Stream<Item = Result<Vec<i16>, Box<dyn Error + 'static>>> + '_ {
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
            let response: TtsStreamResponse = serde_json::from_str(&line)?;
            //println!("{:?}", response);
            if response.result_code != 0 {
                let err: Box<dyn Error + 'static> = response.description.unwrap().into();
                Err(err)?;
            }

        let chunk_size = response.chunk_size.unwrap();
            if chunk_size == 0 {
                return;
            }

            let data = response.data.unwrap();
            let buffer_u8 = <Vec<u8>>::from_hex(data).unwrap();
            let mut buffer_i16 = vec![0; buffer_u8.len() / 2];
            LittleEndian::read_i16_into(&buffer_u8, &mut buffer_i16);

            yield buffer_i16;

            self.writer
                .write_all(b"y\n").await?;
            self.writer.flush().await?;
                }
            }
    }
}
