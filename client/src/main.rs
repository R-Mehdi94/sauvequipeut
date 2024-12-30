mod utils;

use crate::utils::connect_to_server;
use common::message::MessageData;
use common::state::ClientState;
use common::utils::*;

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

    let mut state = ClientState::default();

    let message = build_message(MessageData::RegisterTeam {
        name: "curious_broccoli".to_string(),
    })?;

    if let Err(e) = send_message(&mut stream, &message) {
        eprintln!("Erreur critique lors de l'envoi du message : {}", e);
        return Err(e);
    }

    match receive_response(&mut stream) {
        Ok(response) => {
            if let Err(e) = process_message(response, &mut state) {
                eprintln!("Erreur Critique lors du traitement du message : {}", e);
                return Err(e);
            }
        }
        Err(e) => {
            eprintln!("Erreur critique lors de la réception : {}", e);
            return Err(e);
        }
    };

    let (expected_players, token) = if let Some(team_info) = &state.team_info {
        (
            team_info.expected_players.clone(),
            team_info.registration_token.clone(),
        )
    } else {
        println!("No team info available.");
        return Ok(());
    };

    for _ in 0..expected_players {
        stream = connect_to_server(addr, port)?;

        let mut line = String::new();
        println!("Enter your name :");
        std::io::stdin().read_line(&mut line)?;
        let user = line.trim();

        let message = build_message(MessageData::SubscribePlayer {
            name: user.to_string(),
            registration_token: token.clone(),
        })?;

        if let Err(e) = send_message(&mut stream, &message) {
            eprintln!("Erreur critique lors de l'envoi du message : {}", e);
            return Err(e);
        }

        match receive_response(&mut stream) {
            Ok(response) => {
                if let Err(e) = process_message(response, &mut state) {
                    eprintln!("Erreur Critique lors du traitement du message : {}", e);
                    return Err(e);
                }
            }
            Err(e) => {
                eprintln!("Erreur critique lors de la réception : {}", e);
                return Err(e);
            }
        };
    }

    Ok(())
}
