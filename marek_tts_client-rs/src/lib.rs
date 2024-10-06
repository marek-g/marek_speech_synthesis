use std::error::Error;

use tokio::net::{TcpStream, ToSocketAddrs};

pub struct TtsClient {
    stream: TcpStream,
}

impl TtsClient {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self, Box<dyn Error>> {
        Ok(TtsClient {
            stream: TcpStream::connect(addr).await?,
        })
    }
}
