import { Station, StationDetails, StationType } from "./types";

const stationsStorageName = "savedStations";

export async function saveStation(station: StationDetails) {
  if (isStationSaved(station.type, station.frequency, station.channel)) return;

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
}

export function isStationSaved(
  type: StationType,
  frequency: number,
  channel?: number
) {
  let stations = localStorage.getItem(stationsStorageName);

  if (!stations) return false;

  let parsedStations: [StationDetails] = JSON.parse(stations);

  let filteredStations = parsedStations.filter((station) => {
    return (
      station.type == type &&
      station.frequency == frequency &&
      (channel ? station.channel == channel : true)
    );
  });

  return filteredStations.length > 0 ? true : false;
}

export async function removeStation(station: StationDetails) {
  if (!isStationSaved(station.type, station.frequency, station.channel)) return;

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
  if (
    !isStationSaved(oldStation.type, oldStation.frequency, oldStation.channel)
  )
    await saveStation(newStation);

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
    stationA.channel == stationB.channel
  );
}
