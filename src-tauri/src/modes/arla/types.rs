use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const API_VERSION: u8 = 0;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum RegistrantType {
    Individual,
    Partnership,
    #[serde(rename(deserialize = "CORPORATATION"))]
    Corporation,
    CoOwned,
    Government,
    Llc,
    NonCitizenCorporation,
    NonCitizenCoOwned,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum Region {
    Eastern,
    Southwestern,
    Central,
    WesternPacific,
    Alaskan,
    Southern,
    European,
    GreatLakes,
    NewEngland,
    NorthwestMountain,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum AircraftType {
    Glider,
    Balloon,
    Blimp,
    FixedWingSingleEngine,
    FixedWingMultiEngine,
    Rotorcraft,
    WeightShiftControl,
    PoweredParachute,
    Gyroplane,
    HybridLift,
    Other,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum EngineType {
    None,
    Reciprocating,
    TurboProp,
    TurboShaft,
    TurboJet,
    TurboFan,
    Ramjet,
    TwoCycle,
    FourCycle,
    Unknown,
    Electric,
    Rotary,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum AircraftCategory {
    Land,
    Sea,
    Amphibian,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all(deserialize = "SCREAMING_SNAKE_CASE"))]
pub enum BuilderCertification {
    TypeCertified,
    NotTypeCertified,
    LightSport,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum AircraftWeightClass {
    #[serde(rename(deserialize = "CLASS_1"))]
    Class1,
    #[serde(rename(deserialize = "CLASS_2"))]
    Class2,
    #[serde(rename(deserialize = "CLASS_3"))]
    Class3,
    #[serde(rename(deserialize = "CLASS_4"))]
    Class4,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AircraftModelObject {
    pub code: String,
    pub mfr: String,
    pub model: String,
    pub aircraft_type: AircraftType,
    pub engine_type: EngineType,
    pub aircraft_cat_code: AircraftCategory,
    pub builder_cert_code: BuilderCertification,
    pub engine_count: u8,
    pub seat_count: u16,
    pub weight_class: AircraftWeightClass,
    pub avg_cruising_speed: Option<u16>,
    pub tc_data_sheet: Option<String>,
    pub tc_data_holder: Option<String>,
}
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct EngineModelObject {
    pub code: String,
    pub mfr: String,
    pub model: String,
    pub r#type: EngineType,
    pub horsepower: Option<u32>,
    pub lbs_of_thrust: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RegistrationObject {
    pub n_number: String,
    pub serial_number: String,
    pub mft_mdl_code: String,
    pub eng_mfr_mdl: String,
    pub year_mfr: Option<u16>,
    pub registrant_type: Option<RegistrantType>,
    pub registrant_name: Option<String>,
    pub registrant_street: Option<String>,
    pub registrant_street2: Option<String>,
    pub registrant_city: Option<String>,
    pub registrant_state: Option<String>,
    pub registrant_zip_code: Option<String>,
    pub registrant_region: Option<Region>,
    pub registrant_county_code: Option<u16>,
    pub registrant_country_code: Option<String>,
    pub last_action_date: DateTime<Utc>,
    pub cert_issue_date: Option<DateTime<Utc>>,
    pub cert_details: String,
    pub aircraft_type: AircraftType,
    pub engine_type: EngineType,
    pub status_code: String,
    pub mode_s_code: u32,
    pub fractional_ownership: bool,
    pub air_worth_date: Option<DateTime<Utc>>,
    pub other_registrant_name_1: Option<String>,
    pub other_registrant_name_2: Option<String>,
    pub other_registrant_name_3: Option<String>,
    pub other_registrant_name_4: Option<String>,
    pub other_registrant_name_5: Option<String>,
    pub expiration_date: DateTime<Utc>,
    pub unique_id: u32,
    pub kit_mfr: Option<String>,
    pub kit_model: Option<String>,
    pub mode_s_code_hex: String,
    pub aircraft_info: AircraftModelObject,
    pub engine_info: EngineModelObject,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ApiAircraftRegistrationLookup {
    KnownRegistration { registration: RegistrationObject },
    LookupError { error: String, message: String },
}
