use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use serde_json::json;
use common::message::{Message, MessageData};
use common::message::actiondata::ActionData;
use common::message::challengedata::ChallengeData;
use common::message::hintdata::HintData;
use common::message::message::ActionError;
use common::utils::utils::{build_message, receive_response, send_message};


pub struct TeamSecrets {
    pub(crate) secrets: Arc<Mutex<HashMap<u32, u64>>>,
    last_update: Arc<Mutex<Instant>>,
}

impl TeamSecrets {
    pub fn new() -> Self {
        TeamSecrets {
            secrets: Arc::new(Mutex::new(HashMap::new())),
            last_update: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn update_secret(&self, player_id: u32, secret: u64) {
        let mut secrets = self.secrets.lock().unwrap();
        secrets.insert(player_id, secret);
        *self.last_update.lock().unwrap() = Instant::now();
        println!(" [UPDATE] Secret mis à jour pour joueur {}: {}", player_id, secret);
    }

    pub fn calculate_sum_modulo(&self, modulo: u64) -> (u64, Instant) {
        let secrets = self.secrets.lock().unwrap();
        let last_update = *self.last_update.lock().unwrap();

        println!("\n🔍 [DEBUG] Début du calcul SecretSumModulo");
        println!("  - Modulo: {}", modulo);
        println!("  - Secrets stockés: {:?}", *secrets);

        // Calcul de la somme totale
        let sum: u64 = secrets.values().sum();
        let final_result = sum % modulo;

        println!(" [DEBUG] Somme TOTALE: {}", sum);
        println!(" Résultat FINAL (mod {}): {}\n", modulo, final_result);
        std::io::stdout().flush().unwrap();

        (final_result, last_update)
    }

    pub fn has_been_updated_since(&self, timestamp: Instant) -> bool {
        *self.last_update.lock().unwrap() > timestamp
    }
}


pub fn handle_challenge(
    player_id: u32,
    challenge_data: &ChallengeData,
    secrets: &TeamSecrets,
    stream: &mut std::net::TcpStream,
) {
    match challenge_data {
        ChallengeData::SecretSumModulo(modulo) => {
            println!(" [INFO] Challenge SecretSumModulo reçu pour le joueur {} avec modulo {}", player_id, modulo);

            let mut attempts = 0;
            while attempts < 3 {

                 let (mut answer, _) = secrets.calculate_sum_modulo(*modulo);
                println!("premier calcule  Résultat (tentative {}): {}", attempts + 1, answer);


                let solve_message = match build_message(MessageData::Action(ActionData::SolveChallenge {
                    answer: answer.to_string(),
                })) {
                    Ok(message) => message,
                    Err(e) => {
                        eprintln!(" erreur Construction du message: {:?}", e);
                        return;
                    }
                };


                println!("  JSON envoyé au serveur : {}", serde_json::to_string(&solve_message).unwrap());
                if let Err(e) = send_message(stream, &solve_message) {
                    eprintln!(" Échec de l'envoi : {:?}", e);
                    attempts += 1;
                    continue;
                } else {
                    println!("  Réponse envoyée avec succès !");
                }


                match receive_response(stream) {
                    Ok(Message::RadarViewResult(_)) => {
                        println!("  [SUCCÈS] Challenge résolu !");
                        return;
                    }
                    Ok(Message::ActionError(ActionError::InvalidChallengeSolution)) => {
                        println!("  [INVALID] Le serveur a rejeté la solution.  Recalcul en cours...");
                        //  Recalcul immédiat sans attendre de nouveaux secrets
                        let (new_answer, _) = secrets.calculate_sum_modulo(*modulo);
                        println!(" [NOUVEAU CALCUL] Résultat après rejet : {}", new_answer);
                        answer = new_answer;
                        attempts += 1;
                    }
                    Ok(other) => {
                        println!("⚡ [RÉPONSE] Réponse inattendue : {:?}", other);
                        attempts += 1;
                    }
                    Err(e) => {
                        eprintln!(" [ERREUR] Problème lors de la réception : {:?}", e);
                        attempts += 1;
                    }
                }
            }
            println!(" [ECHEC] Échec après {} tentatives.", attempts);
        }
        _ => println!("️ [INFO] Challenge non supporté."),
    }
}
