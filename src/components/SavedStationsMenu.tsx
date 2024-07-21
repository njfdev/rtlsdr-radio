import { Dispatch, SetStateAction, useEffect, useState } from "react";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "./ui/card";
import { Station, StationDetails, StationType } from "@/lib/types";
import {
  areStationsEqual,
  getSavedStations,
  removeStation,
  updateStation,
} from "@/lib/stationsStorage";
import { Loader2, RadioTower, Star } from "lucide-react";
import { Button } from "./ui/button";
import { Badge } from "./ui/badge";

export default function SavedStationsMenu({
  setRequestedStation,
  currentStation,
  isStationPlaying,
}: {
  setRequestedStation: Dispatch<SetStateAction<Station | undefined>>;
  currentStation: Station | undefined;
  isStationPlaying: boolean;
}) {
  const [stations, setStations] = useState<undefined | StationDetails[]>(
    undefined
  );
  const [loadingStation, setLoadingStation] = useState<undefined | Station>();

  useEffect(() => {
    if (stations === undefined) {
      (async () => {
        setStations(await getSavedStations());
      })();
    }
  }, [stations]);

  useEffect(() => {
    if (areStationsEqual(currentStation, loadingStation)) {
      setLoadingStation(undefined);
    }
  });

  const updateRequestedStation = async (station: Station | undefined) => {
    await setLoadingStation(station);
    await setRequestedStation(station);
  };

  addEventListener("saved_stations", async () => {
    setStations(await getSavedStations());
  });

  return (
    <>
      <div className="max-w-[24rem] float-right w-full m-2" />
      <div className="max-w-[24rem] right-0 w-full m-2 h-[calc(100vh_-_1rem)] absolute">
        <Card className="h-full overflow-y-scroll">
          <CardHeader>
            <CardTitle>Saved Stations</CardTitle>
          </CardHeader>
          <CardContent className="grid gap-2">
            {stations?.map((station) => (
              <SavedStationCard
                key={`${station.type}-${station.frequency}-${
                  station.channel || 0
                }`}
                station={station}
                isStationPlaying={isStationPlaying}
                currentStation={currentStation}
                loadingStation={loadingStation}
                updateRequestedStation={updateRequestedStation}
              />
            ))}
          </CardContent>
        </Card>
      </div>
    </>
  );
}

function SavedStationCard({
  station,
  isStationPlaying,
  currentStation,
  loadingStation,
  updateRequestedStation,
}: {
  station: StationDetails;
  currentStation: Station | undefined;
  isStationPlaying: boolean;
  loadingStation: Station | undefined;
  updateRequestedStation: (station?: Station) => void;
}) {
  const isCurrentStationPlaying =
    isStationPlaying &&
    currentStation &&
    areStationsEqual(station, currentStation);
  const isLoading = areStationsEqual(loadingStation, station);

  return (
    <Card>
      <CardHeader className="pb-0">
        <div className="flex justify-between align-middle">
          <CardTitle className="text-lg">{station.title}</CardTitle>
          <Star
            size="22px"
            className={`transition-all ${
              station.isFavorite
                ? "fill-yellow-300 stroke-yellow-300"
                : "fill-black"
            }`}
            onClick={() => {
              updateStation(station, {
                ...station,
                isFavorite: !station.isFavorite,
              });
            }}
          />
        </div>
      </CardHeader>
      <CardContent className="flex gap-1 pb-4">
        <Badge
          variant="secondary"
          className={`before:content-[''] before:inline-block before:w-2 before:h-2 ${
            station.type == StationType.HDRadio
              ? "before:bg-purple-500"
              : station.type == StationType.FMRadio
              ? "before:bg-blue-400"
              : "before:bg-gray-400"
          } before:rounded-full before:mr-2`}
        >
          {station.type == StationType.HDRadio
            ? "HD Radio"
            : station.type == StationType.FMRadio
            ? "FM Radio"
            : "Unknown"}
        </Badge>
        <Badge variant="secondary">
          <RadioTower className="-ml-1.5 h-[0.8rem]" />
          {station.frequency}
        </Badge>
        {station.channel && (
          <Badge variant="secondary">Channel: {station.channel}</Badge>
        )}
      </CardContent>
      <CardFooter className="flex gap-2">
        <Button
          onClick={() => {
            if (isCurrentStationPlaying) {
              updateRequestedStation(undefined);
            } else {
              updateRequestedStation({
                type: station.type,
                frequency: station.frequency,
                channel: station.channel,
              });
            }
          }}
          disabled={isLoading}
          variant={isCurrentStationPlaying ? "secondary" : "default"}
          className="grow basis-0"
        >
          {isLoading ? (
            <>
              <Loader2 className="animate-spin mr-2" /> Loading...
            </>
          ) : (
            <>{isCurrentStationPlaying ? "Stop Station" : "Start Station"}</>
          )}
        </Button>
        <Button
          className="grow basis-0"
          variant={"destructive"}
          onClick={() => {
            removeStation(station);
          }}
        >
          Remove
        </Button>
      </CardFooter>
    </Card>
  );
}
