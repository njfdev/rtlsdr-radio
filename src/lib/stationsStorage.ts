import { Station, StationType } from "./types";

const stationsStorageName = "savedStations";

export async function saveStation(station: Station) {
  if (isStationSaved(station.type, station.frequency, station.channel)) return;

  let oldStations = localStorage.getItem(stationsStorageName);

  if (!oldStations) {
    oldStations = "[]";
  }

  let parsedStations: [Station] = JSON.parse(oldStations);

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

  let parsedStations: [Station] = JSON.parse(stations);

  let filteredStations = parsedStations.filter((station) => {
    return (
      station.type == type &&
      station.frequency == frequency &&
      (channel ? station.channel == channel : true)
    );
  });

  return filteredStations.length > 0 ? true : false;
}

export async function removeStation(station: Station) {
  if (!isStationSaved(station.type, station.frequency, station.channel)) return;

  let stations: [Station] = JSON.parse(
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

export async function getSavedStations(): Promise<Station[]> {
  let stations = localStorage.getItem(stationsStorageName);

  if (!stations) return [];

  let parsedStations: [Station] = JSON.parse(stations);

  return parsedStations;
}

export async function updateStation(oldStation: Station, newStation: Station) {
  if (
    !isStationSaved(oldStation.type, oldStation.frequency, oldStation.channel)
  )
    await saveStation(newStation);

  let stations: [Station] = JSON.parse(
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

export function areStationsEqual(stationA: Station, stationB: Station) {
  return (
    stationA.type == stationB.type &&
    stationA.frequency == stationB.frequency &&
    stationA.channel == stationB.channel
  );
}
