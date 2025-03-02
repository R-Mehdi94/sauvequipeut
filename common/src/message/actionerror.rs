use serde::{Deserialize, Serialize};

/// Représente les différentes erreurs pouvant survenir lorsqu'un joueur effectue une action.
///
/// Ces erreurs sont généralement liées aux règles du jeu et aux contraintes imposées
/// aux déplacements et interactions des joueurs.
///
/// # Sérialisation
/// Ce type est sérialisé en `PascalCase` grâce à `#[serde(rename_all = "PascalCase")]`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ActionError {
    /// Le joueur tente de passer à travers un mur, ce qui est interdit.
    CannotPassThroughWall,

    /// Le joueur tente de passer à travers un autre joueur, ce qui est interdit.
    CannotPassThroughOpponent,

    /// Le joueur tente de résoudre un défi alors qu'aucun défi n'est en cours.
    NoRunningChallenge,

    /// Le joueur doit d'abord résoudre le défi actuel avant de poursuivre.
    SolveChallengeFirst,

    /// La solution fournie pour le défi est incorrecte.
    InvalidChallengeSolution,
}
