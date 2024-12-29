mod json_message;

use crate::json_message::{Message, RegisterTeam};
use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: client [server_address]");
        return Ok(());
    }

    let addr = &args[1];*/

    let addr = "localhost";
    let port = "8778";

    let mut stream = match connect_to_server(addr, port) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "Erreur critique : impossible de se connecter au serveur : {}",
                e
            );
            return Err(e);
        }
    };

    loop {
        let message = build_message();
        send_message(&mut stream, &message)?;

        let response = receive_response(&mut stream)?;
        let message = process_message(response)?;

        println!("{:#?}", message);
    }
}

fn connect_to_server(addr: &str, port: &str) -> Result<TcpStream, Box<dyn std::error::Error>> {
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

fn build_message() -> Message {
    Message::RegisterTeam(RegisterTeam {
        name: "test team".to_string(),
    })
}

fn send_message(
    stream: &mut TcpStream,
    message: &Message,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_message = serde_json::to_string(message).expect("Failed to serialize message");
    println!("Envoi du message : {}", json_message);

    let size = json_message.len() as u32;
    stream.write(&size.to_le_bytes())?;

    stream.write(json_message.as_bytes())?;
    Ok(())
}

fn receive_response(stream: &mut TcpStream) -> Result<Message, Box<dyn std::error::Error>> {
    let mut size_buffer = [0_u8; 4];
    stream.read_exact(&mut size_buffer)?;
    let response_size = u32::from_le_bytes(size_buffer) as usize;

    let mut response_buffer = vec![0u8; response_size];
    stream.read_exact(&mut response_buffer)?;

    let response: Message = serde_json::from_slice(&response_buffer)?;
    Ok(response)
}

fn process_message(mut message: Message) -> Result<(), Box<dyn std::error::Error>> {
    match message {
        Message::RegisterTeamResult(result) => {
            if let Some(success) = result.Ok {
                println!(
                    "Enregistrement réussi ! Joueurs attendus : {}, Token : {}",
                    success.expected_players, success.registration_token
                );
            } else if let Some(error) = result.Err {
                return Err(format!("Erreur lors de l'enregistrement : {}", error).into());
            } else {
                return Err("Réponse inattendue dans RegisterTeamResult".into());
            }
        }
        _ => {
            return Err("Réponse inattendue !".into());
        }
    }

    Ok(())
}
