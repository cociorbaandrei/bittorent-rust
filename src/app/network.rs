use std::fmt::format;
use anyhow::{Result};
use crate::app::tracker::MetaData;
use reqwest::blocking::get;
use reqwest::Error;
use url::{Url, ParseError};
use hex;
use crate::app::bencode;
// Define characters that do NOT require encoding
fn urlencode(data: &[u8]) -> String {
    let lookup = b"0123456789abcdef";
    let mut encoded = String::new();
    for &byte in data {
        match byte {
            b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'-' | b'_' | b'.' | b'~' => encoded.push(byte as char),
            _ => {
                encoded.push('%');
                encoded.push(lookup[(byte >> 4) as usize] as char);
                encoded.push(lookup[(byte & 0x0F) as usize] as char);
            },
        }
    }
    encoded
}

pub(crate) fn discover_peers(torrent: &MetaData) -> Result<Vec<(String, u32)>> {
    let announce = &torrent.announce;
    let mut url = Url::parse(announce)?;
    let encoded_hash = urlencode(&torrent.raw().info_hash_u8()?).to_string();
    println!("{:#?}", torrent.raw().info_hash_u8()?);
    println!("{:#?}", encoded_hash);
    let peer_id = "00112233445566778899";
    let port = "6881";
    let uploaded = "0";
    let downloaded = "0";
    let left = torrent.info.length.to_string(); // Assuming this is how you get the length
    let compact = "1";

    let query = format!(
        "info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}",
        encoded_hash, peer_id, port, uploaded, downloaded, left, compact
    );

    url.set_query(Some(&query));

    println!("{:#?}", url);
    let res = get(url)?.text()?;
    println!("{:#?}", res);
    println!("Started decoding response.");
    let decoded = bencode::decode(res.as_bytes());
    println!("{:#?}", decoded);
    Ok(Default::default())
}