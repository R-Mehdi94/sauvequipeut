use serde::{Deserialize, Serialize};

/// Représente les différents types de défis pouvant être rencontrés dans le jeu.
///
/// Chaque défi a ses propres règles et objectifs.
///
/// # Sérialisation
/// Ce type est sérialisable et désérialisable avec Serde, ce qui permet son stockage et son envoi en JSON.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ChallengeData {
    /// Un défi où l'objectif est de deviner un nombre secret modifié par un modulo.
    ///
    /// # Paramètre
    /// - `u128` : Le nombre secret utilisé pour le défi.
    ///
    /// # Exemple
    /// ```
    ///
    /// use common::message::challengedata::ChallengeData;
    /// let challenge = ChallengeData::SecretSumModulo(42);
    /// ```
    SecretSumModulo(u128),

    /// Un défi spécifique appelé **SOS**.
    ///
    /// ⚠️ **Actuellement, aucun paramètre n'est défini pour ce type de défi.**
    ///
    /// # Exemple
    /// ```
    ///
    /// use common::message::challengedata::ChallengeData;
    /// let challenge = ChallengeData::SOS;
    /// ```
    SOS,
}
