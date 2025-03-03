use crate::message::relativedirection::RelativeDirection;
use serde::{Deserialize, Serialize};

/// Représente les différentes actions qu'un joueur peut effectuer.
#[derive(Serialize, Deserialize, Debug, Clone,PartialEq)]
pub enum ActionData {
    /// Déplace le joueur dans une direction relative.
    ///
    /// # Exemple
    /// ```
    ///
    ///
    ///
    /// use common::message::actiondata::ActionData;
    /// use common::message::relativedirection::RelativeDirection;
    /// let action = ActionData::MoveTo(RelativeDirection::Back);
    /// ```
    MoveTo(RelativeDirection),

    /// Résout un défi en soumettant une réponse sous forme de chaîne de caractères.
    ///
    /// # Exemple
    /// ```
    /// use common::message::actiondata::ActionData;
    ///
    /// let action = ActionData::SolveChallenge { answer: "42".to_string() };
    /// ```
    SolveChallenge { answer: String },
}

/// Représente une action effectuée par un joueur.
///
/// Cette structure contient l'identifiant du joueur ainsi que l'action qu'il réalise.
///
/// # Exemple
/// ```
///
/// use common::message::actiondata::{ActionData, PlayerAction};
/// let action = ActionData::SolveChallenge { answer: "42".to_string() };
/// let player_action = PlayerAction {
///     player_id: 1,
///     action,
/// };
/// assert_eq!(player_action.player_id, 1);
/// ```
pub struct PlayerAction {
    /// L'identifiant unique du joueur effectuant l'action.
    pub player_id: u32,

    /// L'action effectuée par le joueur.
    pub action: ActionData,
}
