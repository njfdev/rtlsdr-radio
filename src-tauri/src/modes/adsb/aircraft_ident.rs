use crate::modes::types::*;

// Indexed by [TC-1][CA]
const WAKE_VORTEX_CATEGORY_TABLE: [[&str; 8]; 4] = [
    [
        "Reserved", "Reserved", "Reserved", "Reserved", "Reserved", "Reserved", "Reserved",
        "Reserved",
    ],
    [
        "No category information",
        "Surface emergency vehicle",
        "No category information",
        "Surface service vehicle",
        "Ground obstruction",
        "Ground obstruction",
        "Ground obstruction",
        "Ground obstruction",
    ],
    [
        "No category information",
        "Glider, sailplane",
        "Lighter-than-air",
        "Parachutist, skydiver",
        "Ultralight, hang-glider, paraglider",
        "Reserved",
        "Unmanned aerial vehicle",
        "Space or transatmospheric vehicle",
    ],
    [
        "No category information",
        "Light (less than 7000 kg)",
        "Medium 1 (between 7000 kg and 34000 kg)",
        "Medium 2 (between 34000 kg to 136000 kg)",
        "High vortex aircraft",
        "Heavy (larger than 136000 kg)",
        "High performance (>5 g acceleration) and high speed (>400 kt)",
        "Rotorcraft",
    ],
];

pub fn decode_aircraft_ident(me: &[u8]) {
    let tc = me[0] >> 3;
    let ca = me[0] & 0b111;
    let wake_vortex_category = *(WAKE_VORTEX_CATEGORY_TABLE.get(tc as usize - 1).unwrap())
        .get(ca as usize)
        .unwrap();

    // callsign is a 8 character code
    let callsign: String = extract_each_6_bits(me[1..].to_vec())
        .into_iter()
        .map(|char_byte| AIS_CHARSET.as_bytes()[char_byte as usize] as char)
        .collect();

    println!("Callsign: {}", callsign);
    println!("Wake Vortex Category: {}", wake_vortex_category);
}

fn extract_each_6_bits(data: Vec<u8>) -> Vec<u8> {
    let mut new_array: Vec<u8> = vec![];
    let mut current_byte: u8 = 0;
    let mut new_bit_index: u8 = 0;

    for byte in data {
        for bit_i in (0..8).rev() {
            new_bit_index += 1;
            current_byte |= ((byte >> bit_i) & 1) << (6 - new_bit_index);

            if new_bit_index >= 6 {
                new_array.push(current_byte.clone());

                current_byte = 0;
                new_bit_index = 0;
            }
        }
    }

    new_array
}
