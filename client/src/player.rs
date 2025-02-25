use std::collections::HashMap;
use crate::utils::connect_to_server;
use common::message::actiondata::{ActionData, PlayerAction};
use common::message::relativedirection::RelativeDirection;
use std::net::TcpStream;
use common::message::{Message, MessageData};
use common::state::ClientState;
use common::utils::utils::{build_message, handle_response, receive_response, send_message};
use rand::{Rng};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use log::{info, warn};
use rand::prelude::IndexedRandom;
use rand::seq::SliceRandom;
use common::message::challengedata::ChallengeData;
use common::message::hintdata::HintData;
use common::message::message::ActionError;
use crate::challenge::{handle_challenge, TeamSecrets};
use crate::decrypte::{decode_and_format, exemple, DecodedView, RadarCell};

pub struct Player {
    pub name: String,
    pub registration_token: String,
    pub stream: TcpStream,
}

fn extract_radar_from_response(response: &Message) -> Option<String> {
    match response {
        Message::RadarViewResult(encoded_radar) => Some(encoded_radar.clone()),
        _ => None,
    }
}

pub fn handle_player(
    player_id: u32,
    token: String,
    players: &Arc<Mutex<Vec<Player>>>,
    addr: &str,
    port: &str,
    tx: Sender<PlayerAction>,
    radar_view: Arc<Mutex<DecodedView>>,
    team_secrets: Arc<TeamSecrets>,
) {
    let mut stream = connect_to_server(addr, port).unwrap();
    let player_name = format!("Player_{}", player_id);

    let subscribe_message = build_message(MessageData::SubscribePlayer {
        name: player_name.clone(),
        registration_token: token.clone(),
    }).unwrap();

    send_message(&mut stream, &subscribe_message).unwrap();
    handle_response(&mut stream, &mut ClientState::default()).unwrap();

    let player = Player {
        name: player_name.clone(),
        registration_token: token.clone(),
        stream: stream.try_clone().unwrap(),
    };

    {
        let mut players_lock = players.lock().unwrap();
        players_lock.push(player);
    }


let mut last_challenge:Option<ChallengeData>=None;
     loop {
          if let Ok(response) = receive_response(&mut stream) {
              println!(
                  "Réponse du serveur pour le joueur {}: {:?}",
                  player_id, response
              );

              match response {
                  Message::Challenge(challenge_data) => {
                      println!(" Challenge reçu pour le joueur {}: {:?}", player_id, challenge_data);
                      last_challenge = Some(challenge_data.clone());
                      handle_challenge(player_id, &challenge_data, &Arc::clone(&team_secrets), &mut stream);

                  }



                  Message::Hint(hint_data) => {
                      println!(" Indice reçu pour le joueur {}: {:?}", player_id, hint_data);
                      if let HintData::Secret(secret_value) = hint_data {
                          println!(" Secret mis à jour pour le joueur {}: {}", player_id, secret_value);
                          team_secrets.update_secret(player_id, secret_value);
                          println!(" Secrets actuels: {:?}", team_secrets.secrets.lock().unwrap());
                      }
                  }
                  Message::RadarViewResult(radar_encoded) => {
                      if let Ok(decoded_radar) = decode_and_format(&radar_encoded) {
                          let radar_data_locked = decoded_radar;
                            exemple (&radar_data_locked);
                           let action = decide_action(&radar_data_locked);


                          tx.send(PlayerAction {
                              player_id,
                              action: action.clone(),
                          }).unwrap();

                          let send_result = send_message(&mut stream, &Message::Action(action));
                          if let Err(e) = send_result {
                               warn!("🔄 Tentative de reconnexion dans 2 secondes...");
                              thread::sleep(Duration::from_secs(2));

                              if let Ok(new_stream) = connect_to_server(addr, port) {
                                  stream = new_stream;

                                  let resubscribe_message = build_message(MessageData::SubscribePlayer {
                                      name: player_name.clone(),
                                      registration_token: token.clone(),
                                  }).unwrap();

                                  if let Err(e) = send_message(&mut stream, &resubscribe_message) {
                                       return;
                                  }
                                  handle_response(&mut stream, &mut ClientState::default()).unwrap();
                               } else {
                                   return;
                              }
                          }
                      }
                  }

                  Message::ActionError(error) => {
                      match error {
                          ActionError::InvalidChallengeSolution=> {
                              println!(" [INVALID] Le serveur a rejeté la solution. 🔄 Recalcul immédiat...: {:?}", error);

                              if let Some(challenge) = &last_challenge {
                                  handle_challenge(player_id, challenge, &Arc::clone(&team_secrets), &mut stream);
                              } else {
                                  println!("⚠️ Aucun challenge précédent trouvé pour recalculer.");
                              }
                          }
                          ActionError::CannotPassThroughWall => {
                              println!("🚧 [MUR] Impossible de passer à travers le mur. 🚫 Changer de direction !: {:?}", error );
                          }
                          _ => {
                              println!("⚠️ [ERREUR NON GÉRÉE] : {:?}", error);
                          }
                      }
                  }

                  _ => println!("🔍 Réponse non gérée pour le joueur {}: {:?}", player_id, response),



               }
          }
      }


}



    fn decode_passage(value: u32) -> bool {
        value == 1
    }



