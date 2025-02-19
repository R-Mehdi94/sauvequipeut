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

fn format_decoded(decoded: &[u8]) -> String {
    decoded
        .iter()
        .enumerate()
        .map(|(i, &b)| {
            if i < 6 {
                format!("{:08b}", b)
            } else {
                format!("{:02X}", b)
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn decode_and_format(input: &str) -> Result<String, String> {
    custom_decode(input).map(|decoded| format_decoded(&decoded))
}
