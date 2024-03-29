mod bencode;
mod messages;
mod network;
mod peer;
mod tracker;
use anyhow::Result;
use futures::stream::StreamExt;

use std::fs;

use crate::app::messages::{BTMessage, BTMessageFramer, Handshake};
use crate::app::network::*;
use crate::app::peer::PeerManager;
use crate::app::tracker::MetaData;
use futures::SinkExt;

use tokio::net::TcpStream;
use tokio_util::codec::Framed;

static mut DOWNLOADED: u64 = 0;
fn read_binary_file(path: &str) -> Result<Vec<u8>> {
    let data = fs::read(path)?;
    Ok(data)
}

fn decode_bencoded_value(value: &str) -> Result<String> {
    let buffer = value.as_bytes();
    let decoded = bencode::decode(buffer)?;
    bencode::to_string(&decoded)
}
pub async fn download_piece(
    index: usize,
    torrent_info: &MetaData,
    peer: &mut Framed<TcpStream, BTMessageFramer>,
    _file_name: &str,
) -> Result<()> {
    let block_size = 16 * 1024; // 16 KiB
    let mut total_pieces = torrent_info.info.length / torrent_info.info.piece_length;
    if torrent_info.info.length % torrent_info.info.piece_length > 0 {
        total_pieces += 1;
    }
    let mut last_piece_size = torrent_info.info.length % torrent_info.info.piece_length;
    if last_piece_size == 0 {
        // If the total size is a perfect multiple of the piece size
        last_piece_size = torrent_info.info.piece_length; // The last piece is a full piece
    }
    let mut number_of_blocks_in_last_piece = last_piece_size / block_size;
    if last_piece_size % block_size != 0 {
        // If there's a remainder
        number_of_blocks_in_last_piece += 1; // There's an additional, partially-filled block
    }
    let mut size_of_last_block_in_last_piece = last_piece_size % block_size;
    if size_of_last_block_in_last_piece == 0 && last_piece_size != 0 {
        size_of_last_block_in_last_piece = block_size; // The last block is a full block if no remainder
    }
    let i = index;
    let piece_length = torrent_info.info.piece_length as u32;
    const BLOCK_SIZE: u32 = 16 * 1024; // 16 KiB in bytes

    let mut total_blocks = (piece_length as f32 / BLOCK_SIZE as f32).ceil() as u32;
    if i == total_pieces as usize - 1usize {
        total_blocks = number_of_blocks_in_last_piece as u32;
    }

    for block_index in 0..total_blocks {
        let begin = block_index * BLOCK_SIZE;
        let length = if block_index == total_blocks - 1 && i == total_pieces as usize - 1 {
            // Last block, calculate remaining bytes
            size_of_last_block_in_last_piece
        } else {
            // All blocks except the last one are of BLOCK_SIZE
            BLOCK_SIZE as i64
        };

        let r = BTMessage::Request(i as u32, begin, length as u32);
        peer.send(r).await?;
        unsafe {
            DOWNLOADED += 1;
        }
    }
    Ok(())
}

pub async fn download_pieces(
    _index: usize,
    torrent_info: &MetaData,
    peer: &mut Framed<TcpStream, BTMessageFramer>,
    _file_name: &str,
) -> Result<()> {
    let block_size = 16 * 1024; // 16 KiB
    let mut total_pieces = torrent_info.info.length / torrent_info.info.piece_length;
    if torrent_info.info.length % torrent_info.info.piece_length > 0 {
        total_pieces += 1;
    }
    let mut last_piece_size = torrent_info.info.length % torrent_info.info.piece_length;
    if last_piece_size == 0 {
        // If the total size is a perfect multiple of the piece size
        last_piece_size = torrent_info.info.piece_length; // The last piece is a full piece
    }
    let mut number_of_blocks_in_last_piece = last_piece_size / block_size;
    if last_piece_size % block_size != 0 {
        // If there's a remainder
        number_of_blocks_in_last_piece += 1; // There's an additional, partially-filled block
    }
    let mut size_of_last_block_in_last_piece = last_piece_size % block_size;
    if size_of_last_block_in_last_piece == 0 && last_piece_size != 0 {
        size_of_last_block_in_last_piece = block_size; // The last block is a full block if no remainder
    }
    for i in 0..total_pieces {
        let piece_length = torrent_info.info.piece_length as u32;
        const BLOCK_SIZE: u32 = 16 * 1024; // 16 KiB in bytes

        let mut total_blocks = (piece_length as f32 / BLOCK_SIZE as f32).ceil() as u32;
        if i as usize == total_pieces as usize - 1usize {
            total_blocks = number_of_blocks_in_last_piece as u32;
        }

        for block_index in 0..total_blocks {
            let begin = block_index * BLOCK_SIZE;
            let length =
                if block_index == total_blocks - 1 && i as usize == total_pieces as usize - 1 {
                    // Last block, calculate remaining bytes
                    size_of_last_block_in_last_piece
                } else {
                    // All blocks except the last one are of BLOCK_SIZE
                    BLOCK_SIZE as i64
                };

            let r = BTMessage::Request(i as u32, begin, length as u32);
            peer.send(r).await?;
            unsafe {
                DOWNLOADED += 1;
            }
        }
    }
    Ok(())
}

