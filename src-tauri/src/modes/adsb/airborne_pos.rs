use std::f64::consts::PI;

use fundsp::typenum::Pow;
use nalgebra::ComplexField;
use rustfft::num_complex::ComplexFloat;

use crate::modes::types::*;

// # of latitude zones between the equator and a pole
const N_Z: f64 = 15.0;

pub fn decode_aircraft_pos(me: &[u8], adsb_state: &mut AdsbState) {
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

    let lat_cpr = (encoded_lat as f64) / 2.0_f64.powi(17);
    let lon_cpr = (encoded_lon as f64) / 2.0_f64.powi(17);

    // if no previous cpr value, store this one and move on
    if adsb_state.cpr_position.is_none() {
        adsb_state.cpr_position = Some(CprPosition {
            cpr_lat: lat_cpr,
            cpr_lon: lon_cpr,
            cpr_format,
        });
    } else if adsb_state.cpr_position.clone().unwrap().cpr_format != cpr_format {
        // if previous cpr value and formats are different, calculate with it and reset it
        let lat_even_cpr = if cpr_format == 0 {
            lat_cpr
        } else {
            adsb_state.cpr_position.clone().unwrap().cpr_lat
        };
        let lat_odd_cpr = if cpr_format == 1 {
            lat_cpr
        } else {
            adsb_state.cpr_position.clone().unwrap().cpr_lat
        };
        let lon_even_cpr = if cpr_format == 0 {
            lon_cpr
        } else {
            adsb_state.cpr_position.clone().unwrap().cpr_lon
        };
        let lon_odd_cpr = if cpr_format == 1 {
            lon_cpr
        } else {
            adsb_state.cpr_position.clone().unwrap().cpr_lon
        };

        let calculation_result = calc_lat_long(
            lat_even_cpr,
            lat_odd_cpr,
            lon_even_cpr,
            lon_odd_cpr,
            cpr_format,
        );

        if calculation_result.is_ok() {
            let (lat, lon) = calculation_result.unwrap();
            println!("Latitude: {}", lat);
            println!("Longitude: {}", lon);
        } else {
            println!("Error in decoding Latitude and Longitude!");
        }

        adsb_state.cpr_position = None;
    } else {
        // if there is a previous cpr and the formats are the same, assume we missed a message, and reset
        adsb_state.cpr_position = None;
    }
}

fn calculate_lon_zones(latitude: f64) -> u8 {
    if latitude == 0.0 {
        return 59;
    } else if latitude.abs() == 87.0 {
        return 2;
    } else if latitude.abs() > 87.0 {
        return 1;
    }

    ((2.0 * PI)
        / (1.0 - ((1.0 - (PI / (2.0 * N_Z)).cos()) / (((PI / 180.0) * latitude).cos().powi(2))))
            .acos())
    .floor() as u8
}

fn calc_lat_long(
    lat_even_cpr: f64,
    lat_odd_cpr: f64,
    lon_even_cpr: f64,
    lon_odd_cpr: f64,
    most_recent_format: u8,
) -> Result<(f64, f64), ()> {
    let lat_even_zone_size = 360.0 / (4.0 * N_Z);
    let lat_odd_zone_size = 360.0 / (4.0 * N_Z - 1.0);

    let lat_zone_index = ((59.0 * lat_even_cpr) - (60.0 * lat_odd_cpr) + 0.5).floor();

    // calculate even and odd latitude
    let lat_even = lat_even_zone_size * ((lat_zone_index % 60.0) + lat_even_cpr);
    let lat_odd = lat_odd_zone_size * ((lat_zone_index % 59.0) + lat_odd_cpr);

    // if lat even and odd are in different zones, then return err
    if calculate_lon_zones(lat_even) != calculate_lon_zones(lat_odd) {
        return Err(());
    }

    // if the 2 latitudes are in the same zone, use the most recent one
    let final_lat = if most_recent_format == 0 {
        lat_even
    } else {
        lat_odd
    };

    Ok((final_lat, 0.0))
}
