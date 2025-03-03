fn custom_decode(input: &str) -> Result<Vec<u8>, String> {
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
#[derive(Clone,Eq, Hash, PartialEq,Debug)]
pub struct DecodedView {
    pub(crate) horizontal_passages: [u32; 4],
    pub(crate) vertical_passages: [u32; 3],
    pub(crate) cells: Vec<RadarCell>,
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
#[derive(Clone ,Debug ,PartialEq ,Eq, Hash)]
pub enum RadarCell {
    Undefined,
    Open,
    Exit,
    Unknown(String),
}

impl RadarCell {
    pub fn from_bits(bits: &str) -> Self {
        match bits {
            "1111" => RadarCell::Undefined,
            "0000" => RadarCell::Open,
            "1000"|"1001" => RadarCell::Exit,
            _ => RadarCell::Unknown(bits.to_string()),
        }
    }
}

fn parse_cells_part(cells_part: &str) -> Vec<RadarCell> {
    let mut cells = Vec::new();
    for i in 0..9 {
        let cell_bits = &cells_part[i * 4..(i + 1) * 4];
        cells.push(RadarCell::from_bits(cell_bits));
    }
    cells
}
impl DecodedView {
    pub fn get_horizontal_passage(&self, index: usize) -> u32 {
        self.horizontal_passages[index]

    }
    pub fn get_vertical_passage(&self, index: usize) -> u32 {
        self.vertical_passages[index]
    }

}


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

pub fn decode_and_format(input: &str) -> Result<DecodedView, String> {
    custom_decode(input).and_then(|decoded| format_decoded(&decoded))
}

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
