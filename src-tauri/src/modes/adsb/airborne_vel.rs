use std::f32::consts::PI;

use crate::modes::types::*;

pub fn decode_airborne_vel(me: &[u8]) {
    let subtype = me[0] & 0b111;
    let _intent_change_flag = if me[1] >> 7 == 1 { true } else { false };
    let _ifr_capability = if (me[1] >> 6) & 1 == 1 { true } else { false };
    let subtype_specific_data = ((me[1] as u32 & 0b111) << 19)
        | ((me[2] as u32) << 11)
        | ((me[3] as u32) << 3)
        | (me[4] as u32 >> 5);
    let vertical_rate_source = if (me[4] >> 3) & 1 == 1 {
        AltitudeSource::Barometer
    } else {
        AltitudeSource::GNSS
    };
    // 1 means down and 0 means up
    let vertical_rate_sign = if (me[4] >> 3) & 1 == 1 { -1 } else { 0 };
    let vertical_rate_raw = ((me[4] as u16 & 0b111) << 6) | (me[5] as u16 >> 2);

    if vertical_rate_raw != 0 {
        let vertical_rate = ((vertical_rate_raw as i32) - 1) * 64 * (vertical_rate_sign as i32);
        println!(
            "Vertical Velocity ({}): {} ft/min",
            if vertical_rate_source == AltitudeSource::GNSS {
                "GNSS"
            } else {
                "Barometer"
            },
            vertical_rate
        );

        // derive the other velocity type if it exists
        let mut velocity_source_difference_sign = if (me[5] >> 7) == 1 { -1 } else { 1 };
        // swap if vertical rate source is GNSS
        if vertical_rate_source == AltitudeSource::GNSS {
            velocity_source_difference_sign = -velocity_source_difference_sign;
        }
        let velocity_source_difference_raw = me[5] & 0b111_1111;

        if velocity_source_difference_raw != 0 {
            let velocity_source_difference =
                (velocity_source_difference_raw as i32 - 1) * 25 * velocity_source_difference_sign;
            println!(
                "Vertical Velocity ({}): {} ft/min",
                if vertical_rate_source != AltitudeSource::GNSS {
                    "GNSS"
                } else {
                    "Barometer"
                },
                vertical_rate + velocity_source_difference
            );
        }
    } else {
        println!("Vertical Velocity: N/A");
    }

    // subtype 1/3 -> subsonic, subtype 2/4 -> supersonic
    let speed_multiplier = if subtype % 2 == 0 { 4 } else { 1 };

    // decode subtype data
    match subtype {
        // ground speed
        1..=2 => {
            // 0 -> West to East (1), 1 -> East to West (-1)
            let ew_sign: i16 = if (subtype_specific_data >> 21) == 1 {
                -1
            } else {
                1
            };
            let ew_velocity_raw = (subtype_specific_data >> 11) as u16 & 0b11_1111_1111;
            let mut ew_velocity_abs: Option<u16> = None;
            if ew_velocity_raw != 0 {
                ew_velocity_abs = Some((ew_velocity_raw as u16 - 1) * speed_multiplier);
            }

            // 0 -> South to North (1), 1 -> North to South (-1)
            let ns_sign: i16 = if ((subtype_specific_data >> 10) & 1) == 1 {
                -1
            } else {
                1
            };
            let ns_velocity_raw = subtype_specific_data as u16 & 0b11_1111_1111;
            let mut ns_velocity_abs: Option<u16> = None;
            if ns_velocity_raw != 0 {
                ns_velocity_abs = Some((ns_velocity_raw as u16 - 1) * speed_multiplier);
            }

            if ns_velocity_abs.is_some() && ew_velocity_abs.is_some() {
                // convert to heading relative to north
                let angle = ((ew_velocity_abs.unwrap() as i16 * ew_sign) as f32)
                    .atan2((ns_velocity_abs.unwrap() as i16 * ns_sign) as f32)
                    * (360.0 / (2.0 * PI))
                    % 360.0;

                println!("Heading: {:.2}°", angle);
            }

            print!(
                "Ground Speed ({}): ",
                if subtype == 1 {
                    "subsonic"
                } else {
                    "supersonic"
                }
            );
            if ns_velocity_abs.is_some() && ew_velocity_abs.is_some() {
                let real_speed = ((ns_velocity_abs.unwrap() as f32).powi(2)
                    + (ew_velocity_abs.unwrap() as f32).powi(2))
                .sqrt();
                println!("{} knots", real_speed);
            } else {
                println!()
            }
            if ns_velocity_abs.is_some() {
                println!(
                    "  {} knots {}",
                    ns_velocity_abs.unwrap(),
                    if ns_sign > 0 { "North" } else { "South" }
                );
            }
            if ew_velocity_abs.is_some() {
                println!(
                    "  {} knots {}",
                    ew_velocity_abs.unwrap(),
                    if ew_sign > 0 { "East" } else { "West" }
                );
            }
        }
        // air speed
        3..=4 => {
            let is_magnetic_heading_included = if (subtype_specific_data >> 21) == 1 {
                true
            } else {
                false
            };
            let magnetic_heading_raw = (subtype_specific_data >> 11) as u16 & 0b11_1111_1111;
            let mut magnetic_heading: Option<f32> = None;

            if is_magnetic_heading_included {
                magnetic_heading = Some((magnetic_heading_raw as f32) * (360.0 / 1024.0));
            }

            let airspeed_type = if (subtype_specific_data >> 10) & 1 == 1 {
                AirspeedType::TAS
            } else {
                AirspeedType::IAS
            };
            let airspeed_raw = subtype_specific_data as u16 & 0b11_1111_1111;
            let mut airspeed: Option<u16> = None;

            if airspeed_raw != 0 {
                airspeed = Some((airspeed_raw - 1) * speed_multiplier);
            }

            println!(
                "Magnetic Heading: {}",
                if is_magnetic_heading_included {
                    format!("{:.2}°", magnetic_heading.unwrap())
                } else {
                    "N/A".to_string()
                }
            );
            println!(
                "{} Airspeed: {}",
                if airspeed_type == AirspeedType::TAS {
                    "True"
                } else {
                    "Indicated"
                },
                if airspeed.is_some() {
                    format!("{} knots", airspeed.unwrap())
                } else {
                    "N/A".to_string()
                }
            )
        }
        _ => {}
    }
}
