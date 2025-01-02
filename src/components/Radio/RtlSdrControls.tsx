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
  freqStorageName,
  RbdsData,
  srStorageName,
  Station,
  StationDetails,
  StationType,
  RadioStreamSettings,
  StreamType,
  volumeStorageName,
  AvailableSdrArgs,
  HdRadioState,
} from "@/lib/types";
import { Channel, invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Globe, Loader2, MusicIcon } from "lucide-react";
import { Dispatch, SetStateAction, useEffect, useState } from "react";
import { TabsContent, TabsTrigger, Tabs, TabsList } from "../ui/tabs";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "../ui/card";
import { Skeleton } from "../ui/skeleton";
import { emit } from "@tauri-apps/api/event";
import {
  getSecondsListenedTo,
  increaseListeningDuration,
} from "@/lib/statsStorage";
import { GlobalState } from "../AppView";
import { Separator } from "../ui/separator";
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from "../ui/hover-card";
const appWindow = getCurrentWebviewWindow();

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
  streamType,
  globalState,
  setGlobalState,
}: {
  currentStation: Station | undefined;
  setCurrentStation: Dispatch<SetStateAction<Station | undefined>>;
  requestedStation: Station | undefined | null;
  setRequestedStation: Dispatch<SetStateAction<Station | undefined | null>>;
  streamType: StreamType;
  globalState: GlobalState;
  setGlobalState: React.Dispatch<React.SetStateAction<GlobalState>>;
}) {
  const currentStationType =
    streamType == StreamType.FM
      ? StationType.FMRadio
      : StreamType.HD
      ? StationType.HDRadio
      : StationType.AMRadio;

  const [status, setStatus] = useState(RtlSdrStatus.Stopped);
  const [streamSettings, setStreamSettings] = useState<RadioStreamSettings>({
    freq: parseFloat(
      localStorage.getItem(streamType.toString() + freqStorageName) ||
        (streamType == StreamType.AM ? "850" : "101.5")
    ),
    volume: parseFloat(
      localStorage.getItem(streamType.toString() + volumeStorageName) || "0.5"
    ),
    gain: streamType == StreamType.AM ? 0.0 : 12.0,
    sample_rate: parseFloat(
      localStorage.getItem(streamType.toString() + srStorageName) || "48000.0"
    ),
    stream_type: streamType,
  });
  const [isProcessingRequest, setIsProcessingRequest] = useState(false);
  const [error, setError] = useState("");
  //const [rbdsData, setRbdsData] = useState<RbdsData>({} as RbdsData);
  const [has10SecondsElapsed, set10SecondsElapsed] = useState(false);
  const [totalSecondsListened, setTotalSecondsListened] = useState(0);
  const [currentSdrArgs, setCurrentSdrArgs] = useState<
    undefined | AvailableSdrArgs
  >(undefined);
  let counterId: NodeJS.Timeout | undefined;

  const [isSaved, setIsSaved] = useState(
    isStationSaved({
      type: currentStationType,
      frequency: streamSettings.freq,
    })
  );

  useEffect(() => {
    (async () => {
      setTotalSecondsListened(await getSecondsListenedTo());
    })();
  }, []);

  const [listeningIntervalId, setListeningIntervalId] = useState<
    NodeJS.Timeout | undefined
  >(undefined);
  const secondsBetweenIncrease = 5;
  useEffect(() => {
    if (listeningIntervalId === undefined && status == RtlSdrStatus.Running) {
      setListeningIntervalId(
        setInterval(async () => {
          if (status == RtlSdrStatus.Running) {
            setTotalSecondsListened(
              await increaseListeningDuration(secondsBetweenIncrease)
            );
          }
        }, secondsBetweenIncrease * 1000)
      );
    } else if (
      listeningIntervalId !== undefined &&
      status != RtlSdrStatus.Running
    ) {
      clearInterval(listeningIntervalId);
      setListeningIntervalId(undefined);
    }
  }, [status, listeningIntervalId]);

  useEffect(() => {
    if (status == RtlSdrStatus.Running) {
      emit("radio_update_settings", streamSettings);
      setIsProcessingRequest(true);
      // If station is changed, reset RDBD date which is station specific
      if (streamSettings.freq != currentStation?.frequency) {
        setGlobalState((old) => ({ ...old, rbdsData: {} as RbdsData }));
      }
      const newStation: Station = {
        type: currentStationType,
        frequency: streamSettings.freq,
        channel: undefined,
      };
      setCurrentStation(newStation);
      setRequestedStation(newStation);
      setIsProcessingRequest(false);
    }
  }, [streamSettings]);

  useEffect(() => {
    localStorage.setItem(
      streamType.toString() + freqStorageName,
      streamSettings.freq.toString()
    );
    localStorage.setItem(
      streamType.toString() + volumeStorageName,
      streamSettings.volume.toString()
    );
    localStorage.setItem(
      streamType.toString() + srStorageName,
      streamSettings.sample_rate.toString()
    );
  }, [streamSettings, streamType]);

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
        !areStationsEqual(requestedStation || undefined, currentStation)
      ) {
        setIsProcessingRequest(true);

        setStreamSettings((old) => ({
          ...old,
          freq: requestedStation.frequency,
        }));
      }
    })();
  });

  useEffect(() => {
    if (
      isProcessingRequest &&
      requestedStation?.type == currentStationType &&
      !areStationsEqual(requestedStation || undefined, currentStation) &&
      status == RtlSdrStatus.Stopped
    ) {
      (async () => {
        await start_stream();
        setIsProcessingRequest(false);
      })();
    }
  }, [isProcessingRequest, streamSettings]);

  useEffect(() => {
    if (requestedStation === null && status != RtlSdrStatus.Stopped) {
      stop_stream();
    }
  }, [requestedStation, status]);

  const rbdsChannel = new Channel<RbdsData>();
  rbdsChannel.onmessage = (message) => {
    setGlobalState((old) => ({ ...old, rbdsData: message }));
    if (currentSdrArgs) {
      updateSdrGlobalState(currentSdrArgs!, {
        statusText: message.radioText || "",
      });
    }
  };

  const hdRadioChannel = new Channel<HdRadioState>();
  hdRadioChannel.onmessage = (message) => {
    setGlobalState((old) => ({ ...old, hdRadioState: message }));
    if (currentSdrArgs) {
      updateSdrGlobalState(currentSdrArgs!, {
        statusText: message.title || "",
      });
    }
  };

  const updateSdrGlobalState = (
    sdrArgs: AvailableSdrArgs,
    changes: { functionName?: string; statusText?: string }
  ) => {
    setGlobalState((old) => {
      const newSdrs = [...old.sdrStates];
      const currentSdrIndex = newSdrs.findIndex(
        (value) => value.args.serial == sdrArgs?.serial
      );
      if (currentSdrIndex == -1) {
        return old;
      }
      const newChanges = {
        functionName:
          changes.functionName || newSdrs[currentSdrIndex].functionName,
        statusText: changes.statusText || newSdrs[currentSdrIndex].statusText,
      };
      newSdrs[currentSdrIndex] = { ...newSdrs[currentSdrIndex], ...newChanges };
      console.log(newSdrs[currentSdrIndex], changes.functionName);
      return { ...old, sdrStates: newSdrs };
    });
  };

  const start_stream = async () => {
    if (!globalState.defaultSdrArgs) {
      return;
    }

    setStatus(RtlSdrStatus.Starting);
    setCurrentStation({
      type: currentStationType,
      frequency: streamSettings.freq,
      channel: undefined,
    });
    setError("");
    await invoke<string>("start_stream", {
      streamSettings,
      sdrArgs: globalState.defaultSdrArgs,
      rbdsChannel,
      hdRadioChannel,
    });
    setCurrentSdrArgs(globalState.defaultSdrArgs);
    updateSdrGlobalState(globalState.defaultSdrArgs, {
      functionName: streamType.toString().toUpperCase() + " Radio",
      statusText: "",
    });

    // If no RBDS data after 10 seconds, alert user
    counterId = setTimeout(() => set10SecondsElapsed(true), 10 * 1000);
  };
  const stop_stream = async () => {
    setStatus(RtlSdrStatus.Pausing);
    if (counterId) {
      clearTimeout(counterId);
      counterId = undefined;
    }
    set10SecondsElapsed(false);
    await invoke<string>("stop_stream", {});
    updateSdrGlobalState(currentSdrArgs!, {
      functionName: undefined,
      statusText: undefined,
    });
    setCurrentSdrArgs(undefined);
    setCurrentStation(undefined);
    setRequestedStation(null);
    setStatus(RtlSdrStatus.Stopped);
    setGlobalState((old) => ({ ...old, rbdsData: {} as RbdsData }));
  };

  appWindow.listen("rtlsdr_status", (event: { payload: string }) => {
    if (event.payload.startsWith(streamType.toString().toLowerCase() + "_")) {
      const fixed_payload = event.payload.split("_").slice(1).join("_");
      setStatus(
        RtlSdrStatus[
          Object.keys(RtlSdrStatus)[
            Object.values(RtlSdrStatus).indexOf(fixed_payload as RtlSdrStatus)
          ] as keyof typeof RtlSdrStatus
        ]
      );
    }
  });

  appWindow.listen("rtlsdr_err", async (event: { payload: string }) => {
    setError(event.payload);
    await setCurrentStation(undefined);
    await setRequestedStation(null);
  });

  return (
    <div className="flex xl:flex-row flex-col gap-4 xl:max-w-[48rem] max-w-[24rem] grow">
      <form
        className="grid gap-3 max-w-[24rem] w-full xl:grow h-max mx-auto"
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
            step={streamType == StreamType.AM ? 10 : 0.2}
            min={streamType == StreamType.AM ? 540 : 88.1}
            max={streamType == StreamType.AM ? 1700 : 107.9}
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
          <Label htmlFor="volume_slider">
            Volume - {Math.round(streamSettings.volume * 100)}%
          </Label>
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
        <div className="grid w-full gap-1.5">
          <Label htmlFor="gain_slider">Gain - {streamSettings.gain} dB</Label>
          <Slider
            min={0.0}
            max={50.0}
            step={0.1}
            value={[streamSettings.gain]}
            id="gain_slider"
            className="py-[2px]"
            onValueChange={(values) => {
              setStreamSettings((old) => ({ ...old, gain: values[0] }));
            }}
          />
        </div>
        <Button
          onClick={() => {
            if (status == RtlSdrStatus.Running) {
              setRequestedStation(null);
            }
          }}
          disabled={
            status == RtlSdrStatus.Starting ||
            status == RtlSdrStatus.Pausing ||
            (status == RtlSdrStatus.Stopped && !globalState.defaultSdrArgs)
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
          ) : !globalState.defaultSdrArgs ? (
            "Select an SDR Before Starting"
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
              let stationTitle = `${streamType.valueOf()} ${
                streamSettings.freq
              }`;

              if (globalState.rbdsData.programType) {
                stationTitle += ` - ${globalState.rbdsData.programType}`;
              }

              const stationData: StationDetails = {
                type: currentStationType,
                title: stationTitle,
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
        {streamType != StreamType.FM && (
          <HoursListenedToRadioView listenedForSeconds={totalSecondsListened} />
        )}
      </form>
      {streamType != StreamType.AM && (
        <div className="max-w-[24rem] w-full xl-grow">
          <Tabs
            defaultValue="radioInfo"
            className={`transition-all w-full ${
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
                {streamType == StreamType.FM ? (
                  <RbdsDataView
                    globalState={globalState}
                    has10SecondsElapsed={has10SecondsElapsed}
                  />
                ) : (
                  <HdRadioStateView globalState={globalState} />
                )}
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
          <HoursListenedToRadioView
            className="mt-2"
            listenedForSeconds={totalSecondsListened}
          />
        </div>
      )}
    </div>
  );
}

function HdRadioStateView({ globalState }: { globalState: GlobalState }) {
  return (
    <>
      <TabsContent value="radioInfo">
        <Card className="overflow-clip">
          <CardHeader className="flex-row gap-4">
            {globalState.hdRadioState.thumbnail_data ? (
              <img
                src={globalState.hdRadioState.thumbnail_data!}
                width={100}
                className="rounded-sm aspect-square"
              />
            ) : (
              <MusicIcon className="min-w-[100px] min-h-[100px] px-[20px] py-[20px] bg-stone-800 rounded-sm aspect-square" />
            )}
            <div className="flex flex-col">
              {globalState.hdRadioState.title ? (
                <CardTitle className="whitespace-pre-wrap">
                  {globalState.hdRadioState.title}
                </CardTitle>
              ) : (
                <Skeleton className="h-6 max-w-52" />
              )}
              <CardDescription>
                {globalState.hdRadioState.artist ? (
                  globalState.hdRadioState.artist
                ) : (
                  <Skeleton className="h-4 max-w-20" />
                )}
              </CardDescription>
            </div>
          </CardHeader>
          <Separator className="mb-6" />
          <CardContent className="flex flex-col gap-0.5">
            <div className="flex justify-between items-center">
              <CardTitle>
                {globalState.hdRadioState.station_info?.name}
              </CardTitle>
              <HoverCard>
                <HoverCardTrigger>
                  <div className="-mt-1 hover:cursor-pointer hover:border-b-white border-transparent border-b-[1px]">
                    <div className="flex gap-1 -mb-1">
                      <Globe className="w-4 pb-[2px]" />{" "}
                      <span>
                        {globalState.hdRadioState.station_info?.country_code}
                      </span>
                    </div>
                  </div>
                </HoverCardTrigger>
                <HoverCardContent>
                  <p>
                    <b>Location:</b>{" "}
                    {globalState.hdRadioState.station_info?.location.join(", ")}
                  </p>
                  <p>
                    <b>Altitude:</b>{" "}
                    {globalState.hdRadioState.station_info?.altitude}m
                  </p>
                </HoverCardContent>
              </HoverCard>
            </div>
            <CardDescription>
              {globalState.hdRadioState.station_info?.slogan}
            </CardDescription>
          </CardContent>
        </Card>
      </TabsContent>
      <TabsContent value="advancedInfo">
        <Card>
          <CardHeader>
            <CardTitle>Advanced Info</CardTitle>
            <CardDescription>
              This is all the other HD Radio data that RTL-SDR Radio can decode.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-2 text-wrap whitespace-normal">
            {"Coming soon..." ||
              JSON.stringify(globalState.hdRadioState, undefined, 4)}
          </CardContent>
        </Card>
      </TabsContent>
    </>
  );
}

function RbdsDataView({
  globalState,
  has10SecondsElapsed,
}: {
  globalState: GlobalState;
  has10SecondsElapsed: boolean;
}) {
  return (
    <>
      <TabsContent value="radioInfo">
        <Card>
          <CardHeader>
            {globalState.rbdsData.radioText ? (
              <CardTitle
                className="whitespace-pre-wrap"
                dangerouslySetInnerHTML={{
                  __html:
                    globalState.rbdsData.radioText &&
                    globalState.rbdsData.radioText.trimEnd()
                      ? globalState.rbdsData.radioText
                          .trimEnd()
                          .replace(
                            /( {2,})/g,
                            '<span class="font-mono">$1</span>'
                          )
                      : "",
                }}
              ></CardTitle>
            ) : (
              <Skeleton className="h-6 max-w-52" />
            )}
            <CardDescription>
              {globalState.rbdsData.programType ? (
                globalState.rbdsData.programType
              ) : (
                <Skeleton className="h-4 max-w-20" />
              )}
            </CardDescription>
          </CardHeader>
          {has10SecondsElapsed &&
            Object.values(globalState.rbdsData).every(
              (x) => x === undefined
            ) && (
              <CardContent className="flex flex-col items-center align-middle justify-center">
                <CardDescription className="text-center">
                  Cannot receive RBDS signal! It is either too weak or the
                  station does not support RBDS.
                </CardDescription>
              </CardContent>
            )}
        </Card>
      </TabsContent>
      <TabsContent value="advancedInfo">
        <Card>
          <CardHeader>
            <CardTitle>Advanced Info</CardTitle>
            <CardDescription>
              This is all the other RBDS/RDS data that RTL-SDR Radio can decode.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-2">
            <span className="flex items-center gap-1">
              <b>Program Service Name:</b>{" "}
              <span className="font-mono">
                {globalState.rbdsData.serviceName != undefined ? (
                  <>
                    {globalState.rbdsData.serviceName}
                    {globalState.rbdsData.ptyName
                      ? ` - ${globalState.rbdsData.ptyName}`
                      : ""}
                  </>
                ) : (
                  <div>
                    <Skeleton className="h-5 w-24" />
                  </div>
                )}
              </span>
            </span>
            <span className="flex items-center gap-1">
              <b>Program Identification Code:</b>{" "}
              {globalState.rbdsData.pi != undefined ? (
                `0x${globalState.rbdsData.pi.toString(16)}`
              ) : (
                <Skeleton className="h-4 w-[3.5rem]" />
              )}
            </span>
            <span className="flex items-center gap-1">
              <b>Radio Type:</b>{" "}
              {globalState.rbdsData.decoderInfo &&
              globalState.rbdsData.decoderInfo.diIsStereo != undefined ? (
                globalState.rbdsData.msFlag ? (
                  "Music"
                ) : (
                  "Speech"
                )
              ) : (
                <Skeleton className="h-4 w-[3.5rem]" />
              )}
            </span>
            <span className="font-bold">Decoder Identification Info</span>
            <div className="indent-4 flex flex-col -mt-2">
              <span className="flex items-center gap-1">
                <b>Channels:</b>{" "}
                {globalState.rbdsData.decoderInfo &&
                globalState.rbdsData.decoderInfo.diIsStereo != undefined ? (
                  globalState.rbdsData.decoderInfo.diIsStereo ? (
                    "Stereo"
                  ) : (
                    "Mono"
                  )
                ) : (
                  <Skeleton className="h-4 w-[4.25rem]" />
                )}
              </span>
              <span className="flex items-center gap-1">
                <b>Binaural Audio:</b>{" "}
                {globalState.rbdsData.decoderInfo &&
                globalState.rbdsData.decoderInfo.diIsBinaural != undefined ? (
                  globalState.rbdsData.decoderInfo.diIsBinaural ? (
                    "Yes"
                  ) : (
                    "No"
                  )
                ) : (
                  <Skeleton className="h-4 w-8" />
                )}
              </span>
              <span className="flex items-center gap-1">
                <b>Compression:</b>{" "}
                {globalState.rbdsData.decoderInfo &&
                globalState.rbdsData.decoderInfo.diIsCompressed != undefined ? (
                  globalState.rbdsData.decoderInfo.diIsCompressed ? (
                    "Yes"
                  ) : (
                    "No"
                  )
                ) : (
                  <Skeleton className="h-4 w-8" />
                )}
              </span>
              <span className="flex items-center gap-1">
                <b>PTY Type:</b>{" "}
                {globalState.rbdsData.decoderInfo &&
                globalState.rbdsData.decoderInfo.diIsPtyDynamic != undefined ? (
                  globalState.rbdsData.decoderInfo.diIsPtyDynamic ? (
                    "Dynamic"
                  ) : (
                    "Static"
                  )
                ) : (
                  <Skeleton className="h-4 w-[4.25rem]" />
                )}
              </span>
            </div>
            <div className="flex gap-1 items-center">
              <span className="items-center gap-1">
                <b>Traffic Info:</b>{" "}
                {(() => {
                  if (
                    globalState.rbdsData.ta != undefined &&
                    globalState.rbdsData.tp != undefined
                  ) {
                    switch (true) {
                      case globalState.rbdsData.tp == false &&
                        globalState.rbdsData.ta == false:
                        return "This radio station does not carry traffic announcements.";
                      case globalState.rbdsData.tp == false &&
                        globalState.rbdsData.ta == true:
                        return "This radio station does not carry traffic announcements, but it carries EON information about a station that does.";
                      case globalState.rbdsData.tp == true &&
                        globalState.rbdsData.ta == false:
                        return "This radio station carries traffic announcements, but none are ongoing presently.";
                      case globalState.rbdsData.tp == true &&
                        globalState.rbdsData.ta == true:
                        return "There is an ongoing traffic announcement.";
                    }
                  }
                })()}
              </span>
              {(globalState.rbdsData.tp == undefined ||
                globalState.rbdsData.ta == undefined) && (
                <Skeleton className="h-4 grow" />
              )}
            </div>
            {(globalState.rbdsData.tp == undefined ||
              globalState.rbdsData.ta == undefined) && (
              <Skeleton className="h-4 w-64 -mt-0.5" />
            )}
          </CardContent>
        </Card>
      </TabsContent>
    </>
  );
}

function HoursListenedToRadioView({
  className,
  listenedForSeconds,
}: {
  className?: string;
  listenedForSeconds: number;
}) {
  let unit = "minute";
  let value = Math.floor(listenedForSeconds / 60);

  if (value >= 60) {
    unit = "hour";
    value = Math.floor(value / 60);
  }

  if (value != 1) {
    unit += "s";
  }

  return (
    <div className={`text-muted-foreground ${className}`}>
      You have listened to radio for {value} {unit}.
    </div>
  );
}
