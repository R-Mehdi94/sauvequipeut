use common::message::{Message, MessageData, RegisterTeamResult, RegisterTeamSuccess, SubscribePlayerResult};
use common::utils::my_error::MyError;
use common::utils::utils::{build_message, Player};

use std::net::{TcpListener, TcpStream};
use uuid::Uuid;
use common::utils::utils::{receive_response, send_message};

fn main() {
    println!("Server is running on localhost:8778");
    let listener = TcpListener::bind("127.0.0.1:8778").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => match handle_connection(&mut stream) {
                Ok(_) => {
                    println!("Connection closed: {:?}", stream.peer_addr());
                }
                Err(e) => {
                    println!("ERROR Failed to handle connection: {:?}", e);
                }
            },
            Err(e) => {
                println!("ERROR Failed to establish connection: {:?}", e);
            }
        }
    }
}

fn handle_connection(stream: &mut TcpStream) -> Result<(), MyError> {

    let expected_players = 0;
    let registration_token= String::from("registration_token");
    println!("New connection: {:?}", stream.peer_addr()?);
    let mut message = receive_response(stream)?;


    if let Message::RegisterTeam(register_team) = message {
        println!("Registering team: {:?}", register_team);
        let team_success = RegisterTeamSuccess {
            expected_players: 1,
            registration_token: Uuid::new_v4().to_string(),
        };

        let result = RegisterTeamResult {
            Ok: Some(team_success),
            Err: None,
        };
        send_message(stream, &Message::RegisterTeamResult(result))?;
    } else if let Message::SubscribePlayer(_subscribe_player) = message {

        let mut players: Vec<Player> = vec![];

        for _i in 0..expected_players {
            message = receive_response(stream)?;
            if let Message::SubscribePlayer(subscribe_player) = message {
                println!("Subscribing player: {:?}", subscribe_player);
                if subscribe_player.registration_token == registration_token {
                    let player = Player { name: subscribe_player.name, register_token: subscribe_player.registration_token };
                    players.push(player);
                    let message = build_message(MessageData::SubscribePlayerResult(SubscribePlayerResult::Ok));
                    println!("SubscribeResult: {:?}", message);

                    send_message(stream,&message?)?
                }

            }
        }

        println!("Players received: {:?}", players.len());
    }


    let message = build_message(MessageData::RadarView(String::from("ieysGjGO8papd/a")))?;
    println!("Sending first radar view");
    send_message(stream, &message)?;
    println!("First radar sent, waiting for player action...");


    let message = receive_response(stream)?;
    println!("after message     ");

    if let Message::Action(action_data) = message {
        println!("Action received: {:?}", action_data);
        let radar_view2 = Message::RadarView("jiucAjGa//cpapa".to_string());
        println!("Sending second radar view");
        send_message(stream, &radar_view2)?;
    } else {
        println!("Expected ActionData, received: {:?}", message);
        return Ok(());
    }

    let message = receive_response(stream)?;
    if let Message::Action(action_data) = message {
        println!("Second action received: {:?}", action_data);
        println!("Scenario completed successfully");
    } else {
        println!("Expected second ActionData, received: {:?}", message);
    }


    Ok(())
}