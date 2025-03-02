use serde::{Deserialize, Serialize};

/// Représente les informations nécessaires pour inscrire un joueur à une équipe.
///
/// Cette structure est utilisée lors de la phase d'inscription d'un joueur.
///
/// # Exemple
/// ```
///
/// use common::message::SubscribePlayer;
/// let player = SubscribePlayer {
///     name: String::from("Alice"),
///     registration_token: String::from("ABC123"),
/// };
///
/// assert_eq!(player.name, "Alice");
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct SubscribePlayer {
    /// Nom du joueur à inscrire.
    pub name: String,

    /// Jeton d'inscription unique fourni lors de l'enregistrement de l'équipe.
    pub registration_token: String,
}

/// Représente le résultat d'une tentative d'inscription d'un joueur.
///
/// Cette énumération suit le modèle `Result<(), String>`, où :
/// - `Ok` signifie que l'inscription s'est bien déroulée.
/// - `Err(String)` contient un message d'erreur expliquant pourquoi l'inscription a échoué.
///
/// # Exemple
/// ```
///
/// use common::message::SubscribePlayerResult;
/// let success = SubscribePlayerResult::Ok;
/// let failure = SubscribePlayerResult::Err(String::from("Token invalide"));
///
/// if let SubscribePlayerResult::Err(msg) = failure {
///     assert_eq!(msg, "Token invalide");
/// }
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub enum SubscribePlayerResult {
    /// L'inscription du joueur a réussi.
    Ok,

    /// L'inscription a échoué avec un message d'erreur expliquant la raison.
    Err(String),
}
