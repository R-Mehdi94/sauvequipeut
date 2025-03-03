/// Décode une chaîne encodée en base modifiée et retourne un vecteur d'octets.
///
/// Ce format d'encodage suit un schéma similaire au Base64, mais avec quelques modifications.
///
/// # Paramètres
/// - `input`: La chaîne encodée à décoder.
///
/// # Retourne
/// - Un `Result<Vec<u8>, String>` contenant les données décodées ou un message d'erreur en cas d'échec.
///
/// # Exemple
/// ```
///
/// use algorithms::decrypte::custom_decode;
/// let decoded = custom_decode("aBcD");
/// assert!(decoded.is_ok());
/// ```
pub fn custom_decode(input: &str) -> Result<Vec<u8>, String> {
    let char_to_value = |c: char| -> Result<u8, String> {
        match c {
            'a'..='z' => Ok(c as u8 - b'a'),
            'A'..='Z' => Ok(c as u8 - b'A' + 26),
            '0'..='9' => Ok(c as u8 - b'0' + 52),
            '+' => Ok(62),
            '/' => Ok(63),
            _ => Err(format!("Invalid character: {}", c)),
        }
    };

    let mut result = Vec::new();
    let chars: Vec<char> = input.chars().collect();

    for chunk in chars.chunks(4) {
        let n = chunk.len();
        let mut accum = 0u32;

        for (_i, &c) in chunk.iter().enumerate() {
            accum = (accum << 6) | (char_to_value(c)? as u32);
        }

        let bytes = match n {
            4 => vec![(accum >> 16) as u8, (accum >> 8) as u8, accum as u8],
            3 => vec![(accum >> 10) as u8, (accum >> 2) as u8],
            2 => vec![(accum >> 4) as u8],
            _ => vec![],
        };

        result.extend_from_slice(&bytes);
    }

    Ok(result)
}

/// Représente une vue radar décodée à partir d'une entrée encodée.
///
/// La vue radar contient des passages horizontaux, verticaux et des cellules indiquant l'environnement.
#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct DecodedView {
    /// Passages horizontaux de la vue radar.
    pub(crate) horizontal_passages: [u32; 4],

    /// Passages verticaux de la vue radar.
    pub(crate) vertical_passages: [u32; 3],

    /// Liste des cellules de la grille.
    pub cells: Vec<RadarCell>,
}

impl Default for DecodedView {
    fn default() -> Self {
        DecodedView {
            horizontal_passages: [0; 4],
            vertical_passages: [0; 3],
            cells: vec![RadarCell::Undefined; 9],
        }
    }
}

/// Représente une cellule du radar.
///
/// Chaque cellule peut avoir différents états en fonction de son contenu.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RadarCell {
    /// Cellule indéfinie (non analysée).
    Undefined,

    /// Cellule ouverte (espace libre).
    Open,

    /// Cellule indiquant une sortie.
    Exit,

    /// Cellule inconnue avec un code spécifique.
    Unknown(String),
}

impl RadarCell {
    /// Crée une cellule de radar à partir d'une séquence de bits.
    ///
    /// # Paramètres
    /// - `bits`: Une chaîne de 4 bits décrivant la cellule.
    ///
    /// # Retourne
    /// - Une instance de `RadarCell` correspondant aux bits donnés.
    ///
    /// # Exemple
    /// ```
    /// use algorithms::decrypte::RadarCell;
    ///
    /// let cell = RadarCell::from_bits("0000");
    /// assert_eq!(cell, RadarCell::Open);
    /// ```
    pub fn from_bits(bits: &str) -> Self {
        match bits {
            "1111" => RadarCell::Undefined,
            "0000" => RadarCell::Open,
            "1000" | "1001" => RadarCell::Exit,
            _ => RadarCell::Unknown(bits.to_string()),
        }
    }
}

/// Analyse la partie contenant les cellules et retourne une liste de `RadarCell`.
fn parse_cells_part(cells_part: &str) -> Vec<RadarCell> {
    let mut cells = Vec::new();
    for i in 0..9 {
        let cell_bits = &cells_part[i * 4..(i + 1) * 4];
        cells.push(RadarCell::from_bits(cell_bits));
    }
    cells
}

impl DecodedView {
    /// Retourne l'état d'un passage horizontal donné.
    ///
    /// # Paramètres
    /// - `index`: L'index du passage (de 0 à 3).
    ///
    /// # Retourne
    /// - La valeur du passage horizontal correspondant.
    pub fn get_horizontal_passage(&self, index: usize) -> u32 {
        self.horizontal_passages[index]
    }

    /// Retourne l'état d'un passage vertical donné.
    ///
    /// # Paramètres
    /// - `index`: L'index du passage (de 0 à 2).
    ///
    /// # Retourne
    /// - La valeur du passage vertical correspondant.
    pub fn get_vertical_passage(&self, index: usize) -> u32 {
        self.vertical_passages[index]
    }
}

/// Formate les données décodées en une structure `DecodedView`.
///
/// # Paramètres
/// - `decoded`: Un slice d'octets contenant les données brutes décodées.
///
/// # Retourne
/// - Un `DecodedView` en cas de succès, ou un message d'erreur en cas d'échec.
fn format_decoded(decoded: &[u8]) -> Result<DecodedView, String> {
    if decoded.len() < 11 {
        return Err("Input too short".to_string());
    }

    let horizontal = u32::from_le_bytes([decoded[0], decoded[1], decoded[2], 0]);
    let vertical = u32::from_le_bytes([decoded[3], decoded[4], decoded[5], 0]);

    let cells_bits: String = decoded[6..]
        .iter()
        .map(|byte| format!("{:08b}", byte))
        .collect();

    Ok(DecodedView {
        horizontal_passages: [
            (horizontal >> 18) & 0b111111,
            (horizontal >> 12) & 0b111111,
            (horizontal >> 6) & 0b111111,
            horizontal & 0b111111,
        ],
        vertical_passages: [
            (vertical >> 16) & 0xFF,
            (vertical >> 8) & 0xFF,
            vertical & 0xFF,
        ],
        cells: parse_cells_part(&cells_bits),
    })
}

/// Décode une chaîne encodée et la formate en `DecodedView`.
///
/// # Paramètres
/// - `input`: La chaîne encodée.
///
/// # Retourne
/// - Une instance de `DecodedView` en cas de succès, ou un message d'erreur en cas d'échec.
pub fn decode_and_format(input: &str) -> Result<DecodedView, String> {
    custom_decode(input).and_then(|decoded| format_decoded(&decoded))
}

/// Vérifie si un passage est ouvert à un certain index de bit.
///
/// # Paramètres
/// - `passage`: La valeur du passage.
/// - `bit_index`: L'index du bit à vérifier.
///
/// # Retourne
/// - `true` si le passage est ouvert, `false` sinon.
///
/// # Exemple
/// ```
///
/// use algorithms::decrypte::is_passage_open;
/// let open = is_passage_open(0b010000, 1);
/// assert!(open);
/// ```

pub fn is_passage_open(passage: u32, bit_index: usize) -> bool {

    let corrected_index = 3- bit_index;
    let bits = (passage >> (corrected_index * 2)) & 0b11;

    println!(
        "🔎 Vérification passage: bits = {:02b}, bit_index = {}, corrected_index = {}",
        bits, bit_index, corrected_index
    );

    match bits {
        0b01 => {
            println!(" Passage ouvert !");
            true
        }
        0b00 | 0b10 => {
            println!(" Passage fermé !");
            false
        }
        _ => {
            println!("⚠Valeur inattendue !");
            false
        }
    }
}
