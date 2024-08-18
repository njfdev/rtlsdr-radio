import { Station, StationDetails, StationSortOption } from "./types";

const stationsStorageName = "savedStations";

export async function saveStation(station: StationDetails) {
  if (isStationSaved(station)) return;

  let oldStations = localStorage.getItem(stationsStorageName);

  if (!oldStations) {
    oldStations = "[]";
  }

  let parsedStations: [StationDetails] = JSON.parse(oldStations);

  parsedStations.push(station);

  await localStorage.setItem(
    stationsStorageName,
    JSON.stringify(parsedStations)
  );
  dispatchEvent(new Event("saved_stations"));
}

export function isStationSaved(station: Station) {
  let stations = localStorage.getItem(stationsStorageName);

  if (!stations) return false;

  let parsedStations: [StationDetails] = JSON.parse(stations);

  let filteredStations = parsedStations.filter((e_station) => {
    return (
      e_station.type == station.type &&
      e_station.frequency == station.frequency &&
      (station.channel ? e_station.channel == station.channel : true)
    );
  });

  return filteredStations.length > 0 ? true : false;
}

export async function removeStation(station: StationDetails) {
  if (!isStationSaved(station)) return;

  let stations: [StationDetails] = JSON.parse(
    localStorage.getItem(stationsStorageName)!
  );

  let updatedStations = stations.filter((e_station) => {
    return !(
      e_station.type == station.type &&
      e_station.frequency == station.frequency &&
      (station.channel ? e_station.channel == station.channel : true)
    );
  });

  await localStorage.setItem(
    stationsStorageName,
    JSON.stringify(updatedStations)
  );
  dispatchEvent(new Event("saved_stations"));
}

export async function getSavedStations(): Promise<StationDetails[]> {
  let stations = localStorage.getItem(stationsStorageName);

  if (!stations) return [];

  let parsedStations: [StationDetails] = JSON.parse(stations);

  return parsedStations;
}

export async function updateStation(
  oldStation: StationDetails,
  newStation: StationDetails
) {
  if (!isStationSaved(oldStation)) await saveStation(newStation);

  let stations: [StationDetails] = JSON.parse(
    localStorage.getItem(stationsStorageName)!
  );

  let filteredStations = stations.filter((station) => {
    return (
      station.type == oldStation.type &&
      station.frequency == oldStation.frequency &&
      (oldStation.channel ? station.channel == oldStation.channel : true)
    );
  });

  stations[stations.indexOf(filteredStations[0])] = newStation;

  await localStorage.setItem(stationsStorageName, JSON.stringify(stations));
  dispatchEvent(new Event("saved_stations"));
}

export function areStationsEqual(
  stationA: StationDetails | Station | undefined,
  stationB: StationDetails | Station | undefined
) {
  if (!stationA || !stationB) {
    return stationA == stationB;
  }

  return (
    stationA.type == stationB.type &&
    stationA.frequency == stationB.frequency &&
    ((!stationA.channel && !stationB.channel) ||
      stationA.channel == stationB.channel)
  );
}

export function stationSortComparison(
  a: StationDetails,
  b: StationDetails,
  sortType: StationSortOption
) {
  switch (sortType) {
    case StationSortOption.Favorites:
      if (a.isFavorite && !b.isFavorite) return -1;
      if (!a.isFavorite && b.isFavorite) return 1;
      return a.frequency - b.frequency;
    // @ts-expect-error
    case StationSortOption.AlphaDes:
      let tmp_b = b;
      b = a;
      a = tmp_b;
    // fallthrough
    case StationSortOption.AlphaAsc:
      if (a.title < b.title) return -1;
      if (a.title > b.title) return 1;
      return 0;
    case StationSortOption.FreqAsc:
      return a.frequency - b.frequency;
    case StationSortOption.FreqDes:
      return b.frequency - a.frequency;
    case StationSortOption.StationType:
      return a.type - b.type;
  }
}
