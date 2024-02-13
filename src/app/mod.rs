mod bencode;
mod tracker;
mod network;

use anyhow::{Result};
use std::fs;
use crate::app::tracker::{MetaData};
use crate::app::network::*;
use sha1::{Sha1, Digest};

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
    let data = bencode::decode(&_content)?;

    let torrent_info = MetaData::new(data)?;
    //println!("{:#?}", torrent_info);
    println!("Tracker URL: {}", torrent_info.announce);
    println!("Length: {}", torrent_info.info.length);
    println!("Hashed data: {}", torrent_info.raw().info_hash()?);
    println!("Piece Length: {}", torrent_info.info.piece_length);
    println!("Piece Hashes:\n{}", torrent_info.info.hashes().join("\n"));
    let peers = discover_peers(&torrent_info)?;
    for (ip, port) in peers {
        println!("{}:{}", ip, port);
    }
    Ok(())
}

pub fn entrypoint(args: Vec<String>) -> Result<()> {
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
            for (ip, port) in peers {
                println!("{}:{}", ip, port);
            }
        } else {
            println!("unknown command: {}", args[1])
        }
    }
    Ok(())
}
