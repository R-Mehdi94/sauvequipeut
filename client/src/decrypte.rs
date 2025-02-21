use std::fmt;

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

struct DecodedView {
    horizontal_passages: [u32; 4], // 4 lignes de 6 bits
    vertical_passages: [u32; 3],   // 3 lignes de 8 bits
    cells: Vec<u8>,
}

impl DecodedView {
    fn get_horizontal_passage(&self, index: usize) -> u32 {
        self.horizontal_passages[index]
    }

    fn get_vertical_passage(&self, index: usize) -> u32 {
        self.vertical_passages[index]
    }

    fn get_cellules(&self) -> String {
        let mut bits = String::new();
        for &cell in &self.cells {
            bits.push_str(&format!("{:08b}", cell));
        }

        let mut grouped_hex = String::new();
        for chunk in bits.chars().collect::<Vec<char>>().chunks(4) {
            let chunk_str: String = chunk.iter().collect();
            let chunk_value = u8::from_str_radix(&chunk_str, 2).unwrap_or(0);
            grouped_hex.push_str(&format!("{:X}", chunk_value));
        }

        grouped_hex
            .chars()
            .collect::<Vec<char>>()
            .chunks(3)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl fmt::Display for DecodedView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Passages horizontaux
        writeln!(
            f,
            "Passages horizontaux: {:08b} {:08b} {:08b}",
            self.horizontal_passages[0], self.horizontal_passages[1], self.horizontal_passages[2]
        )?;
        writeln!(
            f,
            "=> {:08b} {:08b} {:08b} (inversion des octets, car écrit en little endian)",
            self.horizontal_passages[2], self.horizontal_passages[1], self.horizontal_passages[0]
        )?;
        writeln!(
            f,
            "=> {:06b} {:06b} {:06b} {:06b} (4 lignes de 3 passages)\n",
            self.horizontal_passages[2] >> 2,
            ((self.horizontal_passages[2] & 0b11) << 4) | (self.horizontal_passages[1] >> 4),
            ((self.horizontal_passages[1] & 0b1111) << 2) | (self.horizontal_passages[0] >> 6),
            self.horizontal_passages[0] & 0b111111
        )?;

        // Passages verticaux
        writeln!(
            f,
            "Passages verticaux: {:08b} {:08b} {:08b}",
            self.vertical_passages[0], self.vertical_passages[1], self.vertical_passages[2]
        )?;
        writeln!(
            f,
            "=> {:08b} {:08b} {:08b} (inversion des octets, car écrit en little endian)",
            self.vertical_passages[2], self.vertical_passages[1], self.vertical_passages[0]
        )?;
        writeln!(
            f,
            "=> {:08b} {:08b} {:08b} (3 lignes de 4 passages)\n",
            self.vertical_passages[2], self.vertical_passages[1], self.vertical_passages[0]
        )?;

        // Cellules

        write!(f, "Les cellules: ")?;
        for &cell in &self.cells {
            write!(f, "{:02X} ", cell)?;
        }
        write!(f, "=> ")?;

        let mut bits = String::new();
        for &cell in &self.cells {
            bits.push_str(&format!("{:08b}", cell));
        }

        let mut grouped_hex = String::new();
        for chunk in bits.chars().collect::<Vec<char>>().chunks(4) {
            let chunk_str: String = chunk.iter().collect();
            let chunk_value = u8::from_str_radix(&chunk_str, 2).unwrap_or(0);
            grouped_hex.push_str(&format!("{:X}", chunk_value));
        }

        for chunk in grouped_hex.chars().collect::<Vec<char>>().chunks(3) {
            write!(f, "{} ", chunk.iter().collect::<String>())?;
        }

        if grouped_hex.len() % 3 != 0 {
            write!(
                f,
                "(le {} final étant du padding)",
                "0".repeat(3 - grouped_hex.len() % 3)
            )?;
        }

        Ok(())
    }
}

fn format_decoded(decoded: &[u8]) -> Result<DecodedView, String> {
    if decoded.len() < 11 {
        return Err("Input too short".to_string());
    }

    let horizontal = u32::from_le_bytes([decoded[0], decoded[1], decoded[2], 0]);
    let vertical = u32::from_le_bytes([decoded[3], decoded[4], decoded[5], 0]);

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
        cells: decoded[6..].to_vec(),
    })
}

pub fn decode_and_format(input: &str) -> Result<DecodedView, String> {
    custom_decode(input).and_then(|decoded| format_decoded(&decoded))
}

fn exemple() {
    let input = "ieysGjGO8papd/a";
    let test = decode_and_format(input).unwrap();

    println!("{}", test);

    println!("Bits: {:06b}", test.get_horizontal_passage(1));
    println!("Bits: {:06b}", test.get_vertical_passage(1));
    println!("Bits: {:}", test.get_cellules());
}
