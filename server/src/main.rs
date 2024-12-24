use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("localhost:8778")?;

    let mut test = r#"{"RegisterTeam":{"name":"curious_broccoli"}}"#.to_string();

    let n = test.len() as u32;
    let bytes = n.to_le_bytes();

    stream.write(&bytes)?;
    stream.write(test.as_bytes())?;

    let mut size_buffer = [0_u8; 4];
    stream.read_exact(&mut size_buffer)?;

    let response_size = u32::from_le_bytes(size_buffer) as usize;
    let mut buffer = vec![0u8; response_size];
    stream.read_exact(&mut buffer)?;

    let response = String::from_utf8(buffer.to_vec()).unwrap();
    println!("Réponse du serveur : {}", response);

    Ok(())
}
