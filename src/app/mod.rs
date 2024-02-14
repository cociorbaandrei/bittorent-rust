mod bencode;
mod tracker;
mod network;
mod peer;
mod messages;

use anyhow::{Result, anyhow};
use std::fs;
use tokio::net::TcpStream;
use crate::app::messages::Handshake;
use crate::app::tracker::{MetaData};
use crate::app::network::*;
use crate::app::peer::connect_to_peer;
use tokio::io::AsyncWriteExt;
fn read_binary_file(path: &str) -> Result<Vec<u8>> {
    let data = fs::read(path)?;
    Ok(data)
}

fn decode_bencoded_value(value: &str) -> Result<String> {
    let buffer = value.as_bytes();
    let decoded = bencode::decode(buffer)?;
    return bencode::to_string(&decoded);
}

fn no_args() -> Result<()> {
    let path = "congratulations.gif.torrent";
    let _content = read_binary_file(path)?;
    let torrent_info = MetaData::new(bencode::decode(&_content)?)?;
    let peers = discover_peers(&torrent_info)?;
    let handshake = Handshake::new(b"00112233445566778899", &torrent_info.raw().info_hash_u8()?);
    let (peer_ip, peer_port) = peers.iter().next().ok_or(anyhow!("Failed to get first peer"))?;
    connect_to_peer((peer_ip, *peer_port), handshake)?;
    Ok(())


}

pub  fn entrypoint(args: Vec<String>) -> Result<()> {
    if args.len() < 2 {
        let _ = no_args()?;
    } else {
        let command = &args[1];

        if command == "decode" {
            let encoded_value = &args[2];
            let decoded_value = decode_bencoded_value(encoded_value)?;
            println!("{}", decoded_value);
        } else if command == "info" {
            let path = &args[2];
            let _content = read_binary_file(path)?;
            let data =  bencode::decode(&_content)?;
            let torrent_info = MetaData::new(data.clone())?;
            println!("Tracker URL: {}", torrent_info.announce);
            println!("Length: {}", torrent_info.info.length);
            println!("Info Hash: {}", torrent_info.raw().info_hash()?);
            println!("Piece Length: {}", torrent_info.info.piece_length);
            println!("Piece Hashes:\n{}", torrent_info.info.hashes().join("\n"));
        } else if command == "peers" {
            let path = &args[2];
            let _content = read_binary_file(path)?;
            let data =  bencode::decode(&_content)?;
            let torrent_info = MetaData::new(data.clone())?;
            let peers = discover_peers(&torrent_info)?;
            for (ip, port) in peers.iter() {
                println!("{}:{}", ip, port);
            }
        } else if command == "handshake" {
            let peer = &args[3];
            let _content = read_binary_file(&args[2])?;
            let torrent_info = MetaData::new(bencode::decode(&_content)?)?;
            let peers = discover_peers(&torrent_info)?;
            let handshake = Handshake::new(b"00112233445566778899", &torrent_info.raw().info_hash_u8()?);
            let (peer_ip, peer_port) = peers.iter().next().ok_or(anyhow!("Failed to get first peer"))?;
            connect_to_peer((peer_ip, *peer_port), handshake)?;
        } else {
            println!("unknown command: {}", args[1])
        }
    }
    Ok(())
}
