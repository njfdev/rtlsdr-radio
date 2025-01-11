export const srStorageName = "radio_sample_rate";
export const volumeStorageName = "radio_volume";
export const freqStorageName = "radio_freq";

export enum StationSavingState {
  Idle,
  Saving,
  Removing,
}

export enum StationType {
  HDRadio = 0,
  FMRadio,
  AMRadio,
}

export enum StreamType {
  FM = "FM",
  AM = "AM",
  HD = "HD",
}

export enum StationSortOption {
  Favorites = "Favorites",
  AlphaAsc = "A -> Z",
  AlphaDes = "Z -> A",
  FreqAsc = "88.1 -> 107.9",
  FreqDes = "107.9 -> 88.1",
  StationType = "Station Type",
}

export enum AltitudeType {
  Barometer = "Barometer",
  GNSS = "GNSS",
}

export enum SpeedCategory {
  Subsonic = "Subsonic",
  Supersonic = "Supersonic",
}

export enum AirspeedType {
  IAS = "IAS",
  TAS = "TAS",
}

export interface StationDetails {
  type: StationType;
  title: string;
  channel?: number;
  frequency: number;
  isFavorite: boolean;
}

export interface Station {
  type: StationType;
  frequency: number;
  channel?: number;
}

export interface RadioStreamSettings {
  freq: number;
  volume: number;
  gain: number;
  sample_rate: number;
  stream_type: StreamType;
  hd_radio_program?: number | undefined;
}

export interface RbdsData {
  pi?: number | null;
  serviceName?: string | null;
  programType?: string | null;
  radioText?: string | null;
  radioTextAbFlag?: string | null;
  ptyName?: string | null;
  ptyNameAbFlag?: string | null;
  ta?: boolean | null;
  tp?: boolean | null;
  msFlag?: boolean | null;
  decoderInfo: {
    diIsStereo?: boolean | null;
    diIsBinaural?: boolean | null;
    diIsCompressed?: boolean | null;
    diIsPtyDynamic?: boolean | null;
  } | null;
}

export interface HdRadioState {
  program: number;
  title: string;
  artist: string;
  album: string;
  genre: string;
  thumbnail_data?: string | undefined;
  audio_bitrate: number;
  ber: number;
  lot_id: number;
  ports: [number, number][];
  station_info?:
    | {
        name: string;
        country_code: string;
        fcc_id: number;
        slogan: string;
        message: string;
        alert: string;
        location: [number, number];
        altitude: number;
        audio_services: {
          program: number;
          service_type: string;
          is_restricted: boolean;
          sound_experience: string;
        }[];
      }
    | undefined;
}

export interface AdsbDecodeSettings {
  gain?: number;
}

export interface ModesState {
  aircraft: AircraftState[];
}

export interface AircraftState {
  icaoAddress: number;
  icaoDetails?: AircraftIcaoDetails;
  lastMessageTimestamp: {
    secs_since_epoch: number;
    nanos_since_epoch: number;
  };
  adsbState?: AdsbState;
  flightRoute?: FlightRoute;
  registration?: RegistrationObject;
}

export interface AircraftIcaoDetails {
  type: string;
  icao_type: string;
  manufacturer: string;
  mode_s: string;
  registration: string;
  registered_owner_country_iso_name: string;
  registered_owner_country_name: string;
  registered_owner_operator_flag_code?: string;
  registered_owner: string;
  url_photo?: string;
  url_photo_thumbnail?: string;
}

export interface FlightRoute {
  callsign: string;
  callsign_icao?: string;
  callsign_iata?: string;
  airline?: Airline;
  origin: Airport;
  midpoint?: Airport;
  destination: Airport;
}

export interface Airline {
  name: string;
  icao: string;
  iata?: string;
  country: string;
  country_iso: string;
  callsign?: string;
}

export interface Airport {
  country_iso_name: string;
  country_name: string;
  elevation: number;
  iata_code: string;
  icao_code: string;
  latitude: number;
  longitude: number;
  municipality: string;
  name: string;
}

export interface AdsbState {
  altitude?: number;
  altitudeSource?: AltitudeType;
  barometerVerticalVelocity?: number;
  callsign?: string;
  // this is an internal state, so we don't really care about its contents
  cprPosition?: {
    cpr_lat: number;
    cpr_lon: number;
    cpr_format: number;
  };
  gnssVerticalVelocity?: number;
  heading?: number;
  latitude?: number;
  longitude?: number;
  preferredVerticalVelocitySource?: AltitudeType;
  speed?: number;
  speedCategory?: SpeedCategory;
  velocityType?: "GroundSpeed" | { AirSpeed: AirspeedType };
  wakeVortexCat?: string;
}

export enum RegistrantType {
  Individual,
  Partnership,
  Corporation,
  CoOwned,
  Government,
  Llc,
  NonCitizenCorporation,
  NonCitizenCoOwned,
}

export enum Region {
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

export enum AircraftType {
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

export enum EngineType {
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

export enum AircraftCategory {
  Land,
  Sea,
  Amphibian,
}

export enum BuilderCertification {
  TypeCertified,
  NotTypeCertified,
  LightSport,
}

export enum AircraftWeightClass {
  Class1,
  Class2,
  Class3,
  Class4,
}

export interface AircraftModelObject {
  code: string;
  mfr: string;
  model: string;
  aircraft_type: AircraftType;
  engine_type: EngineType;
  aircraft_cat_code: AircraftCategory;
  builder_cert_code: BuilderCertification;
  engine_count: number;
  seat_count: number;
  weight_class: AircraftWeightClass;
  avg_cruising_speed?: number;
  tc_data_sheet?: string;
  tc_data_holder?: string;
}

export interface EngineModelObject {
  code: string;
  mfr: string;
  model: string;
  type: EngineType;
  horsepower?: number;
  lbs_of_thrust?: number;
}

export interface RegistrationObject {
  n_number: string;
  serial_number: string;
  mft_mdl_code: string;
  eng_mfr_mdl: string;
  year_mfr?: number;
  registrant_type?: RegistrantType;
  registrant_name?: string;
  registrant_street?: string;
  registrant_street2?: string;
  registrant_city?: string;
  registrant_state?: string;
  registrant_zip_code?: string;
  registrant_region?: Region;
  registrant_county_code?: number;
  registrant_country_code?: string;
  last_action_date: Date;
  cert_issue_date?: Date;
  cert_details: string;
  aircraft_type: AircraftType;
  engine_type: EngineType;
  status_code: string;
  mode_s_code: number;
  fractional_ownership: boolean;
  air_worth_date?: Date;
  other_registrant_name_1?: string;
  other_registrant_name_2?: string;
  other_registrant_name_3?: string;
  other_registrant_name_4?: string;
  other_registrant_name_5?: string;
  expiration_date: Date;
  unique_id: number;
  kit_mfr?: string;
  kit_model?: string;
  mode_s_code_hex: string;
  aircraft_info: AircraftModelObject;
  engine_info: EngineModelObject;
}

export interface AvailableSdrArgs {
  driver: string;
  label: string;
  manufacturer: string;
  product: string;
  serial: string;
  tuner: string;
}

export interface SDRState {
  args: AvailableSdrArgs;
  dev: "Available" | "Connected" | "InUse";
  functionName?: string;
  statusText?: string;
}
