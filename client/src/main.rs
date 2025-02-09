mod utils;

use crate::utils::connect_to_server;
use common::message::MessageData;
use common::state::ClientState;
use common::utils::*;
use std::error::Error;

fn main() -> Result<(), MyError> {
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
            let context = format!("Erreur critique lors de la connexion à {}:{}", addr, port);
            return Err(MyError::Other(format!("{}: {:?}", context, e)));
        }
    };

    let mut state = ClientState::default();

    let message = build_message(MessageData::RegisterTeam {
        name: "curious_broccoli".to_string(),
    })?;

    if let Err(e) = send_message(&mut stream, &message) {
        eprintln!("Erreur critique lors de l'envoi du message : {:?}", e);
        return Err(e);
    }

    handle_response(&mut stream, &mut state)?;

    let (expected_players, token) = if let Some(team_info) = &state.team_info {
        (
            team_info.expected_players.clone(),
            team_info.registration_token.clone(),
        )
    } else {
        println!("No team info available.");
        return Ok(());
    };

    let player = {
        let expected_players = expected_players.clone();
        let mut count = 0u32;
        let token = token.clone();
        std::thread::spawn(move || loop {
            count += 1;
            if count > expected_players {
                return Ok(());
            }
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
                eprintln!("Erreur critique lors de l'envoi du message : {:?}", e);
                return Err(e);
            };

            handle_response(&mut stream, &mut state)?;
        })
    };

    player.join().unwrap()?;

    Ok(())
}
