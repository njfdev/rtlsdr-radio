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
#[derive(PartialEq, Clone, Debug)]
pub enum AltitudeSource {
    GNSS,
    Barometer,
}

#[derive(PartialEq, Clone, Debug)]
pub enum AirspeedType {
    IAS, // indicated airspeed
    TAS, // true airspeed
}

#[derive(PartialEq, Clone, Debug)]
pub enum SpeedType {
    // heading is magnetic based
    GroundSpeed,
    // heading is GNSS based
    AirSpeed(AirspeedType),
}

#[derive(PartialEq, Clone, Debug)]
pub enum SpeedCategory {
    Subsonic,
    Supersonic,
}

#[derive(Clone, Debug)]
pub struct AdsbState {
    pub cpr_position: Option<CprPosition>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub altitude: Option<i32>,
    pub altitude_source: Option<AltitudeSource>,
    pub callsign: Option<String>,
    pub wake_vortex_cat: Option<String>,
    pub gnss_vertical_velocity: Option<i32>,
    pub barometer_vertical_velocity: Option<i32>,
    pub preferred_vertical_velocity_source: Option<AltitudeSource>,
    pub heading: Option<f32>,
    pub speed: Option<u16>,
    pub speed_category: Option<SpeedCategory>,
    pub velocity_type: Option<SpeedType>,
}

impl AdsbState {
    pub fn new() -> Self {
        Self {
            cpr_position: None,
            latitude: None,
            longitude: None,
            altitude: None,
            altitude_source: None,
            callsign: None,
            wake_vortex_cat: None,
            gnss_vertical_velocity: None,
            barometer_vertical_velocity: None,
            preferred_vertical_velocity_source: None,
            heading: None,
            speed: None,
            speed_category: None,
            velocity_type: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CprPosition {
    pub cpr_lat: f64,
    pub cpr_lon: f64,
    pub cpr_format: u8,
}
