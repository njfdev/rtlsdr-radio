"use client";

import { Dispatch, SetStateAction, useEffect, useState } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Loader2 } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Separator } from "../ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import {
  areStationsEqual,
  isStationSaved,
  removeStation,
  saveStation,
} from "@/lib/stationsStorage";
import {
  HdRadioState,
  Station,
  StationDetails,
  StationType,
} from "@/lib/types";
const appWindow = getCurrentWebviewWindow();

enum Nrsc5Status {
  Stopped = "stopped",
  Starting = "starting",
  SdrFound = "sdr-found",
  Synced = "synchronized",
  SyncLost = "synchronization_lost",
}

interface StreamDetails {
  songTitle?: string;
  songArtist?: string;
  stationName?: string;
  slogan?: string;
  message?: string;
  audioBitRate?: number;
  bitErrorRate?: number;
  frequency?: number;
  channel?: number;
}

export default function Nrsc5Controls({
  currentStation,
  setCurrentStation,
  requestedStation,
  setRequestedStation,
}: {
  currentStation: Station | undefined;
  setCurrentStation: Dispatch<SetStateAction<Station | undefined>>;
  requestedStation: Station | null | undefined;
  setRequestedStation: Dispatch<SetStateAction<Station | null | undefined>>;
}) {
  const [freq, setFreq] = useState<number>(101.5);
  const [channel, setChannel] = useState<number>(1);

  const [nrsc5Status, setNrsc5Status] = useState(Nrsc5Status.Stopped);
  const [streamDetails, setStreamDetails] = useState<StreamDetails>({});

  const [isSaved, setIsSaved] = useState(
    isStationSaved({
      type: StationType.HDRadio,
      frequency: streamDetails.frequency!,
      channel: streamDetails.channel,
    })
  );

  useEffect(() => {
    if (currentStation) {
      setIsSaved(isStationSaved(currentStation));
    }
  });

  useEffect(() => {
    (async () => {
      if (
        requestedStation &&
        requestedStation.type == StationType.HDRadio &&
        !areStationsEqual(requestedStation, currentStation)
      ) {
        await setFreq(requestedStation.frequency);
        await setChannel(requestedStation.channel!);
        start_nrsc5();
      }
    })();
  });

  const hdRadioChannel = new Channel<HdRadioState>();
  hdRadioChannel.onmessage = (message) => {
    console.log(message);
  };

  const start_nrsc5 = () => {
    setNrsc5Status(Nrsc5Status.Starting);
    setStreamDetails({ frequency: freq, channel });
    invoke<string>("start_nrsc5", {
      fmFreq: freq.toString(),
      channel: (channel - 1).toString(),
      hdRadioChannel,
    })
      .then((_result) => {
        setStreamDetails((old) => ({
          ...old,
          audioBitRate: 0,
          bitErrorRate: 0,
        }));
        setCurrentStation({
          type: StationType.HDRadio,
          frequency: freq,
          channel,
        });
      })
      .catch(console.error);
  };
  const stop_nrsc5 = async () => {
    await invoke<string>("stop_nrsc5", {});
    setCurrentStation(undefined);
  };

  useEffect(() => {
    if (!requestedStation && nrsc5Status != Nrsc5Status.Stopped) {
      stop_nrsc5();
    }
  });

  appWindow.listen("nrsc5_status", (event: { payload: string }) => {
    setNrsc5Status(
      Nrsc5Status[
        Object.keys(Nrsc5Status)[
          Object.values(Nrsc5Status).indexOf(event.payload as Nrsc5Status)
        ] as keyof typeof Nrsc5Status
      ]
    );
  });

  appWindow.listen("nrsc5_title", (event: { payload: string }) => {
    setStreamDetails((old) => ({ ...old, songTitle: event.payload }));
  });
  appWindow.listen("nrsc5_artist", (event: { payload: string }) => {
    setStreamDetails((old) => ({ ...old, songArtist: event.payload }));
  });
  appWindow.listen("nrsc5_br", (event: { payload: string }) => {
    setStreamDetails((old) => ({
      ...old,
      audioBitRate: parseFloat(event.payload),
    }));
  });
  appWindow.listen("nrsc5_station", (event: { payload: string }) => {
    setStreamDetails((old) => ({ ...old, stationName: event.payload }));
  });
  appWindow.listen("nrsc5_slogan", (event: { payload: string }) => {
    setStreamDetails((old) => ({ ...old, slogan: event.payload }));
  });
  appWindow.listen("nrsc5_message", (event: { payload: string }) => {
    setStreamDetails((old) => ({ ...old, message: event.payload }));
  });
  appWindow.listen("nrsc5_ber", (event: { payload: string }) => {
    setStreamDetails((old) => ({
      ...old,
      bitErrorRate: parseFloat(event.payload),
    }));
  });

  return (
    <div className="flex flex-col-reverse xl:flex-row-reverse xl:w-[48rem] w-[24rem] gap-4">
      <div className="flex flex-col gap-2 items-center grow basis-0 justify-center align-middle w-full">
        <div className="grid grid-cols-1 grid-rows-1 relative w-full">
          <Tabs
            defaultValue="radioInfo"
            className={`w-full ${
              nrsc5Status != Nrsc5Status.Synced ? "opacity-75" : ""
            }`}
            style={{ gridColumn: 0, gridRow: 0 }}
          >
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger
                disabled={nrsc5Status != Nrsc5Status.Synced}
                value="radioInfo"
              >
                Radio Info
              </TabsTrigger>
              <TabsTrigger
                disabled={nrsc5Status != Nrsc5Status.Synced}
                value="stationInfo"
              >
                Station Info
              </TabsTrigger>
            </TabsList>
            {nrsc5Status != Nrsc5Status.Stopped ? (
              <>
                <TabsContent value="radioInfo">
                  <Card
                    className={`
                  ${
                    nrsc5Status == Nrsc5Status.SyncLost ? "*:blur-sm" : ""
                  } *:transition-all`}
                  >
                    <CardHeader>
                      <CardTitle>
                        {streamDetails.songTitle &&
                        streamDetails.songTitle.trim().length > 0 ? (
                          <>{streamDetails.songTitle}</>
                        ) : (
                          <Skeleton className="h-6 max-w-52" />
                        )}
                      </CardTitle>
                      <CardDescription className="grid grid-cols-2">
                        {streamDetails.songArtist &&
                        streamDetails.songArtist.trim().length > 0 ? (
                          <>{streamDetails.songArtist}</>
                        ) : (
                          <Skeleton className="h-4 max-w-36" />
                        )}
                      </CardDescription>
                    </CardHeader>
                    <CardContent>
                      {streamDetails.audioBitRate != undefined && (
                        <Badge
                          variant="outline"
                          className={`before:content-[''] before:inline-block before:w-2 before:h-2 before:${
                            streamDetails.audioBitRate > 64
                              ? "bg-green-500"
                              : streamDetails.audioBitRate > 32
                              ? "bg-yellow-500"
                              : "bg-red-500"
                          } before:rounded-full before:mr-2`}
                        >
                          {streamDetails.audioBitRate}kbps
                        </Badge>
                      )}
                      {streamDetails.bitErrorRate != undefined && (
                        <Badge
                          variant="outline"
                          className={`before:content-[''] before:inline-block before:w-2 before:h-2 before:${
                            streamDetails.bitErrorRate < 0.0075
                              ? "bg-green-500"
                              : streamDetails.bitErrorRate < 0.05
                              ? "bg-yellow-500"
                              : "bg-red-500"
                          } before:rounded-full before:mr-2`}
                        >
                          {Math.floor(streamDetails.bitErrorRate * 100_00) /
                            100}
                          % BER
                        </Badge>
                      )}
                    </CardContent>
                  </Card>
                </TabsContent>
                <TabsContent value="stationInfo">
                  <Card>
                    <CardHeader>
                      <CardTitle>
                        {streamDetails.stationName &&
                        streamDetails.stationName.trim().length > 0 ? (
                          <>{streamDetails.stationName}</>
                        ) : (
                          <Skeleton className="h-6 max-w-52" />
                        )}
                      </CardTitle>
                      <CardDescription>
                        {streamDetails.slogan &&
                        streamDetails.slogan.trim().length > 0 ? (
                          <>{streamDetails.slogan}</>
                        ) : (
                          <Skeleton className="h-4 max-w-36" />
                        )}
                      </CardDescription>
                    </CardHeader>
                    {streamDetails.message &&
                      streamDetails.message.trim().length > 0 && (
                        <CardContent>
                          <h2 className="text-lg font-bold">Station Message</h2>
                          <span className="text-sm">
                            {streamDetails.message}
                          </span>
                        </CardContent>
                      )}
                  </Card>
                </TabsContent>
              </>
            ) : (
              <Card>
                <CardHeader />
                <CardContent>
                  <span className="w-full text-center">SDR Not Running</span>
                </CardContent>
                <CardFooter />
              </Card>
            )}
          </Tabs>
          {nrsc5Status == Nrsc5Status.SyncLost && (
            <div
              className="absolute w-full h-full flex align-middle items-center justify-center"
              style={{ gridColumn: 0, gridRow: 0 }}
            >
              <span className="text-red-500 bg-black rounded-sm border-stone-900 border-[2px] px-4 py-2">
                Synchronization Lost
              </span>
            </div>
          )}
        </div>
        {(nrsc5Status == Nrsc5Status.Synced ||
          nrsc5Status == Nrsc5Status.SyncLost) && (
          <Button
            className="w-full"
            variant={isSaved ? "secondary" : "default"}
            onClick={async () => {
              const stationData: StationDetails = {
                type: StationType.HDRadio,
                title:
                  streamDetails.stationName ||
                  `HD Radio: ${streamDetails.frequency!}`,
                frequency: streamDetails.frequency!,
                channel: streamDetails.channel!,
                isFavorite: false,
              };

              if (isSaved) {
                await removeStation(stationData);
                setIsSaved(false);
              } else {
                await saveStation(stationData);
                setIsSaved(true);
              }
            }}
          >
            {isSaved ? "Remove " : "Save "} Station
          </Button>
        )}
      </div>
      <Separator orientation="vertical" />
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (nrsc5Status == Nrsc5Status.Stopped) {
            setRequestedStation({
              type: StationType.HDRadio,
              frequency: freq,
              channel,
            });
          }
        }}
        className="grid gap-2 grow basis-0 h-max"
      >
        <div className="grid w-full gap-1.5">
          <Label htmlFor="fm_freq_slider" className="flex">
            HD Radio Station
          </Label>
          <Input
            id="fm_freq_slider"
            type="number"
            step={0.2}
            min={88.1}
            max={107.9}
            value={freq}
            onChange={(e) => setFreq(parseFloat(e.target.value))}
          />
        </div>
        <div className="grid w-full gap-1.5">
          <div>
            <Label htmlFor="fm_freq_slider">Station Channel</Label>
          </div>
          <Input
            type="number"
            step={1}
            min={1}
            max={4}
            placeholder="#"
            value={channel}
            onChange={(e) => setChannel(parseInt(e.target.value))}
          />
        </div>
        <Button
          onClick={() => {
            if (nrsc5Status != Nrsc5Status.Starting) {
              setRequestedStation(undefined);
            }
          }}
          disabled={nrsc5Status == Nrsc5Status.Starting}
        >
          {nrsc5Status == Nrsc5Status.Starting && (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          )}
          {nrsc5Status == Nrsc5Status.Stopped
            ? "Start nrsc5"
            : nrsc5Status == Nrsc5Status.Starting
            ? "Starting..."
            : "Stop nrsc5"}
        </Button>
      </form>
    </div>
  );
}
