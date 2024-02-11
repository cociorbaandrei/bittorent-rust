pub mod bencode;
use std::fs;
use std::io;

fn read_binary_file(path: &str) -> io::Result<Vec<u8>> {
    let data = fs::read(path)?;
    Ok(data)
}

fn decode_bencoded_value(value: &str) -> Result<String, &'static str> {
    let buffer = value.as_bytes();
    let decoded = bencode::decode(buffer).unwrap();
    return bencode::to_string(&decoded);
}

fn no_args() -> io::Result<()> {
    let path = "itsworking.gif.torrent";
    let _content = read_binary_file(path)?;

    let test = "li24ed3:keyli3123e3:heli23e3:assi1337eeei23ed3:assi23eee";
    let decoded_test = bencode::decode(test.as_bytes()).unwrap();
    //     //bencode::print_bvalue(&decoded_test);

    //     let decoded_torrent = bencode::decode(&_content).unwrap();
    //    // bencode::print_bvalue(&decoded_torrent);
    println!("{}", bencode::to_string(&decoded_test).unwrap());
    Ok(())
}

pub fn entrypoint(args: Vec<String>) -> io::Result<()> {
    if args.len() < 2 {
        let _ = no_args()?;
    } else {
        let command = &args[1];

        if command == "decode" {
            let encoded_value = &args[2];
            let decoded_value = decode_bencoded_value(encoded_value).unwrap();
            println!("{}", decoded_value);
        } else {
            println!("unknown command: {}", args[1])
        }
    }
    Ok(())
}
