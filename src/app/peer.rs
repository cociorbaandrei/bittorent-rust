use anyhow::{Result, anyhow};
use tokio::net::TcpStream;
use tokio::io::{self};
use std::io::{Write, Read};
use crate::app::messages::Handshake;
use core::convert::TryInto;
use crate::app::messages::BTMessage;
use futures::stream::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::app::network::discover_peers;
use crate::app::tracker::MetaData;


pub struct PeerManager {
    peers: Vec<(String, u16)>,
    torrent: MetaData,
    handshake_received: bool
}

impl PeerManager{
    pub(crate) async fn new(torrent : MetaData) -> Result<Self> {
        let peers = discover_peers(&torrent).await?;
        println!("Piece len {} total {}", torrent.info.piece_length, torrent.info.length);
        Ok(Self{
            peers,
            torrent,
            handshake_received: false
        })
    }

    pub(crate) async fn connect_to_peer(&mut self) -> Result<TcpStream> {
        let (peer_ip, peer_port) = self.peers.iter().next().ok_or(anyhow!("Failed to get first peer"))?;
        let handshake = Handshake::new(b"00112233445566778899", &self.torrent.raw().info_hash_u8()?);
        let mut stream = connect_to_peer((peer_ip, *peer_port), handshake).await;
        let (data, stream) = read_exact_bytes(stream?, 68).await?;
        let peer_handshake = Handshake::deserialize(&data[..68]);
        println!("Received peer handshake: {}", peer_handshake);
        println!("Peer ID: {}", peer_handshake.peer_id());
        self.handshake_received = true;
        Ok(stream)
    }

    pub(crate) fn finished_handshake(&self) -> bool {
        self.handshake_received
    }

    pub(crate) async fn process_messages(&mut self) -> Result<()> {
        Ok(())
    }


}
pub async fn dispatch( message: BTMessage, stream: &mut TcpStream) -> Result<()>{

    match message {
        BTMessage::Choke => {},
        BTMessage::Unchoke => {
            for i in 0..3{
                let piece_length= 262144;
                const BLOCK_SIZE: u32 = 16 * 1024; // 16 KiB in bytes

                let total_blocks = (piece_length as f32 / BLOCK_SIZE as f32).ceil() as u32;

                for block_index in 0..total_blocks {
                    let begin = block_index * BLOCK_SIZE;
                    let length = if block_index == total_blocks - 1 {
                        // Last block, calculate remaining bytes
                        piece_length - begin
                    } else {
                        // All blocks except the last one are of BLOCK_SIZE
                        BLOCK_SIZE
                    };

                    let r =  BTMessage::Request(i, begin, length);
                    let _ = stream.write_all(&r.serialize()?).await?;
                }
            }

        },
        BTMessage::Interested => {},
        BTMessage::NotInterested => {},
        BTMessage::Have(piece) => {},
        BTMessage::Bitfield(bitfield) => {
            let intr = BTMessage::Interested;
            let _ = stream.write_all(&intr.serialize()?).await?;
        },
        BTMessage::Request(idx, offset, length) => {},
        BTMessage::Piece(idx, offset, data) => {
            println!("Piece: {} {} {:?} ", idx, offset, &data[0..5]);
        },
        BTMessage::Cancel(idx, offset, length) => {},
    };
    Ok(())
}
// Now the function takes the stream by value and returns it along with the read data
async fn read_exact_bytes(mut stream: TcpStream, num_bytes: usize) -> Result<(Vec<u8>, TcpStream)> {
    let mut buffer = vec![0u8; num_bytes];
    stream.read_exact(&mut buffer).await?;
    Ok((buffer, stream)) // Return both the buffer and the stream
}


pub(crate) async fn can_parse_message(buffer: &mut Vec<u8>) -> Result<Option<(u8, Vec<u8>)>> {
    if buffer.len() < 4 {
        // Not enough data to determine message length
        return Ok(None);
    }

    // Determine if we have a complete message based on length prefix
    let length_prefix = u32::from_be_bytes(buffer[0..4].try_into().unwrap());

    if (length_prefix as usize) + 4 <= buffer.len() {
        if buffer.len() == 4 {
            println!("Received keepalive message.");
            return Ok(None);
        }
        let message_type = buffer[4];
        let payload = buffer[5..(4 + length_prefix as usize)].to_vec();

        // Remove the processed message from the buffer
        buffer.drain(0..(4 + length_prefix as usize));

        Ok(Some((message_type, payload)))
    } else {
        // Complete message not yet received
        Ok(None)
    }
}



pub async fn connect_to_peer(peer: (&str, u16), handshake: Handshake) -> Result<TcpStream>{
    let (ip, port) = peer;
    let address = format!("{}:{}", ip, port);
    let mut stream = TcpStream::connect(&address).await
        .map_err(|e| anyhow!("Failed to connect to peer {}: {}", address, e))?;
    let bytes = &handshake.serialize();
    stream.write_all(&bytes).await
        .map_err(|e| anyhow!("Failed to write handshake to peer {}: {}", address, e))?;


    Ok(stream)
}