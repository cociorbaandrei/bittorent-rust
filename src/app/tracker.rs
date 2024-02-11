use crate::app::bencode;
use crate::app::bencode::Value;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

#[allow(unused_imports)]
#[derive(Debug, Clone)]
pub struct MetaData {
    pub announce: String,
    pub info: Info,
}

impl MetaData {
    pub(crate) fn new(values: bencode::Value) -> Result<Self> {
        match values {
            bencode::Dict(map) => {
                let announce = map.get("announce")
                    .and_then(|announce| match announce {
                        Value::Str(url) => Some(std::str::from_utf8(url)),
                        _ => None
                    })
                    .ok_or(anyhow!("Missing or invalid field 'announce'"))?
                    .map_err(|_| anyhow!("Invalid utf-8 bytes while parsing 'name'."))?
                    .to_owned();

                let info = map.get("info")
                    .and_then(|info| match info {
                        Value::Dict(info_dict) => Some(Info::new(&info_dict)),
                        _ => None
                    })
                    .ok_or(anyhow!("Missing or invalid 'info'"))??;

                Some(Self { announce, info })
            }
            _ => None,
        }.ok_or( anyhow!("Expected object to be a dictionary."))
    }
}

#[derive(Debug, Clone)]
pub struct Info {
    pub length: i64,
    pub name: String,
    pub piece_length: i64,
    pub pieces: Vec<u8>,
}

impl Info {
    pub fn new(values: &HashMap<String, bencode::Value>) -> Result<Self>  {
        let name = values.get("name")
            .and_then(|name| match name {
                Value::Str(bytes) => Some(std::str::from_utf8(bytes)),
                _ => None
            })
            .ok_or(anyhow!("Missing or invalid 'name'"))?
            .map_err(|_| anyhow!("Invalid utf-8 bytes while parsing 'name'."))?
            .to_owned();

        let piece_length = values.get("piece length")
            .and_then(|piece_length| match piece_length {
                Value::Int(piece_length) => Some(*piece_length),
                _ => None
            })
            .ok_or(anyhow!("Expected that 'piece length' is an integer."))?;

        let length = values.get("length")
            .and_then(|length| match length {
                Value::Int(length) => Some(*length),
                _ => None
            })
            .ok_or(anyhow!("Expected that 'length' is an integer."))?;

        let pieces = values.get("pieces")
            .and_then(|pieces| match pieces {
                Value::Str(pieces) => Some(pieces.clone()),
                _ => None
            })
            .ok_or(anyhow!("Expected that 'length' is an integer."))?;

        Ok(Self { length,name, piece_length, pieces })
    }
}