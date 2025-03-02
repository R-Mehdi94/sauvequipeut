use serde::{Deserialize, Serialize};

/// Représente une direction relative par rapport à l'orientation actuelle du joueur.
///
/// Cette énumération permet de spécifier un mouvement ou une orientation en fonction de la position actuelle.
///
/// # Sérialisation
/// - Cette énumération est sérialisable et désérialisable avec **Serde**, permettant son stockage et son envoi en JSON.
/// - Elle implémente `Clone`, `Copy` et `PartialEq` pour faciliter sa manipulation.
///
/// # Exemple
/// ```
///
/// use common::message::relativedirection::RelativeDirection;
/// let direction = RelativeDirection::Left;
/// assert_eq!(direction, RelativeDirection::Left);
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum RelativeDirection {
    /// Direction **gauche** par rapport à l'orientation actuelle.
    Left,

    /// Direction **droite** par rapport à l'orientation actuelle.
    Right,

    /// Direction **devant** par rapport à l'orientation actuelle.
    Front,

    /// Direction **derrière** par rapport à l'orientation actuelle.
    Back,
}