async fn no_args() -> Result<()> {
    let path = "sample.torrent";
    let _content = read_binary_file(path)?;
    let torrent_info = MetaData::new(bencode::decode(&_content)?)?;

    let mut peer_manager = PeerManager::new(torrent_info.clone()).await?;
    let stream = peer_manager.connect_to_peer().await?;

    let mut peer = tokio_util::codec::Framed::new(stream, BTMessageFramer);

    while let Some(msg) = peer.next().await {
        match msg? {
            BTMessage::Choke => {}
            BTMessage::Unchoke => {
                download_pieces(0, &torrent_info, &mut peer, "sample.txt").await?;
            }
            BTMessage::Interested => {}
            BTMessage::NotInterested => {}
            BTMessage::Have(_) => {}
            BTMessage::Bitfield(_) => {
                let intr = BTMessage::Interested;
                peer.send(intr).await?;
            }
            BTMessage::Request(_, _, _) => {}
            BTMessage::Piece(idx, offset, data) => {
                //println!("Piece: {} {} {:?} ", idx, offset, data.len());
                peer::write_at_offset(
                    &torrent_info.info.name,
                    (idx * torrent_info.info.piece_length as u32 + offset) as u64,
                    &data,
                )
                .await?;
                break;
            }
            BTMessage::Cancel(_, _, _) => {}
        }
    }

    Ok(())
}
// can_parse_message now also removes the processed message from the buffer

