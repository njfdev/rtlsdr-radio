export enum StationSavingState {
  Idle,
  Saving,
  Removing,
}

export enum StationType {
  HDRadio = 0,
  FMRadio = 1,
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
