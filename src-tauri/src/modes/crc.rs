use crate::modes::{get_message_length, types::*};

// Precalculated values for ADS-B checksum for each bit of the 112 bits.
const MODES_CHECKSUM_TABLE: [u32; 112] = [
    0x3935ea, 0x1c9af5, 0xf1b77e, 0x78dbbf, 0xc397db, 0x9e31e9, 0xb0e2f0, 0x587178, 0x2c38bc,
    0x161c5e, 0x0b0e2f, 0xfa7d13, 0x82c48d, 0xbe9842, 0x5f4c21, 0xd05c14, 0x682e0a, 0x341705,
    0xe5f186, 0x72f8c3, 0xc68665, 0x9cb936, 0x4e5c9b, 0xd8d449, 0x939020, 0x49c810, 0x24e408,
    0x127204, 0x093902, 0x049c81, 0xfdb444, 0x7eda22, 0x3f6d11, 0xe04c8c, 0x702646, 0x381323,
    0xe3f395, 0x8e03ce, 0x4701e7, 0xdc7af7, 0x91c77f, 0xb719bb, 0xa476d9, 0xadc168, 0x56e0b4,
    0x2b705a, 0x15b82d, 0xf52612, 0x7a9309, 0xc2b380, 0x6159c0, 0x30ace0, 0x185670, 0x0c2b38,
    0x06159c, 0x030ace, 0x018567, 0xff38b7, 0x80665f, 0xbfc92b, 0xa01e91, 0xaff54c, 0x57faa6,
    0x2bfd53, 0xea04ad, 0x8af852, 0x457c29, 0xdd4410, 0x6ea208, 0x375104, 0x1ba882, 0x0dd441,
    0xf91024, 0x7c8812, 0x3e4409, 0xe0d800, 0x706c00, 0x383600, 0x1c1b00, 0x0e0d80, 0x0706c0,
    0x038360, 0x01c1b0, 0x00e0d8, 0x00706c, 0x003836, 0x001c1b, 0xfff409, 0x000000, 0x000000,
    0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000,
    0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000, 0x000000,
    0x000000, 0x000000, 0x000000, 0x000000,
];

fn compute_modes_crc(msg: Vec<u8>, msg_bits: usize) -> u32 {
    let mut crc: u32 = 0;
    let offset = if msg_bits == MODES_LONG_MSG_BITS {
        0
    } else {
        MODES_LONG_MSG_BITS - MODES_SHORT_MSG_BITS
    };

    for i in 0..msg_bits {
        let byte = i / 8;
        let bit = i % 8;
        let bitmask = 1 << (7 - bit);

        if msg[byte] & bitmask > 0 {
            crc ^= MODES_CHECKSUM_TABLE[i + offset];
        }
    }

    crc
}

pub fn perform_modes_crc(msg: Vec<u8>) -> Result<Vec<u8>, ()> {
    let msg_type = msg[0] >> 3;
    let msg_bits = get_message_length(msg_type);

    // the crc is always the last 3 bytes
    let received_crc = ((msg[(msg_bits / 8) - 3] as u32) << 16)
        | ((msg[(msg_bits / 8) - 2] as u32) << 8)
        | msg[(msg_bits / 8) - 1] as u32;
    let computed_crc = compute_modes_crc(msg.clone(), msg_bits);

    if received_crc != computed_crc {
        return Err(());
    }

    print!("Valid Mode S Message Demodulated: ");
    for byte in msg.clone() {
        print!("{:08b}", byte);
    }
    println!();

    Ok(msg)
}
