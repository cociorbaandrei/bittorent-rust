
use std::fmt;
use std::fmt::format;
use tokio::io::{self, AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use anyhow::{anyhow, Result};
use bytes::{BytesMut, BufMut, Buf};
use tokio_util::codec::{Decoder, Encoder, Framed};

#[derive(Debug)]
pub enum BTMessage {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have(u32),
    Bitfield(String),
    Request(u32, u32, u32),
    Piece(u32, u32, Vec<u8>),
    Cancel(u32,u32,u32),
}

pub struct BTMessageFramer;

impl Encoder<BTMessage> for BTMessageFramer{
    type Error = anyhow::Error;
    fn encode(&mut self, item: BTMessage, dst: &mut BytesMut) -> std::result::Result<(), Self::Error> {
        let serialized = item.serialize()?;
        dst.extend_from_slice(&serialized);
        Ok(())
    }
}
impl Decoder for BTMessageFramer{
    type Item = BTMessage;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        if src.len() < 4 {
            // Not enough data to determine message length
            return Ok(None);
        }
        // Determine if we have a complete message based on length prefix
        let length_prefix = u32::from_be_bytes(src[0..4].try_into().unwrap());

        if (length_prefix as usize) + 4 <= src.len() {
            if src.len() == 4 {
                println!("Received keepalive message.");
                src.advance(4 + length_prefix as usize);
                return Ok(None);
            }
            let message_type = src[4];
            let payload = src[5..(4 + length_prefix as usize)].to_vec();

            src.advance((4 + length_prefix as usize));
            return Ok(Some(BTMessage::new(message_type, payload)?));
        } else {
            // Complete message not yet received
            return Ok(None)
        }
    }
}
impl BTMessage {
    pub(crate) fn new(message_type: u8, payload: Vec<u8>) -> Result<Self> {
        let m : Option<BTMessage> = match message_type {
            0 => Some(BTMessage::Choke),
            1 => Some(BTMessage::Unchoke),
            2 => Some(BTMessage::Interested),
            3 => Some(BTMessage::NotInterested),
            4 => Some(BTMessage::Have(u32::from_be_bytes(payload[0..4].try_into()?))),
            5 => Some(BTMessage::Bitfield(payload.iter().map(|byte| format!("{:08b}", byte)).collect::<String>())),
            6 => Some(
                BTMessage::Request(
                    u32::from_be_bytes(payload[0..4].try_into()?),
                    u32::from_be_bytes(payload[4..8].try_into()?),
                    u32::from_be_bytes(payload[8..12].try_into()?)
                )
            ),
            7 => Some(
                BTMessage::Piece(
                    u32::from_be_bytes(payload[0..4].try_into()?),
                    u32::from_be_bytes(payload[4..8].try_into()?),
                    payload[8..].to_vec()
                )
            ),
            8 => Some(
                BTMessage::Cancel(
                    u32::from_be_bytes(payload[0..4].try_into()?),
                    u32::from_be_bytes(payload[4..8].try_into()?),
                    u32::from_be_bytes(payload[8..12].try_into()?)
                )
            ),
            _ => None
        };
        m.ok_or(anyhow!(format!("Unexpected message type: {}", message_type)))
    }
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut buf = BytesMut::new();

