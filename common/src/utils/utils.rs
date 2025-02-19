use crate::message::Message::RadarViewResult;
use crate::message::{Message, MessageData, RegisterTeam, SubscribePlayer, SubscribePlayerResult};
use crate::state::ClientState;
use crate::utils::my_error::MyError;
use serde::de::Unexpected::Option;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;

pub fn build_message(data: MessageData) -> Result<Message, MyError> {
    match data {
        MessageData::RegisterTeam { name } => Ok(Message::RegisterTeam(RegisterTeam { name })),
        MessageData::SubscribePlayer {
            name,
            registration_token,
        } => Ok(Message::SubscribePlayer(SubscribePlayer {
            name,
            registration_token,
        })),
        MessageData::RadarViewResult(encoded_data) => Ok(Message::RadarViewResult(encoded_data)),
        MessageData::Hint(hint) => Ok(Message::Hint(hint)),
        MessageData::Action(action) => Ok(Message::Action(action)),
        MessageData::Challenge(challenge) => Ok(Message::Challenge(challenge)),
    }
}

pub fn send_message(stream: &mut TcpStream, message: &Message) -> Result<(), MyError> {
    let json_message = serde_json::to_string(message)?;

    let size = json_message.len() as u32;
    stream.write_all(&size.to_le_bytes())?;

    stream.write_all(json_message.as_bytes())?;

    Ok(())
}

pub fn receive_response(stream: &mut TcpStream) -> Result<Message, MyError> {
    let mut size_buffer = [0_u8; 4];
    stream.read_exact(&mut size_buffer)?;
    let response_size = u32::from_le_bytes(size_buffer) as usize;
    let mut response_buffer = vec![0u8; response_size];
    stream.read_exact(&mut response_buffer)?;
    let mut raw_message: serde_json::Value = serde_json::from_slice(&response_buffer)?;
    if let Some(radar_view) = raw_message.get("RadarView") {
        if radar_view.is_string() {
            // Si "RadarView" est une chaîne, crée le message approprié
            let response = Message::RadarViewResult(radar_view.as_str().unwrap().to_string());
            return Ok(response);
        }
    }

    let response: Message = serde_json::from_slice(&response_buffer)?;
    Ok(response)
}

pub fn process_message(message: Message, state: &mut ClientState) -> Result<(), MyError> {
    match message {
        Message::RegisterTeamResult(result) => {
            if let Some(success) = result.Ok {
                println!(
                    "Enregistrement réussi Joueurs : {}, Token : {}",
                    success.expected_players, success.registration_token
                );
                state.team_info = Some(success);
            } else if let Some(error) = result.Err {
                println!("Erreur lors de l'enregistrement : {}", error);
                return Err(format!("Erreur lors de l'enregistrement : {}", error).into());
            } else {
                return Err("Réponse inattendue dans RegisterTeamResult"
                    .to_string()
                    .into());
            }
        }
        Message::SubscribePlayerResult(result) => match result {
            SubscribePlayerResult::Ok => {
                println!("Souscription réussie !");
            }
            SubscribePlayerResult::Err(error) => {
                println!("Erreur lors de la souscription : {}", error);
                return Err(format!("Erreur lors de la souscription : {}", error).into());
            }
        },
        Message::RadarViewResult(result) => {
            state.radar_view = Some(result);
        }
        _ => println!("Message inattendu !"),
    }

    Ok(())
}

pub fn handle_response(stream: &mut TcpStream, state: &mut ClientState) -> Result<(), MyError> {
    let response = receive_response(stream)?;
    process_message(response, state)?;

    Ok(())
}

pub fn is_connection_closed_with_peek(stream: &mut TcpStream) -> bool {
    let mut buffer = [0; 1];
    match stream.peek(&mut buffer) {
        Ok(0) => {
            // Si 0 octets sont disponibles, cela signifie que la connexion est fermée.
            println!("La connexion est fermée (EOF).");
            true
        }
        Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
            // La connexion est non-bloquante et il n'y a pas encore de données.
            println!("La connexion est toujours ouverte.");
            false
        }
        Err(ref e) if e.kind() == ErrorKind::ConnectionReset => {
            println!("La connexion a été réinitialisée.");
            true
        }
        Err(e) => {
            println!("Erreur inattendue lors du peek : {:?}", e);
            true
        }
        Ok(_) => {
            // Des données sont disponibles, donc la connexion est toujours ouverte.
            false
        }
    }
}
