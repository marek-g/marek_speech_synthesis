use std::error::Error;

use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
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
            .write_all(b"{ \"method\": \"enumerateVoices\" }\n")
            .await?;
        self.writer.flush().await?;

        let mut line = String::new();
        self.reader.read_line(&mut line).await?;
        Ok(serde_json::from_str(&line)?)
    }
}
