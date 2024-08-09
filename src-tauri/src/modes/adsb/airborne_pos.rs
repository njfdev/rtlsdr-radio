use crate::modes::types::*;

pub fn decode_aircraft_pos(me: &[u8]) {
    let type_code = me[0] >> 3;
    let altitude_type = if type_code >= 9 && type_code <= 18 {
        AltitudeSource::Barometer
    } else {
        AltitudeSource::GNSS
    };
    let ss_bits = (me[0] >> 1) & 0b11;
    let single_antenna_flag = if (me[0] & 1) == 1 { true } else { false };
    let encoded_alt = ((me[1] as u16) << 4) | (me[2] as u16 >> 4);
    // 0 -> even frame, 1 -> odd frame
    let cpr_format = (me[2] >> 2) & 1;
    let encoded_lat = ((me[2] as u32 & 0b11) << 15) | ((me[3] as u32) << 7) | (me[4] as u32 >> 1);
    let encoded_lon = ((me[4] as u32 & 0b1) << 16) | ((me[5] as u32) << 8) | me[6] as u32;
}
