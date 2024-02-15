use crate::app::messages::Handshake;
use crate::app::network::discover_peers;
use crate::app::tracker::MetaData;
use anyhow::{anyhow, Result};


use tokio::io::{self};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use tokio::fs::OpenOptions;
use tokio::io::AsyncSeekExt;
pub struct PeerManager {
    peers: Vec<(String, u16)>,
    pub torrent: MetaData,
    handshake_received: bool,
}

impl PeerManager {
    pub(crate) async fn new(torrent: MetaData) -> Result<Self> {
        let peers = discover_peers(&torrent).await?;
        // println!("Piece len {} total {}", torrent.info.piece_length, torrent.info.length);
        Ok(Self {
            peers,
            torrent,
            handshake_received: false,
        })
    }

    pub(crate) async fn connect_to_peer(&mut self) -> Result<TcpStream> {
        let (peer_ip, peer_port) = self
            .peers
            .first()
            .ok_or(anyhow!("Failed to get first peer"))?;
        let handshake =
            Handshake::new(b"00112233445566778899", &self.torrent.raw().info_hash_u8()?);
        let stream = connect_to_peer((peer_ip, *peer_port), handshake).await;
        let (data, stream) = read_exact_bytes(stream?, 68).await?;
        let peer_handshake = Handshake::deserialize(&data[..68]);
        //println!("Received peer handshake: {}", peer_handshake);
        println!("Peer ID: {}", peer_handshake.peer_id());
        self.handshake_received = true;
        Ok(stream)
    }
}

pub(crate) async fn write_at_offset(file_path: &str, offset: u64, data: &[u8]) -> io::Result<()> {
    // Open or create the file with write and read access
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(file_path)
        .await?;

    let mut file = file;

    // Seek to the specified offset
    file.seek(io::SeekFrom::Start(offset)).await?;

    // Write data starting from the specified offset
    file.write_all(data).await?;

    Ok(())
}

// Now the function takes the stream by value and returns it along with the read data
async fn read_exact_bytes(mut stream: TcpStream, num_bytes: usize) -> Result<(Vec<u8>, TcpStream)> {
    let mut buffer = vec![0u8; num_bytes];
    stream.read_exact(&mut buffer).await?;
    Ok((buffer, stream)) // Return both the buffer and the stream
}

pub async fn connect_to_peer(peer: (&str, u16), handshake: Handshake) -> Result<TcpStream> {
    let (ip, port) = peer;
    let address = format!("{}:{}", ip, port);
    let mut stream = TcpStream::connect(&address)
        .await
        .map_err(|e| anyhow!("Failed to connect to peer {}: {}", address, e))?;
    let bytes = &handshake.serialize();
    stream
        .write_all(bytes)
        .await
        .map_err(|e| anyhow!("Failed to write handshake to peer {}: {}", address, e))?;

    Ok(stream)
}
