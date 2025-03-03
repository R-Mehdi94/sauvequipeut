use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Instant};
use common::message::{MessageData};
use common::message::actiondata::ActionData;
use common::message::challengedata::ChallengeData;

use common::utils::utils::{build_message, send_message};


pub struct TeamSecrets {
    pub secrets: Arc<Mutex<HashMap<u32, (u128, Instant)>>>,
}

impl TeamSecrets {
    pub fn new() -> Self {
        TeamSecrets {
            secrets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn update_secret(&self, player_id: u32, secret: u128) {
        let mut secrets = self.secrets.lock().unwrap();
        secrets.insert(player_id, (secret, Instant::now()));
        println!("🔄 [UPDATE] Secret mis à jour pour joueur {}: {}", player_id, secret);
    }


    pub fn calculate_sum_modulo(&self, modulo: u128) -> (u128, Instant) {
        let secrets = self.secrets.lock().unwrap();
        let sum: u128 = secrets.values().map(|(value, _)| *value).sum();
        let final_result = sum % modulo;


        let latest_update = secrets.values().map(|(_, ts)| *ts).max().unwrap_or(Instant::now());

        println!(" Somme: {}, Résultat final (mod {}): {}", sum, modulo, final_result);
        (final_result, latest_update)
    }

    pub fn has_secret_updated_after(&self, timestamp: Instant) -> bool {
        let secrets = self.secrets.lock().unwrap();
        secrets.values().any(|(_, ts)| *ts > timestamp)
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
            let last_calculation_time = Instant::now();

            let attempts = 0;


            if secrets.has_secret_updated_after(last_calculation_time) {
                println!(" [INFO] Mise à jour détectée avant recalcul.");
            }
            let (answer, _) = secrets.calculate_sum_modulo(*modulo);
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

            } else {
                println!("  Réponse envoyée avec succès !");
                return;
            }



        }
        _ => println!("️ [INFO] Challenge non supporté."),
    }
}