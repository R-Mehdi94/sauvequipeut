use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use common::message::{Message, MessageData};
use common::message::actiondata::ActionData;
use common::message::challengedata::ChallengeData;
use common::message::hintdata::HintData;
use common::message::message::ActionError;
use common::utils::utils::{build_message, receive_response, send_message};

pub struct TeamSecrets {
    pub secrets: Arc<Mutex<HashMap<u32, u64>>>,
    pub team_size: usize,
}

impl TeamSecrets {
    pub fn new(team_size: usize) -> Self {
        TeamSecrets {
            secrets: Arc::new(Mutex::new(HashMap::new())),
            team_size,
        }
    }

    pub fn update_secret(&self, player_id: u32, secret: u64) {
        let mut secrets = self.secrets.lock().unwrap();
         secrets.insert(player_id, secret);

        println!(" [LOG] Mise à jour des secrets:");
        println!("  - Joueur {}: {}", player_id, secret);
        println!("  - État actuel: {:?}", *secrets);

        // Forcer le flush du println pour s'assurer que les logs sont visibles
        std::io::stdout().flush().unwrap();
    }
    pub fn calculate_sum_modulo(&self, modulo: u64) -> u64 {
        let secrets = self.secrets.lock().unwrap();

        println!("\n🔍 [DEBUG] Début du calcul SecretSumModulo");
        println!("  - Modulo: {}", modulo);
        println!("  - Secrets stockés: {:?}", *secrets);

        // ✅ Somme totale, modulo appliqué une fois à la fin
        let sum: u64 = secrets.values().sum();
        let final_result = sum % modulo;

        println!("🧮 [DEBUG] Somme TOTALE: {}", sum);
        println!("📊 Résultat FINAL (mod {}): {}\n", modulo, final_result);

        std::io::stdout().flush().unwrap();
        final_result
    }


 }

 pub fn debug_secrets(secrets: &TeamSecrets) {
    let guard = secrets.secrets.lock().unwrap();
    println!("\n🔍 [DEBUG] Vérification des secrets:");
    println!("  - Taille de l'équipe: {}", secrets.team_size);
    println!("  - Secrets stockés: {:?}", *guard);
    println!("  - Nombre de secrets: {}", guard.len());
    std::io::stdout().flush().unwrap();
}


pub fn handle_challenge(
    player_id: u32,
    challenge_data: &ChallengeData,
    secrets: &TeamSecrets,
    stream: &mut std::net::TcpStream,
) {
    match challenge_data {
        ChallengeData::SecretSumModulo(modulo) => {
            debug_secrets(secrets);  // ✅ Debug initial

            let mut attempts = 0;
            while attempts < 3 {
                 let mut answer = secrets.calculate_sum_modulo(*modulo);
                println!(" [CALCUL] Résultat calculé (tentative {}): {}", attempts + 1, answer);

                // 🕒 Étape 2 : Vérifier s'il y a un nouveau secret juste avant l'envoi
                let pre_send_wait = Instant::now();
                let mut recalcul_needed = false;
                while pre_send_wait.elapsed() < Duration::from_millis(300) {
                    if let Ok(Message::Hint(HintData::Secret(secret))) = receive_response(stream) {
                        println!("🔑 [AVANT ENVOI] Nouveau secret reçu: {}", secret);
                        secrets.update_secret(player_id, secret);
                        recalcul_needed = true;
                    } else {
                        thread::sleep(Duration::from_millis(50));
                    }
                }

                if recalcul_needed {
                    println!(" [RECALCUL] Nouveau secret détecté, recalcul en cours...");
                    answer = secrets.calculate_sum_modulo(*modulo);
                    println!(" [NOUVEAU CALCUL] Résultat mis à jour: {}", answer);
                }

                // ✅ Étape 3 : Envoi de la réponse
                let solve_message = match build_message(MessageData::Action(ActionData::SolveChallenge {
                    answer: answer.to_string(),
                })) {
                    Ok(message) => message,
                    Err(e) => {
                        eprintln!(" [ERREUR] Construction du message: {:?}", e);
                        return;
                    }
                };

                let json_message = serde_json::to_string(&solve_message).unwrap();
                println!(" [ENVOI] JSON envoyé au serveur : {}", json_message);

                if let Err(e) = send_message(stream, &solve_message) {
                    eprintln!("[ERREUR] Échec de l'envoi : {:?}", e);
                    attempts += 1;
                    continue;
                } else {
                    println!(" [ENVOI] Réponse envoyée avec succès !");
                }

                // 🕒 **Timeout de réception**
                let response_start = Instant::now();
                let mut response_received = false;
                while response_start.elapsed() < Duration::from_secs(3) {
                    match receive_response(stream) {
                        Ok(Message::RadarViewResult(_)) => {
                            println!(" [SUCCÈS] Challenge résolu !");
                            response_received = true;
                            return;
                        }
                        Ok(Message::ActionError(ActionError::InvalidChallengeSolution)) => {
                            println!(" [INVALID] Le serveur a rejeté la solution.");
                            debug_secrets(secrets);
                            attempts += 1;
                            response_received = true;
                            break;
                        }
                        Ok(other) => {
                            println!("⚡ [RÉPONSE] Réponse inattendue : {:?}", other);
                            response_received = true;
                            break;
                        }
                        Err(e) => {
                            eprintln!(" [ERREUR] Réception impossible : {:?}", e);
                            break;
                        }
                    }
                }

                if !response_received {
                    println!(" [ALERTE] Aucune réponse du serveur après 3 secondes.");
                    attempts += 1;
                }
            }
            println!(" [ÉCHEC] Challenge non résolu après {} tentatives.", attempts);
        }
        _ => println!(" Challenge non supporté"),
    }
}
