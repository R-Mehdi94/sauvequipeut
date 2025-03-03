use std::sync::{Arc, Mutex};
use common::message::hintdata::HintData;

use common::message::relativedirection::RelativeDirection;


/// Détermine une liste de directions en fonction d'un angle donné.
///
/// L'angle est **normalisé** entre 0° et 360° avant de classer les directions par ordre de priorité.
///
/// # Paramètres
/// - `angle`: L'angle donné en degrés (peut être négatif ou supérieur à 360°).
///
/// # Retourne
/// - Un `Vec<RelativeDirection>` contenant les directions triées par priorité.
///
/// # Exemple
/// ```
/// use ma_lib::direction_from_angle;
/// use common::message::relativedirection::RelativeDirection;
///
/// let directions = direction_from_angle(30.0);
/// assert_eq!(directions[0], RelativeDirection::Front);
/// ```
pub fn direction_from_angle(angle: f32) -> Vec<RelativeDirection> {
    let normalized_angle = ((angle % 360.0) + 360.0) % 360.0;
    println!("🧭 [INFO] Angle normalisé : {:.2}°", normalized_angle);

    match normalized_angle {
        a if a <= 45.0 || a > 315.0 => {
            println!("⬆️ [DIRECTION] Priorité : Front > Right > Left > Back");
            vec![RelativeDirection::Front, RelativeDirection::Right, RelativeDirection::Left, RelativeDirection::Back]
        }
        a if a > 45.0 && a <= 135.0 => {
            println!("➡️ [DIRECTION] Priorité : Right > Front > Back > Left");
            vec![RelativeDirection::Right, RelativeDirection::Front, RelativeDirection::Back, RelativeDirection::Left]
        }
        a if a > 135.0 && a <= 225.0 => {
            println!("🔄 [DIRECTION] Priorité : Back > Left > Right > Front");
            vec![RelativeDirection::Back, RelativeDirection::Left, RelativeDirection::Right, RelativeDirection::Front]
        }
        _ => {
            println!("⬅️ [DIRECTION] Priorité : Left > Front > Back > Right");
            vec![RelativeDirection::Left, RelativeDirection::Front, RelativeDirection::Back, RelativeDirection::Right]
        }
    }
}

/// Détermine une liste de directions en fonction de la **taille du labyrinthe**.
///
/// - Si la grille est **plus large** que haute, les déplacements horizontaux sont favorisés.
/// - Si la grille est **plus haute** que large, les déplacements verticaux sont favorisés.
///
/// # Paramètres
/// - `grid_size`: Option contenant le nombre de **colonnes** et de **lignes**.
///
/// # Retourne
/// - Un `Vec<RelativeDirection>` contenant les directions triées par priorité.
///
/// # Exemple
/// ```
/// use ma_lib::direction_from_grid_size;
/// use common::message::relativedirection::RelativeDirection;
///
/// let directions = direction_from_grid_size(Some((10, 5)));
/// assert_eq!(directions[0], RelativeDirection::Right);
/// ```
pub fn direction_from_grid_size(grid_size: Option<(u32, u32)>) -> Vec<RelativeDirection> {
    if let Some((cols, rows)) = grid_size {
        if cols > rows {
            println!("➡️ [STRATÉGIE] Labyrinthe large ➔ Priorité : Droite > Gauche > Haut > Bas");
            vec![
                RelativeDirection::Right,
                RelativeDirection::Left,
                RelativeDirection::Front,
                RelativeDirection::Back,
            ]
        } else {
            println!("⬆️ [STRATÉGIE] Labyrinthe haut ➔ Priorité : Haut > Bas > Droite > Gauche");
            vec![
                RelativeDirection::Front,
                RelativeDirection::Back,
                RelativeDirection::Right,
                RelativeDirection::Left,
            ]
        }
    } else {
        println!("⚠️ [INFO] GridSize non connue ➔ Priorité par défaut : Haut > Droite > Gauche > Bas.");
        vec![
            RelativeDirection::Front,
            RelativeDirection::Right,
            RelativeDirection::Left,
            RelativeDirection::Back,
        ]
    }
}


/// Gère un indice (`HintData`) et met à jour les informations partagées.
///
/// - **Boussole** : Met à jour l'orientation vers la sortie et assigne un leader.
/// - **Taille du labyrinthe** : Met à jour la taille de la grille.
/// - **SOS** : Informe que le joueur a demandé de l'aide.
///
/// # Paramètres
/// - `player_id`: Identifiant du joueur recevant l'indice.
/// - `hint_data`: L'indice reçu.
/// - `shared_compass`: Référence partagée pour stocker **l'angle de la boussole**.
/// - `leader_id`: Référence partagée pour stocker **l'identifiant du leader**.
/// - `shared_grid_size`: Référence partagée pour stocker **la taille de la grille**.
///
/// # Exemple
/// ```
/// use std::sync::{Arc, Mutex};
/// use ma_lib::handle_hint;
/// use common::message::hintdata::HintData;
///
/// let compass = Arc::new(Mutex::new(None));
/// let leader_id = Arc::new(Mutex::new(None));
/// let grid_size = Arc::new(Mutex::new(None));
///
/// let hint = HintData::RelativeCompass { angle: 90.0 };
/// handle_hint(1, &hint, &compass, &leader_id, &grid_size);
/// ```
pub fn handle_hint(
    player_id: u32,
    hint_data: &HintData,

    shared_compass: &Arc<Mutex<Option<f32>>>,
    leader_id: &Arc<Mutex<Option<u32>>>,
    shared_grid_size: &Arc<Mutex<Option<(u32, u32)>>>

) {
    match hint_data {
        HintData::RelativeCompass { angle } => {
            println!(
                "🧭 [INFO] Boussole reçue pour le joueur {}: {:.2}° vers la sortie.",
                player_id, angle
            );

            let mut compass = shared_compass.lock().unwrap();
            *compass = Some(*angle);
            println!("🧭 [INFO] Boussole partagée mise à jour : {:.2}°", angle);


            let mut leader = leader_id.lock().unwrap();
            if leader.is_none() || leader.unwrap() != player_id {
                println!("👑 [LEADER] Le joueur {} devient le leader.", player_id);
                *leader = Some(player_id);
            }

        }

        HintData::GridSize { columns, rows } => {
            println!(
                "🗺️ [INFO] Grille reçue: {} colonnes x {} lignes.",
                columns, rows
            );
            let mut grid_size = shared_grid_size.lock().unwrap();
            *grid_size = Some((*columns, *rows));
            println!("🗺️ [INFO] GridSize partagée mise à jour : {}x{}", columns, rows);

        }

        HintData::SOSHelper => {
            println!("🆘 [INFO] SOS reçu pour le joueur {}", player_id);
            return ;

        }
        _ => {}
    }
}