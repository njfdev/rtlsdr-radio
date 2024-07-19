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
