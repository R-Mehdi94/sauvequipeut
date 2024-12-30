use std::net::TcpStream;

pub fn connect_to_server(addr: &str, port: &str) -> Result<TcpStream, Box<dyn std::error::Error>> {
    let full_addr = format!("{}:{}", addr, port);
    for _ in 0..3 {
        match TcpStream::connect(&full_addr) {
            Ok(stream) => return Ok(stream),
            Err(e) => eprintln!("Erreur de connexion : {}. Nouvelle tentative...", e),
        }
        std::thread::sleep(std::time::Duration::from_secs(2)); //2 seconde
    }
    Err(
        "Impossible de se connecter au serveur après plusieurs tentatives"
            .to_string()
            .into(),
    )
}
