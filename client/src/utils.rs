use common::utils::my_error::MyError;
use std::{net::TcpStream, thread, time};

pub fn connect_to_server(addr: &str, port: &str) -> Result<TcpStream, MyError> {
    let full_addr = format!("{}:{}", addr, port);
    loop {
        match TcpStream::connect(&full_addr) {
            Ok(stream) => return Ok(stream),
            Err(e) => eprintln!("Erreur de connexion : {}. Nouvelle tentative...", e),
        }
        thread::sleep(time::Duration::from_secs(2)); //2 secondes
    }
}
