use std::collections::HashMap;
use common::message::relativedirection::RelativeDirection;

/// Suit l'exploration d'un joueur en enregistrant les **positions visitées** et les **directions prises**.
///
/// - Permet de détecter des **boucles** si un joueur repasse plusieurs fois par la même position.
/// - Garde en mémoire les **dernières positions** explorées.
/// - Stocke la **dernière direction empruntée**.
pub struct ExplorationTracker {
    /// Positions visitées avec le nombre de fois où elles ont été explorées.
    pub(crate) visited_positions: HashMap<(i32, i32), usize>,

    /// Historique des dernières positions explorées.
    last_positions: Vec<(i32, i32)>,

    /// Dernière direction empruntée par le joueur.
    last_direction: Option<RelativeDirection>,
}

impl ExplorationTracker {
    /// Crée une nouvelle instance de `ExplorationTracker` initialisée avec des structures vides.
    ///
    /// # Retourne
    /// - Une instance de `ExplorationTracker`.
    ///
    /// # Exemple
    /// ```
    /// use ma_lib::ExplorationTracker;
    ///
    /// let tracker = ExplorationTracker::new();
    /// assert!(tracker.visited_positions.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            visited_positions: HashMap::new(),
            last_positions: Vec::new(),
            last_direction: None,
        }
    }

    /// Marque une position comme **visitée** et met à jour l'historique des déplacements.
    ///
    /// - Incrémente le compteur de visites pour cette position.
    /// - Ajoute la position à l'historique des **dernières positions** (limité à 8).
    /// - Stocke la dernière direction empruntée.
    /// - Détecte si une **boucle** a été formée (le joueur repasse par une position récente).
    ///
    /// # Paramètres
    /// - `position`: Coordonnées `(x, y)` de la position visitée.
    /// - `direction`: Direction prise pour arriver à cette position.
    ///
    /// # Exemple
    /// ```
    /// use ma_lib::ExplorationTracker;
    /// use common::message::relativedirection::RelativeDirection;
    ///
    /// let mut tracker = ExplorationTracker::new();
    /// tracker.mark_position((3, 3), RelativeDirection::Front);
    /// ```
    pub fn mark_position(&mut self, position: (i32, i32), direction: RelativeDirection) {
        let count = self.visited_positions.entry(position).or_insert(0);
        *count += 1;

        println!("📝 [DEBUG] Ajout de la position {:?} avec direction {:?}", position, direction);
        println!("📌 [DEBUG] Avant ajout, last_positions: {:?}", self.last_positions);

        self.last_positions.push(position);
        if self.last_positions.len() > 8 {
            self.last_positions.remove(0);
        }

        self.last_direction = Some(direction);

        if self.last_positions.len() >= 3 && self.last_positions[..self.last_positions.len() - 1].contains(&position) {
            println!("🔄 [ALERTE] Boucle détectée à {:?}", position);
        }

        println!("📌 [DEBUG] Après ajout, last_positions: {:?}", self.last_positions);
        println!("➡️ [DEBUG] Dernière direction prise : {:?}", self.last_direction);
    }

    /// Vérifie si une position a été **récemment visitée**.
    ///
    /// - Une position est considérée comme récemment visitée si elle figure dans l'historique des **dernières 5 positions** explorées.
    ///
    /// # Paramètres
    /// - `position`: Coordonnées `(x, y)` à vérifier.
    ///
    /// # Retourne
    /// - `true` si la position a été récemment visitée.
    /// - `false` sinon.
    ///
    /// # Exemple
    /// ```
    /// use ma_lib::ExplorationTracker;
    ///
    /// let mut tracker = ExplorationTracker::new();
    /// tracker.mark_position((2, 2), common::message::relativedirection::RelativeDirection::Front);
    /// assert!(tracker.is_recently_visited((2, 2)));
    /// ```
    pub fn is_recently_visited(&self, position: (i32, i32)) -> bool {
        if self.last_positions.len() < 5 {
            return false;
        }
        println!("🔍 [DEBUG] Vérification de la position {} dans is_recently_visited() {}", position.0, position.1);
        self.last_positions[..self.last_positions.len() - 1].contains(&position)
    }
}