pub(crate) async fn entrypoint(args: Vec<String>) -> Result<()> {
    if args.len() < 2 {
        no_args().await?;
        println!("{}", &args[0]);
    } else {
        let command = &args[1]; // &args[1];
        if command == "decode" {
            let encoded_value = &args[2];
            let decoded_value = decode_bencoded_value(encoded_value)?;
            println!("{}", decoded_value);
        } else if command == "info" {
            let path = &args[2];
            let _content = read_binary_file(path)?;
            let data = bencode::decode(&_content)?;
            let torrent_info = MetaData::new(data.clone())?;
            println!("Tracker URL: {}", torrent_info.announce);
            println!("Length: {}", torrent_info.info.length);
            println!("Info Hash: {}", torrent_info.raw().info_hash()?);
            println!("Piece Length: {}", torrent_info.info.piece_length);
            println!("Piece Hashes:\n{}", torrent_info.info.hashes().join("\n"));
        } else if command == "peers" {
            let path = &args[2];
            let _content = read_binary_file(path)?;
            let data = bencode::decode(&_content)?;
            let torrent_info = MetaData::new(data.clone())?;
            let peers = discover_peers(&torrent_info).await?;
            for (ip, port) in peers.iter() {
                println!("{}:{}", ip, port);
            }
        } else if command == "handshake" {
            let _peer = &args[3];
            println!("peer: {}", _peer);
            let _content = read_binary_file(&args[2])?;
            let torrent_info = MetaData::new(bencode::decode(&_content)?)?;
            let _peers = discover_peers(&torrent_info).await?;
            let _handshake =
                Handshake::new(b"00112233445566778899", &torrent_info.raw().info_hash_u8()?);
            let mut peer_manager = PeerManager::new(torrent_info.clone()).await?;
            // let (peer_ip, peer_port) = peers.iter().next().ok_or(anyhow!("Failed to get first peer"))?;
            let mut p = _peer.split(':');
            let _peer_ip = p.next().unwrap();
            let _peer_port = p.next().unwrap().parse::<u16>()?;
            let _stream = peer_manager.connect_to_peer().await?;
        } else if command == "download_piece" {
            println!("no args {} {:#?}", args.len(), args);
            println!("file_name: {}, _content {}", &args[3], &args[4]);
            let file_name = &args[3];
            let _content = read_binary_file(&args[4])?;
            let _piece_number = &args[5].parse::<usize>()?;
            let torrent_info = MetaData::new(bencode::decode(&_content)?)?;
            let _peers = discover_peers(&torrent_info).await?;
            let _handshake =
                Handshake::new(b"00112233445566778899", &torrent_info.raw().info_hash_u8()?);
            let mut peer_manager = PeerManager::new(torrent_info.clone()).await?;
            let stream = peer_manager.connect_to_peer().await?;

            let mut peer = tokio_util::codec::Framed::new(stream, BTMessageFramer);

            while let Some(msg) = peer.next().await {
                //println!("{:#?}", msg);
                match msg? {
                    BTMessage::Choke => {}
                    BTMessage::Unchoke => {
                        download_piece(*_piece_number, &torrent_info, &mut peer, file_name).await?;
                    }
                    BTMessage::Interested => {}
                    BTMessage::NotInterested => {}
                    BTMessage::Have(_) => {}
                    BTMessage::Bitfield(_) => {
                        let intr = BTMessage::Interested;
                        peer.send(intr).await?;
                    }
                    BTMessage::Request(_, _, _) => {}
                    BTMessage::Piece(_idx, offset, data) => {
                        peer::write_at_offset(
                            file_name,
                            (offset) as u64,
                            &data,
                        )
                        .await?;
                        unsafe {
                            DOWNLOADED -= 1;
                            if DOWNLOADED == 0 {
                                break;
                            }
                        }
                    }
                    BTMessage::Cancel(_, _, _) => {}
                }
            }
        } else if command == "download" {
            println!("no args {} {:#?}", args.len(), args);
            println!("file_name: {}, _content {}", &args[3], &args[4]);
            let file_name = &args[3];
            let _content = read_binary_file(&args[4])?;
            let _piece_number = 0;
            let torrent_info = MetaData::new(bencode::decode(&_content)?)?;
            let _peers = discover_peers(&torrent_info).await?;
            let _handshake =
                Handshake::new(b"00112233445566778899", &torrent_info.raw().info_hash_u8()?);
            let mut peer_manager = PeerManager::new(torrent_info.clone()).await?;
            let stream = peer_manager.connect_to_peer().await?;

            let mut peer = tokio_util::codec::Framed::new(stream, BTMessageFramer);

            while let Some(msg) = peer.next().await {
                //println!("{:#?}", msg);
                match msg? {
                    BTMessage::Choke => {}
                    BTMessage::Unchoke => {
                        download_pieces(_piece_number, &torrent_info, &mut peer, file_name).await?;
                    }
                    BTMessage::Interested => {}
                    BTMessage::NotInterested => {}
                    BTMessage::Have(_) => {}
                    BTMessage::Bitfield(_) => {
                        let intr = BTMessage::Interested;
                        peer.send(intr).await?;
                    }
                    BTMessage::Request(_, _, _) => {}
                    BTMessage::Piece(idx, offset, data) => {
                        peer::write_at_offset(
                            file_name,
                            (idx * torrent_info.info.piece_length as u32 + offset) as u64,
                            &data,
                        )
                        .await?;
                        unsafe {
                            DOWNLOADED -= 1;
                            if DOWNLOADED == 0 {
                                break;
                            }
                        }
                    }
                    BTMessage::Cancel(_, _, _) => {}
                }
            }
        } else {
            println!("unknown command: {}", args[1])
        }
    }
    Ok(())
}
