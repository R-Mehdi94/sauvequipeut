use crate::message::{Message, MessageData, RegisterTeam, SubscribePlayer, SubscribePlayerResult};
use crate::state::ClientState;
use std::io::{Read, Write};
use std::net::TcpStream;

pub fn build_message(data: MessageData) -> Result<Message, Box<dyn std::error::Error>> {
    match data {
        MessageData::RegisterTeam { name } => Ok(Message::RegisterTeam(RegisterTeam { name })),
        MessageData::SubscribePlayer {
            name,
            registration_token,
        } => Ok(Message::SubscribePlayer(SubscribePlayer {
            name,
            registration_token,
        })),
        MessageData::RadarView(encoded_data) => Ok(Message::RadarView(encoded_data)),
        MessageData::Hint(hint) => Ok(Message::Hint(hint)),
        MessageData::Action(action) => Ok(Message::Action(action)),
        MessageData::Challenge(challenge) => Ok(Message::Challenge(challenge)),
    }
}

pub fn send_message(
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

pub fn receive_response(stream: &mut TcpStream) -> Result<Message, Box<dyn std::error::Error>> {
    let mut size_buffer = [0_u8; 4];
    stream.read_exact(&mut size_buffer)?;
    let response_size = u32::from_le_bytes(size_buffer) as usize;

    let mut response_buffer = vec![0u8; response_size];
    stream.read_exact(&mut response_buffer)?;

    let response: Message = serde_json::from_slice(&response_buffer)?;
    Ok(response)
}

pub fn process_message(
    message: Message,
    state: &mut ClientState,
) -> Result<(), Box<dyn std::error::Error>> {
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
                return Err("Réponse inattendue dans RegisterTeamResult".into());
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
        _ => println!("Message inattendu !"),
    }

    Ok(())
}
