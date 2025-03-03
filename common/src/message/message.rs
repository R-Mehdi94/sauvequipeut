use crate::message::actiondata::ActionData;
use crate::message::challengedata::ChallengeData;
use crate::message::hintdata::HintData;
use crate::message::registerteam::{RegisterTeam, RegisterTeamResult};
use crate::message::subscribeplayer::{SubscribePlayer, SubscribePlayerResult};
use serde::{Deserialize, Serialize};
use crate::message::actionerror::ActionError;

/// Représente les **données d'un message** pouvant être échangées.
///
/// Cette énumération permet de structurer les différents types de messages envoyés ou reçus.
///
/// # Variantes
/// - `RegisterTeam`: Enregistre une équipe.
/// - `SubscribePlayer`: Inscrit un joueur.
/// - `Hint`: Représente un indice.
/// - `Action`: Contient une action effectuée par un joueur.
/// - `Challenge`: Contient un défi reçu.
///
/// # Exemple
/// ```
///
/// use common::message::MessageData;
/// let message = MessageData::RegisterTeam {
///     name: "TeamX".to_string(),
/// };
/// ```
pub enum MessageData {
    /// Demande d'inscription d'une équipe.
    RegisterTeam {
        /// Nom de l'équipe à enregistrer.
        name: String,
    },

    /// Demande d'inscription d'un joueur.
    SubscribePlayer {
        /// Nom du joueur à inscrire.
        name: String,
        /// Jeton de connexion du joueur.
        registration_token: String,
    },

    /// Contient un **indice** reçu.
    Hint(HintData),

    /// Représente une **action effectuée par un joueur**.
    Action(ActionData),

    /// Contient un **challenge** à résoudre.
    Challenge(ChallengeData),

    RadarView(String),
    SubscribePlayerResult(SubscribePlayerResult),


}

/// Représente un **message** échangé dans le système.
///
/// Cette énumération regroupe tous les types de messages possibles.
///
/// # Variantes
/// - `RegisterTeam`: Message pour enregistrer une équipe.
/// - `RegisterTeamResult`: Résultat de l'enregistrement d'une équipe.
/// - `SubscribePlayer`: Message pour inscrire un joueur.
/// - `SubscribePlayerResult`: Résultat de l'inscription.
/// - `Hint`: Contient un indice.
/// - `Action`: Contient une action effectuée.
/// - `Challenge`: Contient un challenge.
/// - `RadarViewResult`: Contient la vue radar sous forme de texte.
/// - `ActionError`: Erreur liée à une action.
///
/// # Exemple
/// ```
/// use common::message::Message;
/// use common::message::registerteam::RegisterTeam;
///
/// let message = Message::RegisterTeam(RegisterTeam { name: "TeamX".to_string() });
/// ```
#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    /// Message pour enregistrer une équipe.
    RegisterTeam(RegisterTeam),

    /// Résultat de l'inscription d'une équipe.
    RegisterTeamResult(RegisterTeamResult),

    /// Résultat de l'inscription d'un joueur.
    SubscribePlayerResult(SubscribePlayerResult),

    /// Message pour inscrire un joueur.
    SubscribePlayer(SubscribePlayer),

    SubscribePlayerResultClient(SubscribePlayer),

    /// Contient un **indice** reçu.
    Hint(HintData),

    /// Représente une **action effectuée par un joueur**.
    Action(ActionData),

    /// Contient un **challenge** à résoudre.
    Challenge(ChallengeData),

    /// Résultat d'une **vue radar**, représentée sous forme de chaîne de caractères.
    RadarViewResult(String),
    RadarView(String),

    /// Indique une **erreur lors d'une action**.
    ActionError(ActionError),
}