pub fn is_passage_open(passage: u32, bit_index: usize) -> bool {
    let bits = (passage >> (bit_index * 2)) & 0b11;
    bits == 0b01
}

pub fn decide_action(radar: &DecodedView) -> ActionData {
    let front_cell = &radar.cells[1];
    let right_cell = &radar.cells[5];
    let left_cell = &radar.cells[3];

    let right_open =  is_passage_open(radar.get_vertical_passage(1), 2);

    let front_open =  is_passage_open(radar.get_horizontal_passage(1), 2);

    let left_open =  is_passage_open(radar.get_vertical_passage(1), 1);

    if right_open {
        println!("➡️ [ACTION] On va à droite (passage libre)!");
        ActionData::MoveTo(RelativeDirection::Right)
    } else if front_open {
        println!("⬆️ [ACTION] On avance (passage libre)!");
        ActionData::MoveTo(RelativeDirection::Front)
    } else if left_open {
        println!("⬅️ [ACTION] On va à gauche (passage libre)!");
        ActionData::MoveTo(RelativeDirection::Left)
    } else   {
        println!("⬇️ [ACTION] Tout bloqué, on recule (passage libre).");
        ActionData::MoveTo(RelativeDirection::Back)
    }
}









/*fn decide_action(radar_data: &DecodedView, last_action: &mut RelativeDirection) -> ActionData {
    println!(
        "🔍 Analyse détaillée du radar:"
    );

    // Vérification des passages dans chaque direction
    let front_open = DecodedView::is_passage_open(radar_data.get_horizontal_passage(1));
    let right_open = DecodedView::is_passage_open(radar_data.get_vertical_passage(1));
    let left_open = DecodedView::is_passage_open(radar_data.get_vertical_passage(0));
    let back_open = DecodedView::is_passage_open(radar_data.get_horizontal_passage(2));



     if radar_data.is_goal_nearby() {
        println!(" Objectif détecté à proximité!");
    }

     let mut possible_moves = Vec::new();

     if front_open {
        possible_moves.push(RelativeDirection::Front);
    }
    if right_open {
        possible_moves.push(RelativeDirection::Right);
    }
    if left_open {
        possible_moves.push(RelativeDirection::Left);
    }
    if back_open {
        possible_moves.push(RelativeDirection::Back);
    }

     if possible_moves.is_empty() {
         // Rotation systématique
        *last_action = match *last_action {
            RelativeDirection::Front => RelativeDirection::Right,
            RelativeDirection::Right => RelativeDirection::Back,
            RelativeDirection::Back => RelativeDirection::Left,
            RelativeDirection::Left => RelativeDirection::Front,
        };
        return ActionData::MoveTo(*last_action);
    }

    // Essayer d'éviter de revenir sur ses pas si possible
    let best_move = if possible_moves.len() > 1 {
        match *last_action {
            RelativeDirection::Front => {
                if back_open && possible_moves.len() == 1 && possible_moves[0] == RelativeDirection::Back {
                    &RelativeDirection::Back
                } else {
                    possible_moves.iter().find(|&&dir| dir != RelativeDirection::Back).unwrap_or(&possible_moves[0])
                }
            },
            _ => &possible_moves[0]
        }
    } else {
        &possible_moves[0]
    };

    println!(" Direction choisie: {:?}", best_move);
    *last_action = *best_move;
    ActionData::MoveTo(*best_move)
} */