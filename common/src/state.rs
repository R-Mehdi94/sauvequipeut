use crate::message::{RegisterTeamSuccess, SubscribePlayer};
use crate::utils::utils::Player;

/// Représente l'état actuel du client du jeu.
///
/// Cette structure stocke des informations sur l'équipe, les joueurs inscrits et la vue radar.
///
/// # Détails
/// - Contient les informations de l'équipe si elle est enregistrée.
/// - Stocke la liste des joueurs inscrits.
/// - Conserve la dernière vue radar reçue.
///
/// # Exemple
/// ```
///
/// use common::message::{RegisterTeamSuccess, SubscribePlayer};
/// use common::state::ClientState;
/// let mut client_state = ClientState::default();
///
/// // Ajout d'informations sur l'équipe
/// client_state.team_info = Some(RegisterTeamSuccess {
///     expected_players: 4,
///     registration_token: String::from("ABC123"),
/// });
///
/// // Inscription d'un joueur
/// client_state.subscribed_players.push(SubscribePlayer {
///     name: String::from("Alice"),
///     registration_token: String::from("ABC123"),
/// });
///
/// // Mise à jour de la vue radar
/// client_state.radar_view = Some(String::from("ieysGjGO8papd/a"));
///
/// assert!(client_state.team_info.is_some());
/// assert_eq!(client_state.subscribed_players.len(), 1);
/// assert!(client_state.radar_view.is_some());
/// ```
#[derive(Default)]
pub struct ClientState {
    /// Informations sur l'équipe enregistrée, si elle existe.
    pub team_info: Option<RegisterTeamSuccess>,

    /// Liste des joueurs actuellement inscrits.
    pub subscribed_players: Vec<SubscribePlayer>,
    pub players: Vec<Player>,

    /// Dernière vue radar reçue sous forme de chaîne de caractères.
    ///
    /// Cette chaîne représente une **zone de 3x3 cellules** autour du joueur,
    /// avec des murs et des éléments de la carte.
    pub radar_view: Option<String>,
}
