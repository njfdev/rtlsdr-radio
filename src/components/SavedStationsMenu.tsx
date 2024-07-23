import { Dispatch, SetStateAction, useEffect, useState } from "react";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "./ui/card";
import {
  Station,
  StationDetails,
  StationSortOption,
  StationType,
} from "@/lib/types";
import {
  areStationsEqual,
  getSavedStations,
  removeStation,
  stationSortComparison,
  updateStation,
} from "@/lib/stationsStorage";
import { Loader2, RadioTower, Star } from "lucide-react";
import { Button } from "./ui/button";
import { Badge } from "./ui/badge";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";
import { Label } from "./ui/label";
import { Input } from "./ui/input";

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
  const [sortedStations, setSortedStations] = useState<
    undefined | StationDetails[]
  >(stations);
  const [loadingStation, setLoadingStation] = useState<undefined | Station>();
  const [sortOption, setSortOption] = useState(StationSortOption.Favorites);

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

  useEffect(() => {
    if (!stations) {
      setSortedStations(undefined);
    } else {
      let updatedStations = [...stations]?.sort((a, b) => {
        return stationSortComparison(a, b, sortOption);
      });
      setSortedStations(updatedStations);
    }
  }, [stations, sortOption]);

  return (
    <>
      <div className="max-w-[24rem] float-right w-full m-2" />
      <div className="max-w-[24rem] right-0 w-full m-2 h-[calc(100vh_-_1rem)] absolute">
        <Card className="flex flex-col h-full">
          <CardHeader>
            <CardTitle>Saved Stations</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-2 h-full overflow-y-clip">
            <Label htmlFor="saved-stations-sort">Sort</Label>
            <Select
              value={(
                Object.keys(StationSortOption) as Array<
                  keyof typeof StationSortOption
                >
              ).find((key) => StationSortOption[key] === sortOption)}
              onValueChange={(newValue) => {
                setSortOption(
                  StationSortOption[newValue as keyof typeof StationSortOption]
                );
              }}
            >
              <SelectTrigger id="saved-stations-sort">
                <SelectValue placeholder="Sort" />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  {Object.entries(StationSortOption).map(
                    (sortString, index) => {
                      return (
                        <SelectItem
                          key={`saved-stations-sort-option-${sortString[0]}`}
                          value={sortString[0]}
                        >
                          {sortString[1]}
                        </SelectItem>
                      );
                    }
                  )}
                </SelectGroup>
              </SelectContent>
            </Select>
            <div className="h-full flex flex-col gap-2 overflow-y-auto">
              {sortedStations && sortedStations.length > 0 ? (
                sortedStations?.map((station) => (
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
                ))
              ) : (
                <div className="flex items-center align-middle justify-center w-full h-[12rem]">
                  <span className="text-gray-400">No Saved Stations!</span>
                </div>
              )}
            </div>
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
  const [newTitle, setNewTitle] = useState(station.title);

  const isCurrentStationPlaying =
    isStationPlaying &&
    currentStation &&
    areStationsEqual(station, currentStation);
  const isLoading = areStationsEqual(loadingStation, station);

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex justify-between align-middle items-center gap-4">
          <CardTitle>
            <Input
              value={newTitle}
              className="text-lg"
              onChange={(e) => {
                setNewTitle(e.target.value || "");
              }}
              onBlur={() => {
                updateStation(station, {
                  ...station,
                  title: newTitle,
                });
              }}
            />
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
            ? "HD "
            : station.type == StationType.FMRadio
            ? "FM "
            : ""}
          {station.frequency}
        </Badge>
        {station.channel && (
          <Badge variant="secondary">
            <RadioTower className="-ml-1.5 h-[0.8rem]" /> {station.channel}
          </Badge>
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
