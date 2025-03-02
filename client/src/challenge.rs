use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use common::message::{MessageData};
use common::message::actiondata::ActionData;
use common::message::challengedata::ChallengeData;
use common::utils::utils::{build_message, send_message};

/// Gère les secrets partagés par l'équipe.
///
/// Cette structure permet de stocker les **secrets des joueurs** et de les mettre à jour.
/// Elle offre également des fonctionnalités pour **calculer la somme des secrets modulo** une valeur donnée.
pub struct TeamSecrets {
    /// Une table contenant les secrets des joueurs, associant un `player_id` à un `secret` et un `Instant` (horodatage de mise à jour).
    pub secrets: Arc<Mutex<HashMap<u32, (u128, Instant)>>>,
}

impl TeamSecrets {
    /// Crée une nouvelle instance de `TeamSecrets`.
    ///
    /// # Retourne
    /// - Une instance de `TeamSecrets` avec un `HashMap` vide.
    ///
    /// # Exemple
    /// ```
    /// use ma_lib::TeamSecrets;
    ///
    /// let team_secrets = TeamSecrets::new();
    /// assert!(team_secrets.secrets.lock().unwrap().is_empty());
    /// ```
    pub fn new() -> Self {
        TeamSecrets {
            secrets: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Met à jour le **secret** d'un joueur.
    ///
    /// # Paramètres
    /// - `player_id`: L'identifiant du joueur.
    /// - `secret`: Le secret du joueur.
    ///
    /// # Exemple
    /// ```
    /// use ma_lib::TeamSecrets;
    ///
    /// let team_secrets = TeamSecrets::new();
    /// team_secrets.update_secret(1, 42);
    /// ```
    pub fn update_secret(&self, player_id: u32, secret: u128) {
        let mut secrets = self.secrets.lock().unwrap();
        secrets.insert(player_id, (secret, Instant::now()));
        println!("🔄 [UPDATE] Secret mis à jour pour joueur {}: {}", player_id, secret);
    }

    /// Calcule la somme des secrets modulo une valeur donnée.
    ///
    /// # Paramètres
    /// - `modulo`: La valeur du modulo à appliquer.
    ///
    /// # Retourne
    /// - Un tuple contenant :
    ///   - `u128` : Le résultat de `(somme des secrets) % modulo`
    ///   - `Instant` : L'horodatage de la mise à jour la plus récente.
    ///
    /// # Exemple
    /// ```
    /// use ma_lib::TeamSecrets;
    ///
    /// let team_secrets = TeamSecrets::new();
    /// team_secrets.update_secret(1, 100);
    /// let (result, _) = team_secrets.calculate_sum_modulo(10);
    /// assert_eq!(result, 0);
    /// ```
    pub fn calculate_sum_modulo(&self, modulo: u128) -> (u128, Instant) {
        let secrets = self.secrets.lock().unwrap();
        let sum: u128 = secrets.values().map(|(value, _)| *value).sum();
        let final_result = sum % modulo;

        let latest_update = secrets.values().map(|(_, ts)| *ts).max().unwrap_or(Instant::now());

        println!(" Somme: {}, Résultat final (mod {}): {}", sum, modulo, final_result);
        (final_result, latest_update)
    }

    /// Vérifie si un **secret** a été mis à jour après un instant donné.
    ///
    /// # Paramètres
    /// - `timestamp`: L'horodatage de référence.
    ///
    /// # Retourne
    /// - `true` si un secret a été mis à jour après `timestamp`, `false` sinon.
    ///
    /// # Exemple
    /// ```
    /// use ma_lib::TeamSecrets;
    /// use std::time::Instant;
    ///
    /// let team_secrets = TeamSecrets::new();
    /// let timestamp = Instant::now();
    /// team_secrets.update_secret(1, 50);
    ///
    /// assert!(team_secrets.has_secret_updated_after(timestamp));
    /// ```
    pub fn has_secret_updated_after(&self, timestamp: Instant) -> bool {
        let secrets = self.secrets.lock().unwrap();
        secrets.values().any(|(_, ts)| *ts > timestamp)
    }
}

/// Gère un **challenge** reçu par un joueur.
///
/// Si le challenge est de type `SecretSumModulo`, cette fonction :
/// - Vérifie si des mises à jour de secret ont eu lieu.
/// - Calcule la somme modulo et envoie la réponse au serveur.
///
/// # Paramètres
/// - `player_id`: L'identifiant du joueur qui reçoit le challenge.
/// - `challenge_data`: Les données du challenge.
/// - `secrets`: L'objet `TeamSecrets` contenant les secrets de l'équipe.
/// - `stream`: Une connexion `TcpStream` vers le serveur.
///
/// # Exemple
/// ```no_run
/// use std::net::TcpStream;
/// use ma_lib::{handle_challenge, TeamSecrets};
/// use common::message::challengedata::ChallengeData;
///
/// let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
/// let team_secrets = TeamSecrets::new();
/// handle_challenge(1, &ChallengeData::SecretSumModulo(100), &team_secrets, &mut stream);
/// ```
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

            if secrets.has_secret_updated_after(last_calculation_time) {
                println!(" [INFO] Mise à jour détectée avant recalcul.");
            }

            let (answer, _) = secrets.calculate_sum_modulo(*modulo);
            println!("Premier calcul - Résultat: {}", answer);

            let solve_message = match build_message(MessageData::Action(ActionData::SolveChallenge {
                answer: answer.to_string(),
            })) {
                Ok(message) => message,
                Err(e) => {
                    eprintln!("Erreur lors de la construction du message: {:?}", e);
                    return;
                }
            };

            println!("📤 JSON envoyé au serveur : {}", serde_json::to_string(&solve_message).unwrap());
            if let Err(e) = send_message(stream, &solve_message) {
                eprintln!("❌ Échec de l'envoi : {:?}", e);
            } else {
                println!("✅ Réponse envoyée avec succès !");
            }
        }
        _ => println!("️⚠️ [INFO] Challenge non supporté."),
    }
}
