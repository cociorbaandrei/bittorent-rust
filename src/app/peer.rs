use anyhow::{Result, anyhow};
use std::net::TcpStream;
use std::io::{self, Write, Read};
use crate::app::messages::Handshake;
use crate::app::tracker::MetaData;

pub fn connect_to_peer(peer: (&str, u16), handshake: Handshake) -> Result<()>{
    let (ip, port) = peer;
    let address = format!("{}:{}", ip, port);
    let mut stream = TcpStream::connect(&address)
        .map_err(|e| anyhow!("Failed to connect to peer {}: {}", address, e))?;
    let bytes = &handshake.serialize();
    stream.write_all(&bytes)
        .map_err(|e| anyhow!("Failed to write handshake to peer {}: {}", address, e))?;

    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(bytes_read) => {
           //println!("Received {} bytes: {:?}", bytes_read, &buffer[..bytes_read]);
            let peer_handshake = Handshake::deserialize(&buffer[..bytes_read]);
            //println!("Received peer handshake: {:#?}", peer_handshake);
            //println!("Received peer handshake: {}", peer_handshake);
            println!("Peer ID: {}", peer_handshake.peer_id())
        }
        Err(_) => {}
    }

    Ok(())
}