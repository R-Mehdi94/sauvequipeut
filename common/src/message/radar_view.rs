use serde::{Deserialize, Serialize};

/// Représente le résultat de la vue radar d'un joueur.
///
/// La vue radar est une représentation de l'environnement immédiat du joueur.
/// Elle est **centrée sur le joueur** et affiche une **zone de 3x3 cellules** avec leurs murs.
///
/// # Détails de la vue radar
/// - **12 murs horizontaux** (représentés par `━`).
/// - **12 murs verticaux** (représentés par `┃`).
/// - **9 cellules**, chacune contenant un `RadarItem` (élément de la carte).
///
/// Cette structure est sérialisable et désérialisable via **Serde**, permettant son échange en format JSON.
#[derive(Serialize, Deserialize)]
pub struct RadarViewResult {
    /// Représentation textuelle de la vue radar.
    ///
    /// Cette chaîne contient la structure de la vue radar sous forme de caractères ASCII.
    /// Elle permet d'afficher facilement la disposition des murs et des éléments de la carte.
    ///
    /// # Exemple
    /// ```
    ///
    ///
    /// use common::message::radar_view::RadarViewResult;
    /// let radar_view = RadarViewResult {
    ///     radar_view_result: String::from("ieysGjGO8papd/a"),
    /// };
    ///
    /// assert!(!radar_view.radar_view_result.is_empty());
    /// ```
    pub radar_view_result: String,
}
