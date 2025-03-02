use serde::{Deserialize, Serialize};

/// Représente les informations nécessaires pour enregistrer une nouvelle équipe.
///
/// Cette structure est utilisée pour envoyer une requête d'inscription d'équipe.
///
/// # Exemple
/// ```
///
/// use common::message::RegisterTeam;
/// let team = RegisterTeam {
///     name: String::from("Les Explorateurs"),
/// };
/// assert_eq!(team.name, "Les Explorateurs");
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterTeam {
    /// Nom de l'équipe à enregistrer.
    pub name: String,
}

/// Représente le résultat d'une tentative d'enregistrement d'une équipe.
///
/// Cette structure suit le modèle `Result<RegisterTeamSuccess, String>`, où :
/// - `Ok` contient les détails du succès de l'enregistrement.
/// - `Err` contient un message d'erreur en cas d'échec.
///
/// # Exemple
/// ```
/// use common::message::{RegisterTeamResult, RegisterTeamSuccess};
///
/// let success_result = RegisterTeamResult {
///     Ok: Some(RegisterTeamSuccess {
///         expected_players: 4,
///         registration_token: String::from("ABC123"),
///     }),
///     Err: None,
/// };
///
/// assert!(success_result.Ok.is_some());
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterTeamResult {
    /// Contient les détails de l'inscription réussie, si elle a réussi.
    pub Ok: Option<RegisterTeamSuccess>,

    /// Contient un message d'erreur en cas d'échec de l'inscription.
    pub Err: Option<String>,
}

/// Contient les informations de confirmation lorsqu'une équipe est enregistrée avec succès.
///
/// # Exemple
/// ```
/// use common::message::RegisterTeamSuccess;
///
/// let success = RegisterTeamSuccess {
///     expected_players: 4,
///     registration_token: String::from("XYZ789"),
/// };
///
/// assert_eq!(success.expected_players, 4);
/// assert_eq!(success.registration_token, "XYZ789");
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterTeamSuccess {
    /// Nombre de joueurs attendus pour l'équipe.
    pub expected_players: u32,

    /// Jeton unique d'inscription permettant de finaliser l'enregistrement.
    pub registration_token: String,
}
