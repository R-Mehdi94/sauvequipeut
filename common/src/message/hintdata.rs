use serde::{Deserialize, Serialize};

/// Représente les différents types d'indices pouvant être donnés aux joueurs.
///
/// Ces indices permettent d'aider les joueurs à résoudre un défi ou à mieux s'orienter.
///
/// # Sérialisation
/// Ce type est sérialisable et désérialisable avec **Serde**, ce qui permet son utilisation
/// dans des échanges de données (ex: JSON).
#[derive(Serialize, Deserialize, Debug)]
pub enum HintData {
    /// Indice indiquant une direction sous forme d'un angle en degrés par rapport à un compas.
    ///
    /// # Paramètre
    /// - `angle (f32)`: L'angle (en degrés) indiquant la direction relative.
    ///
    /// # Exemple
    /// ```
    ///
    ///
    /// use common::message::hintdata::HintData;
    /// let hint = HintData::RelativeCompass { angle: 45.0 };
    /// ```
    RelativeCompass { angle: f32 },

    /// Indice sur la taille de la grille utilisée dans le jeu.
    ///
    /// # Paramètres
    /// - `columns (u32)`: Nombre de colonnes de la grille.
    /// - `rows (u32)`: Nombre de lignes de la grille.
    ///
    /// # Exemple
    /// ```
    ///
    ///
    /// use common::message::hintdata::HintData;
    /// let hint = HintData::GridSize { columns: 10, rows: 10 };
    /// ```
    GridSize { columns: u32, rows: u32 },

    /// Un indice sous forme d'un nombre secret.
    ///
    /// # Paramètre
    /// - `u128`: Un nombre secret pouvant être utilisé pour aider à résoudre un défi.
    ///
    /// # Exemple
    /// ```
    ///
    ///
    /// use common::message::hintdata::HintData;
    /// let hint = HintData::Secret(123456789);
    /// ```
    Secret(u128),

    /// Un indice générique d'aide appelé **SOSHelper**.
    ///
    /// ⚠️ **Ce type d'indice ne contient actuellement aucune donnée supplémentaire.**
    ///
    /// # Exemple
    /// ```
    ///
    ///
    /// use common::message::hintdata::HintData;
    /// let hint = HintData::SOSHelper;
    /// ```
    SOSHelper,
}
