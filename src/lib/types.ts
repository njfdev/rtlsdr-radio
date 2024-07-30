export const srStorageName = "fm_radio_sample_rate";
export const volumeStorageName = "fm_radio_volume";

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

export interface StreamSettings {
  freq: number;
  volume: number;
  gain: number;
  sample_rate: number;
  stream_type: StreamType;
}

export interface RbdsData {
  programType?: string;
}
