"use client";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import {
  areStationsEqual,
  isStationSaved,
  removeStation,
  saveStation,
} from "@/lib/stationsStorage";
import {
  RbdsData,
  srStorageName,
  Station,
  StationDetails,
  StationType,
  StreamSettings,
  StreamType,
  volumeStorageName,
} from "@/lib/types";
import { invoke } from "@tauri-apps/api";
import { appWindow } from "@tauri-apps/api/window";
import { Loader2 } from "lucide-react";
import { Dispatch, SetStateAction, useEffect, useState } from "react";
import { TabsContent, TabsTrigger, Tabs, TabsList } from "./ui/tabs";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "./ui/card";
import { Skeleton } from "./ui/skeleton";
import { Badge } from "./ui/badge";

enum RtlSdrStatus {
  Stopped = "stopped",
  Starting = "starting",
  Pausing = "pausing",
  Running = "running",
}

export default function RtlSdrControls({
  currentStation,
  setCurrentStation,
  requestedStation,
  setRequestedStation,
  isInUse,
  setIsInUse,
  streamType,
}: {
  currentStation: Station | undefined;
  setCurrentStation: Dispatch<SetStateAction<Station | undefined>>;
  requestedStation: Station | undefined;
  setRequestedStation: Dispatch<SetStateAction<Station | undefined>>;
  isInUse: boolean;
  setIsInUse: Dispatch<SetStateAction<boolean>>;
  streamType: StreamType;
}) {
  let currentStationType =
    streamType == StreamType.FM ? StationType.FMRadio : StationType.AMRadio;

  const [status, setStatus] = useState(RtlSdrStatus.Stopped);
  const [streamSettings, setStreamSettings] = useState<StreamSettings>({
    freq: streamType == StreamType.FM ? 101.5 : 850,
    volume: parseFloat(localStorage.getItem(volumeStorageName) || "0.5"),
    gain: streamType == StreamType.FM ? 1.0 : 2000.0,
    sample_rate: parseFloat(localStorage.getItem(srStorageName) || "48000.0"),
    stream_type: streamType,
  });
  const [isProcessingRequest, setIsProcessingRequest] = useState(false);
  const [error, setError] = useState("");
  const [rbdsData, setRbdsData] = useState<RbdsData>({});

  const [isSaved, setIsSaved] = useState(
    isStationSaved({
      type: currentStationType,
      frequency: streamSettings.freq,
    })
  );

  useEffect(() => {
    localStorage.setItem(volumeStorageName, streamSettings.volume.toString());
    localStorage.setItem(srStorageName, streamSettings.sample_rate.toString());
  });

  useEffect(() => {
    if (currentStation) {
      setIsSaved(isStationSaved(currentStation));
    }
  }, [currentStation]);

  useEffect(() => {
    if (isProcessingRequest) return;
    (async () => {
      if (
        requestedStation &&
        requestedStation.type == currentStationType &&
        !areStationsEqual(requestedStation, currentStation)
      ) {
        await setIsProcessingRequest(true);
        if (isInUse) {
          console.log("stopping stream");
          await stop_stream();
        }

        setIsInUse(true);
        await setStreamSettings((old) => ({
          ...old,
          freq: requestedStation.frequency,
        }));
        await start_stream();
        setIsProcessingRequest(false);
      }
    })();
  });

  useEffect(() => {
    if (
      (!requestedStation && status != RtlSdrStatus.Stopped) ||
      (requestedStation && requestedStation?.type != currentStationType)
    ) {
      stop_stream();
    }
  }, [requestedStation, status]);

  const start_stream = async () => {
    setIsInUse(true);
    setStatus(RtlSdrStatus.Starting);
    await invoke<string>("start_stream", {
      streamSettings,
    });
    setError("");
    setCurrentStation({
      type: currentStationType,
      frequency: streamSettings.freq,
      channel: undefined,
    });
  };
  const stop_stream = async () => {
    setStatus(RtlSdrStatus.Pausing);
    setRbdsData({});
    await invoke<string>("stop_stream", {});
    await setIsInUse(false);
    setCurrentStation(undefined);
  };

  appWindow.listen("rtlsdr_status", (event: { payload: string }) => {
    let fixed_payload = event.payload.replace("fm_", "").replace("am_", "");

    setStatus(
      RtlSdrStatus[
        Object.keys(RtlSdrStatus)[
          Object.values(RtlSdrStatus).indexOf(fixed_payload as RtlSdrStatus)
        ] as keyof typeof RtlSdrStatus
      ]
    );
  });

  appWindow.listen("rtlsdr_err", async (event: { payload: string }) => {
    setError(event.payload);
    await setCurrentStation(undefined);
    await setRequestedStation(undefined);
    setIsInUse(false);
  });

  appWindow.listen("rtlsdr_rbds", async (event: { payload: string }) => {
    let parsed_data = JSON.parse(event.payload);
    setRbdsData((old) => ({ ...old, ...parsed_data }));
  });

  let firstRun = true;
  useEffect(() => {
    return () => {
      if (firstRun) {
        firstRun = false;
      } else {
        (async () => {
          await stop_stream();
        })();
      }
    };
  }, []);

  return (
    <div className="flex xl:flex-row flex-col gap-8 xl:w-[48rem] w-[24rem]">
      <form
        className="grid gap-3 max-w-[24rem] w-[24rem] h-max"
        onSubmit={(e) => {
          e.preventDefault();
          if (status == RtlSdrStatus.Stopped) {
            setRequestedStation({
              type: currentStationType,
              frequency: streamSettings.freq,
            });
          }
        }}
      >
        {streamType == StreamType.AM && (
          <span className="text-center text-amber-300">
            RTL-SDRs often struggle with AM radio signals below 24 MHz (without
            an upconvertor), resulting in significant static. Reception quality
            will likely be poor.
          </span>
        )}
        <div className="grid w-full gap-1.5">
          <Label htmlFor="freq_slider">{streamType.valueOf()} Station</Label>
          <Input
            type="number"
            step={streamType == StreamType.FM ? 0.2 : 10}
            min={streamType == StreamType.FM ? 88.1 : 540}
            max={streamType == StreamType.FM ? 107.9 : 1700}
            placeholder="#"
            value={streamSettings.freq}
            onChange={(e) =>
              setStreamSettings((old) => ({
                ...old,
                freq: parseFloat(e.target.value),
              }))
            }
          />
        </div>
        <div className="grid w-full gap-1.5">
          <Label htmlFor="audio_sr">Audio Sample Rate</Label>
          <Input
            type="number"
            id="audio_sr"
            step={1}
            min={44100.0}
            max={192000.0}
            placeholder="#"
            value={streamSettings.sample_rate}
            onChange={(e) =>
              setStreamSettings((old) => ({
                ...old,
                sample_rate: parseFloat(e.target.value),
              }))
            }
          />
        </div>
        <div className="grid w-full gap-1.5">
          <Label htmlFor="volume_slider">Volume</Label>
          <Slider
            min={0.0}
            max={1.0}
            step={0.01}
            value={[streamSettings.volume]}
            id="volume_slider"
            className="py-[2px]"
            onValueChange={(values) => {
              setStreamSettings((old) => ({ ...old, volume: values[0] }));
            }}
          />
        </div>
        <Button
          onClick={() => {
            if (status == RtlSdrStatus.Running) {
              setRequestedStation(undefined);
            }
          }}
          disabled={
            status == RtlSdrStatus.Starting || status == RtlSdrStatus.Pausing
          }
        >
          {status == RtlSdrStatus.Running ? (
            `Stop ${streamType.valueOf()} Stream`
          ) : status == RtlSdrStatus.Starting ? (
            <>
              <Loader2 className="animate-spin mr-2" /> Starting...
            </>
          ) : status == RtlSdrStatus.Pausing ? (
            <>
              <Loader2 className="animate-spin mr-2" /> Stopping...
            </>
          ) : (
            `Start ${streamType.valueOf()} Stream`
          )}
        </Button>
        {error.length > 0 && (
          <span className="text-center text-red-400">{error}</span>
        )}
        {status == RtlSdrStatus.Running && (
          <Button
            className="w-full"
            variant={isSaved ? "secondary" : "default"}
            onClick={async () => {
              let stationData: StationDetails = {
                type: currentStationType,
                title: `${streamType.valueOf()} Radio: ${streamSettings.freq}`,
                frequency: streamSettings.freq,
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
      </form>
      {streamType == StreamType.FM && (
        <>
          <Tabs
            defaultValue="radioInfo"
            className={`transition-all max-w-[24rem] w-[24rem] ${
              status == RtlSdrStatus.Stopped ? "opacity-75" : ""
            }`}
            style={{ gridColumn: 0, gridRow: 0 }}
          >
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger
                disabled={status == RtlSdrStatus.Stopped}
                value="radioInfo"
              >
                Radio Info
              </TabsTrigger>
              <TabsTrigger
                disabled={status == RtlSdrStatus.Stopped}
                value="advancedInfo"
              >
                Advanced Info
              </TabsTrigger>
            </TabsList>
            {status != RtlSdrStatus.Stopped ? (
              <>
                <TabsContent value="radioInfo">
                  <Card>
                    <CardHeader>
                      <CardTitle className="whitespace-pre-wrap">
                        {rbdsData.radio_text &&
                        rbdsData.radio_text.trimEnd() ? (
                          <>{rbdsData.radio_text.trimEnd()}</>
                        ) : (
                          <Skeleton className="h-6 max-w-52" />
                        )}
                      </CardTitle>
                    </CardHeader>
                    <CardContent>
                      {rbdsData.program_type && (
                        <Badge variant="outline">{rbdsData.program_type}</Badge>
                      )}
                      {rbdsData.ms_flag != undefined && (
                        <Badge variant="outline">
                          {rbdsData.ms_flag ? "Music" : "Speech"}
                        </Badge>
                      )}
                    </CardContent>
                  </Card>
                </TabsContent>
                <TabsContent value="advancedInfo">
                  <Card>
                    <CardHeader>
                      <CardTitle>Advanced Info</CardTitle>
                      <CardDescription>
                        This is all the other RBDS/RDS data that RTL-SDR Radio
                        can decode.
                      </CardDescription>
                    </CardHeader>
                    <CardContent className="flex flex-col gap-2">
                      <span>
                        <b>Program Service Name:</b>{" "}
                        <span className="font-mono">
                          {rbdsData.program_service_name || "loading..."}
                          {rbdsData.pty_name ? ` ${rbdsData.pty_name}` : ""}
                        </span>
                      </span>
                      <span className="font-bold">
                        Decoder Identification Info
                      </span>
                      <div className="indent-4 flex flex-col -mt-2">
                        <span>
                          <b>Channels:</b>{" "}
                          {rbdsData.di_is_stereo != undefined
                            ? rbdsData.di_is_stereo
                              ? "Stereo"
                              : "Mono"
                            : "Loading..."}
                        </span>
                        <span>
                          <b>Binaural Audio:</b>{" "}
                          {rbdsData.di_is_binaural != undefined
                            ? rbdsData.di_is_binaural
                              ? "Yes"
                              : "No"
                            : "Loading..."}
                        </span>
                        <span>
                          <b>Compression:</b>{" "}
                          {rbdsData.di_is_compressed != undefined
                            ? rbdsData.di_is_compressed
                              ? "Yes"
                              : "No"
                            : "Loading..."}
                        </span>
                        <span>
                          <b>PTY Type:</b>{" "}
                          {rbdsData.di_is_pty_dynamic != undefined
                            ? rbdsData.di_is_pty_dynamic
                              ? "Dynamic"
                              : "Static"
                            : "Loading..."}
                        </span>
                      </div>
                      <span>
                        <b>Traffic Info:</b>{" "}
                        {(() => {
                          if (
                            rbdsData.ta != undefined &&
                            rbdsData.tp != undefined
                          ) {
                            switch (true) {
                              case rbdsData.tp == false && rbdsData.ta == false:
                                return "This radio station does not carry traffic announcements.";
                              case rbdsData.tp == false && rbdsData.ta == true:
                                return "This radio station does not carry traffic announcements, but it carries EON information about a station that does.";
                              case rbdsData.tp == false && rbdsData.ta == false:
                                return "This radio station carries traffic announcements, but none are ongoing presently.";
                              case rbdsData.tp == false && rbdsData.ta == false:
                                return "There is an ongoing traffic announcement.";
                            }
                          }

                          return "Loading...";
                        })()}
                      </span>
                    </CardContent>
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
        </>
      )}
    </div>
  );
}
