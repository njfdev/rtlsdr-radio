// Mode S
pub const MODES_LONG_MSG_BITS: usize = 112;
pub const MODES_SHORT_MSG_BITS: usize = 56;
pub const MODES_PREAMBLE_US: usize = 8; // preamble length in microseconds

pub const AIS_CHARSET: &str = "?ABCDEFGHIJKLMNOPQRSTUVWXYZ????? ???????????????0123456789??????";

// ADS-B
#[derive(PartialEq)]
pub enum AltitudeSource {
    GNSS,
    Barometer,
}

#[derive(PartialEq)]
pub enum AirspeedType {
    IAS, // indicated airspeed
    TAS, // true airspeed
}
