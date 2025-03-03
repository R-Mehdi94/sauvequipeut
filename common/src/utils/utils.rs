use crate::message::{
    Message, MessageData, RegisterTeam, SubscribePlayer, SubscribePlayerResult,
};
use crate::state::ClientState;
use crate::utils::my_error::MyError;
use std::io::{Read, Write};
use std::net::TcpStream;
use crate::message::challengedata::ChallengeData;
use crate::message::hintdata::HintData;

/// Construit un message `Message` à partir de `MessageData`.
///
/// Cette fonction convertit les différentes variantes de `MessageData` en une instance de `Message`.
///
/// # Paramètres
/// - `data`: Les données du message.
///
/// # Retourne
/// - `Ok(Message)`: Le message construit.
/// - `Err(MyError)`: Une erreur en cas d'échec.
///
/// # Exemple
/// ```
///
/// use common::message::MessageData;
/// use common::utils::utils::build_message;
/// let message_data = MessageData::RegisterTeam { name: "Team A".to_string() };
/// let message = build_message(message_data).unwrap();
/// ```

pub struct Player {
    pub name :String, pub register_token: String}

pub fn build_message(data: MessageData) -> Result<Message, MyError> {
    match data {
        MessageData::RegisterTeam { name } => Ok(Message::RegisterTeam(RegisterTeam { name})),
        MessageData::SubscribePlayer {
            name,
            registration_token,
        } => Ok(Message::SubscribePlayer(SubscribePlayer {
            name,
            registration_token,
        })),
        MessageData::Hint(hint) => Ok(Message::Hint(hint)),
        MessageData::Action(action) => Ok(Message::Action(action)),
        MessageData::Challenge(challenge) => Ok(Message::Challenge(challenge)),
        MessageData::RadarView(radar) => Ok(Message::RadarView(radar)),
        MessageData::SubscribePlayerResult(result) => Ok(Message::SubscribePlayerResult(result)),

    }
}

/// Envoie un message au serveur via un `TcpStream`.
///
/// # Paramètres
/// - `stream`: Une référence mutable vers le `TcpStream`.
/// - `message`: Une référence vers le message à envoyer.
///
/// # Retourne
/// - `Ok(())` si l'envoi est réussi.
/// - `Err(MyError)` en cas d'échec.
///
/// # Exemple
/// ```no_run
/// use std::net::TcpStream;
/// use common::message::{Message, MessageData, RegisterTeam};
/// use common::utils::utils::{build_message, send_message};
///
/// let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
/// let message = Message::RegisterTeam(RegisterTeam {name: "test".to_string() });
/// send_message(&mut stream, &message).unwrap();
/// ```
pub fn send_message(stream: &mut TcpStream, message: &Message) -> Result<(), MyError> {
    let json_message = serde_json::to_string(message)?;
    println!("JSON ENVOYÉ AU SERVEUR : {}", json_message);

    let size = json_message.len() as u32;
    stream.write_all(&size.to_le_bytes())?;
    stream.write_all(json_message.as_bytes())?;

    Ok(())
}

