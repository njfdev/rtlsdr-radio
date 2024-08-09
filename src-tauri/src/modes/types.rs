// Mode S
pub const MODES_LONG_MSG_BITS: usize = 112;
pub const MODES_SHORT_MSG_BITS: usize = 56;
pub const MODES_PREAMBLE_US: usize = 8; // preamble length in microseconds

pub const AIS_CHARSET: &str = "?ABCDEFGHIJKLMNOPQRSTUVWXYZ????? ???????????????0123456789??????";

#[derive(Debug)]
pub struct ModeSState {
    pub aircraft: Vec<AircraftState>,
}

impl ModeSState {
    pub fn new() -> Self {
        Self { aircraft: vec![] }
    }
}

#[derive(Clone, Debug)]
pub struct AircraftState {
    pub icao_address: u32,
    pub adsb_state: AdsbState,
}

impl AircraftState {
    pub fn new(icao_address: u32) -> Self {
        Self {
            icao_address,
            adsb_state: AdsbState::new(),
        }
    }
}

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

#[derive(Clone, Debug)]
pub struct AdsbState {
    pub cpr_position: CprPosition,
}

impl AdsbState {
    pub fn new() -> Self {
        Self {
            cpr_position: CprPosition::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CprPosition {
    pub cpr_even_lat: u32,
    pub cpr_odd_lat: u32,
    pub cpr_even_lon: u32,
    pub cpr_odd_lon: u32,
}

impl CprPosition {
    pub fn new() -> Self {
        Self {
            cpr_even_lat: 0,
            cpr_odd_lat: 0,
            cpr_even_lon: 0,
            cpr_odd_lon: 0,
        }
    }
}
