use std::sync::{Arc, Mutex};
use common::message::hintdata::HintData;

use common::message::relativedirection::RelativeDirection;



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

pub fn handle_hint(
    player_id: u32,
    hint_data: &HintData,

    shared_compass: &Arc<Mutex<Option<f32>>>,
    leader_id: &Arc<Mutex<Option<u32>>>,
    shared_grid_size: &Arc<Mutex<Option<(u32, u32)>>>,
    hint_received: &Arc<Mutex<bool>>


) {
    match hint_data {
        HintData::RelativeCompass { angle } => {
            let mut hint_flag = hint_received.lock().unwrap();
            *hint_flag = true;
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