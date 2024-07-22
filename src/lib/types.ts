export enum StationSavingState {
  Idle,
  Saving,
  Removing,
}

export enum StationType {
  HDRadio = 0,
  FMRadio = 1,
}

export enum StationSortOption {
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
