
use std::fmt;
use std::hash::Hash;
use log::info;

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

        // Step 1: Add the length prefix
        serialized.push(self.length);

        // Step 2: Add the protocol identifier
        serialized.extend_from_slice(&self.protocol);

        // Step 3: Add the reserved bytes
        let reserved_bytes = self.reserved.to_be_bytes(); // Big-endian representation
        serialized.extend_from_slice(&reserved_bytes);

        // Step 4: Add the info hash
        serialized.extend_from_slice(&self.info_hash);

        // Step 5: Add the peer ID
        serialized.extend_from_slice(&self.peer_id);

        serialized
    }

    pub fn deserialize(bytes: &[u8]) -> Self {
        let mut protocol: [u8; 19] = [0u8; 19];;
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