/// Reçoit une réponse du serveur et la convertit en `Message`.
///
/// # Paramètres
/// - `stream`: Une référence mutable vers le `TcpStream`.
///
/// # Retourne
/// - `Ok(Message)`: Le message reçu et interprété.
/// - `Err(MyError)`: Une erreur en cas d'échec.
///
/// # Exemple
/// ```no_run
/// use std::net::TcpStream;
/// use common::utils::utils::receive_response;
///
/// let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
/// let response = receive_response(&mut stream).unwrap();
/// ```
pub fn receive_response(stream: &mut TcpStream) -> Result<Message, MyError> {
    let mut size_buffer = [0_u8; 4];
    stream.read_exact(&mut size_buffer)?;
    let response_size = u32::from_le_bytes(size_buffer) as usize;
    let mut response_buffer = vec![0u8; response_size];
    stream.read_exact(&mut response_buffer)?;

    let raw_message: serde_json::Value = serde_json::from_slice(&response_buffer)?;

    if let Some(radar_view) = raw_message.get("RadarView") {
        if radar_view.is_string() {
            return Ok(Message::RadarViewResult(radar_view.as_str().unwrap().to_string()));
        }
    }

    if let Some(challenge) = raw_message.get("Challenge") {
        if let Some(challenge_obj) = challenge.as_object() {
            if let Some(secret_sum_modulo) = challenge_obj.get("SecretSumModulo") {
                if let Some(modulo) = secret_sum_modulo.as_u64() {
                    return Ok(Message::Challenge(ChallengeData::SecretSumModulo(modulo as u128)));
                }
            } else if challenge_obj.contains_key("SOS") {
                return Ok(Message::Challenge(ChallengeData::SOS));
            }
        }
    }

    if let Some(hint) = raw_message.get("Hint") {
        if let Some(hint_obj) = hint.as_object() {
            if let Some(angle) = hint_obj.get("RelativeCompass").and_then(|v| v.as_f64()) {
                return Ok(Message::Hint(HintData::RelativeCompass { angle: angle as f32 }));
            } else if let Some(grid) = hint_obj.get("GridSize").and_then(|v| v.as_object()) {
                if let (Some(columns), Some(rows)) = (grid.get("columns"), grid.get("rows")) {
                    if let (Some(cols), Some(rws)) = (columns.as_u64(), rows.as_u64()) {
                        return Ok(Message::Hint(HintData::GridSize { columns: cols as u32, rows: rws as u32 }));
                    }
                }
            } else if let Some(secret) = hint_obj.get("Secret").and_then(|v| v.as_u64()) {
                return Ok(Message::Hint(HintData::Secret(secret as u128)));
            } else if hint_obj.contains_key("SOSHelper") {
                return Ok(Message::Hint(HintData::SOSHelper));
            }
        }
    }

    let response: Message = serde_json::from_slice(&response_buffer)?;
    Ok(response)
}

/// Traite un message reçu et met à jour l'état du client.
///
/// # Paramètres
/// - `message`: Le message reçu.
/// - `state`: L'état actuel du client.
///
/// # Retourne
/// - `Ok(())` si le traitement est réussi.
/// - `Err(MyError)` en cas d'échec.
pub fn process_message(message: Message, state: &mut ClientState) -> Result<(), MyError> {
    match message {
        Message::RegisterTeamResult(result) => {
            if let Some(success) = result.Ok {
                println!(
                    "Enregistrement réussi - Joueurs : {}, Token : {}",
                    success.expected_players, success.registration_token
                );
                state.team_info = Some(success);
            } else if let Some(error) = result.Err {
                println!("Erreur lors de l'enregistrement : {}", error);
                return Err(error.into());
            } else {
                return Err("Réponse inattendue dans RegisterTeamResult".to_string().into());
            }
        }
        Message::SubscribePlayerResult(result) => match result {
            SubscribePlayerResult::Ok => {
                println!("Souscription réussie !");
            }
            SubscribePlayerResult::Err(error) => {
                println!("Erreur lors de la souscription : {}", error);
                return Err(error.into());
            }
        },
        Message::RadarViewResult(result) => {
            state.radar_view = Some(result);
        }
        _ => println!("Message inattendu !"),
    }

    Ok(())
}

/// Gère la réponse du serveur en la recevant et en la traitant.
///
/// # Paramètres
/// - `stream`: Une référence mutable vers le `TcpStream`.
/// - `state`: L'état actuel du client.
///
/// # Retourne
/// - `Ok(())` si la gestion est réussie.
/// - `Err(MyError)` en cas d'échec.
pub fn handle_response(stream: &mut TcpStream, state: &mut ClientState) -> Result<(), MyError> {
    let response = receive_response(stream)?;
    process_message(response, state)?;
    Ok(())
}
