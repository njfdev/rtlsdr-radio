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
}

export interface RbdsData {
  pi?: number;
  program_type?: string;
  pty_name?: string;
  program_service_name?: string;
  radio_text?: string;
  ms_flag?: boolean;
  di_is_stereo?: boolean;
  di_is_binaural?: boolean;
  di_is_compressed?: boolean;
  di_is_pty_dynamic?: boolean;
  ta?: boolean;
  tp?: boolean;
}

export interface AdsbDecodeSettings {
  gain?: number;
}

export interface ModesState {
  aircraft: AircraftState[];
}

export interface AircraftState {
  icaoAddress: number;
  adsbState?: AdsbState;
}

export interface AdsbState {
  altitude?: number;
  altitudeSource?: AltitudeType;
  barometerVerticalVelocity?: number;
  callsign?: string;
  // this is an internal state, so we don't really care about its contents
  cprPosition?: any;
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
