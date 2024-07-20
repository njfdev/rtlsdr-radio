import { Dispatch, SetStateAction, useEffect, useState } from "react";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "./ui/card";
import { Station, StationType } from "@/lib/types";
import {
  areStationsEqual,
  getSavedStations,
  updateStation,
} from "@/lib/stationsStorage";
import { Star } from "lucide-react";
import { Button } from "./ui/button";

export default function SavedStationsMenu({
  setRequestedStation,
  requestedStation,
  isStationPlaying,
}: {
  setRequestedStation: Dispatch<SetStateAction<Station | undefined>>;
  requestedStation: Station | undefined;
  isStationPlaying: boolean;
}) {
  const [stations, setStations] = useState<undefined | Station[]>(undefined);

  useEffect(() => {
    if (stations === undefined) {
      (async () => {
        setStations(await getSavedStations());
      })();
    }
  }, [stations]);

  return (
    <>
      <div className="max-w-[24rem] float-right w-full m-2" />
      <div className="max-w-[24rem] right-0 w-full m-2 h-[calc(100vh_-_1rem)] absolute">
        <Card className="h-full">
          <CardHeader>
            <CardTitle>Saved Stations</CardTitle>
          </CardHeader>
          <CardContent className="grid gap-2">
            {stations &&
              stations.map((station) => {
                const isCurrentStationPlaying =
                  isStationPlaying &&
                  requestedStation &&
                  areStationsEqual(station, requestedStation);
                console.log(requestedStation);
                return (
                  <Card
                    key={`${station.type}-${station.frequency}-${
                      station.channel || 0
                    }`}
                  >
                    <CardHeader>
                      <div className="flex justify-between align-middle">
                        <CardTitle className="text-lg">
                          {station.title}
                        </CardTitle>
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
                    <CardContent>
                      <p>
                        Type:{" "}
                        {station.type == StationType.HDRadio
                          ? "HD Radio"
                          : station.type == StationType.FMRadio
                          ? "FM Radio"
                          : "Unknown"}
                      </p>
                      <p>Frequency: {station.frequency}</p>
                      {station.channel && <p>Channel: {station.channel!}</p>}
                    </CardContent>
                    <CardFooter>
                      <Button
                        onClick={() => {
                          if (isCurrentStationPlaying) {
                            setRequestedStation(undefined);
                          } else {
                            setRequestedStation(station);
                          }
                        }}
                      >
                        {isCurrentStationPlaying
                          ? "Stop Station"
                          : "Start Station"}
                      </Button>
                    </CardFooter>
                  </Card>
                );
              })}
          </CardContent>
        </Card>
      </div>
    </>
  );
}
