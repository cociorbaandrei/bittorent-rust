pub mod bencode;
mod tracker;
use anyhow::{Result};
use std::fs;
use crate::app::tracker::MetaData;
use sha1::{Sha1, Digest};
use sha1::digest::Update;

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
    let path = "sample.torrent";
    let _content = read_binary_file(path)?;
    let _test = "di24ed3:keyli3123e3:heli23e3:assi1337eeei23ed3:assi23eee".as_bytes();
    let data = bencode::decode(&_content)?;

    let torrent_info = MetaData::new(data.clone())?;
    println!("{:#?}", torrent_info);
    println!("Tracker URL: {}", torrent_info.announce);
    println!("Length: {}", torrent_info.info.length);
    let mut hasher = Sha1::new();
    if let bencode::Dict(dict) = &data {
        let info = bencode::to_vec_u8(&dict["info"])?;
       // hasher.update(info);
        Digest::update(&mut hasher, info);
        let hashed_data  = hasher.finalize().to_vec();

        println!("Hashed data: {}", hashed_data.iter().map(|byte| format!("{:02x}", byte)).collect::<String>());
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
            let mut hasher = Sha1::new();
            if let bencode::Dict(dict) = &data {
                let info = bencode::to_vec_u8(&dict["info"])?;
                // hasher.update(info);
                Digest::update(&mut hasher, info);
                let hashed_data  = hasher.finalize().to_vec();

                println!("Hashed data: {}", hashed_data.iter().map(|byte| format!("{:02x}", byte)).collect::<String>());
            }
        } else {
            println!("unknown command: {}", args[1])
        }
    }
    Ok(())
}