        match self {
            BTMessage::Choke => {
                buf.put_u32(1); // Message length
                buf.put_u8(0); // Message ID
            },
            BTMessage::Unchoke => {
                buf.put_u32(1);
                buf.put_u8(1);
            },
            BTMessage::Interested => {
                buf.put_u32(1);
                buf.put_u8(2);
            },
            BTMessage::NotInterested => {
                buf.put_u32(1);
                buf.put_u8(3);
            },
            BTMessage::Have(piece_index) => {
                buf.put_u32(5); // Message length: 1 byte ID + 4 bytes piece index
                buf.put_u8(4); // Message ID
                buf.put_u32(*piece_index); // Piece index
            },
            BTMessage::Bitfield(bitfield) => {
                let bitfield_bytes = hex::decode(bitfield)?;
                buf.put_u32(1 + bitfield_bytes.len() as u32); // Message length
                buf.put_u8(5); // Message ID
                buf.extend_from_slice(&bitfield_bytes); // Bitfield
            },
            BTMessage::Request(index, begin, length) => {
                buf.put_u32(13); // Message length: 1 byte ID + 3 * 4 bytes
                buf.put_u8(6); // Message ID
                buf.put_u32(*index); // Piece index
                buf.put_u32(*begin); // Block begin
                buf.put_u32(*length); // Block length
            },
            BTMessage::Piece(index, begin, block) => {
                buf.put_u32(9 + block.len() as u32); // Message length
                buf.put_u8(7); // Message ID
                buf.put_u32(*index); // Piece index
                buf.put_u32(*begin); // Block begin
                buf.extend_from_slice(block); // Block data
            },
            BTMessage::Cancel(index, begin, length) => {
                buf.put_u32(13); // Message length
                buf.put_u8(8); // Message ID
                buf.put_u32(*index); // Piece index
                buf.put_u32(*begin); // Block begin
                buf.put_u32(*length); // Block length
            },
        }

        Ok(buf.to_vec())
    }
}
#[derive(Debug, PartialEq, Default)]
pub struct Handshake {
    length: u8,
    protocol: [u8; 19],
    reserved: u64,
    info_hash: [u8; 20],
    peer_id: [u8; 20],
}
impl Handshake {
    pub fn new(peer_id: &[u8], info_hash: &[u8]) -> Self {
        // Ensure the protocol string is exactly 19 bytes.
        let protocol_str = "BitTorrent protocol";
        let protocol_bytes = protocol_str.as_bytes();
        let mut protocol_array = [0u8; 19]; // Initialize with zeros.
        protocol_array.copy_from_slice(protocol_bytes);

        let mut info_hash_array = [0u8; 20]; // Initialize with zeros.
        info_hash_array.copy_from_slice(info_hash);

        let mut peer_id_array = [0u8; 20]; // Initialize with zeros.
        peer_id_array.copy_from_slice(peer_id);

        Self {
            length: 19,
            protocol: protocol_array,
            reserved: 0,
            info_hash: info_hash_array,
            peer_id : peer_id_array,
        }
    }
    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = Vec::new();

        serialized.push(self.length);
        serialized.extend_from_slice(&self.protocol);
        let reserved_bytes = self.reserved.to_be_bytes();
        serialized.extend_from_slice(&reserved_bytes);
        serialized.extend_from_slice(&self.info_hash);
        serialized.extend_from_slice(&self.peer_id);

        serialized
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        let mut protocol: [u8; 19] = [0u8; 19];
        protocol.copy_from_slice(&bytes[1..20]);

        let mut info_hash: [u8; 20] = [0u8; 20];
        info_hash.copy_from_slice(&bytes[28..48]);

        let mut peer_id: [u8; 20] = [0u8; 20];
        peer_id.copy_from_slice(&bytes[48..68]);

        Self {
            length: bytes[0],
            protocol,
            reserved: 0,
            info_hash,
            peer_id,
        }
    }

    pub fn peer_id(&self) -> String {
        let peer_id_str  = self.peer_id
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();
        peer_id_str
    }
}

impl fmt::Display for Handshake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let protocol_str = match std::str::from_utf8(&self.protocol) {
            Ok(s) => s,
            Err(_) => return Err(fmt::Error)
        };

        let info_hash_str = self.info_hash
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();

        let peer_id_str  = self.peer_id
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();

        write!(f, "Handshake[length: {}, protocol: '{}', reserved: {}, info_hash: {}, peer_id: {}]",
            self.length, protocol_str, self.reserved, info_hash_str, peer_id_str)
    }
}