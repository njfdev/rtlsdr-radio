"use client";

import { invoke } from "@tauri-apps/api/core";
import {
  AdsbDecodeSettings,
  AircraftState,
  AirspeedType,
  AltitudeType,
  ModesState,
} from "@/lib/types";
import { Button } from "./ui/button";
import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from "./ui/resizable";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import {
  Dispatch,
  MouseEventHandler,
  SetStateAction,
  useEffect,
  useState,
} from "react";
import { Map, Marker, Overlay, ZoomControl } from "pigeon-maps";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "./ui/card";
import {
  ArrowLeft,
  Compass,
  Gauge,
  Globe,
  Loader2,
  Mountain,
  MoveLeft,
  MoveVertical,
  Plane,
  Timer,
} from "lucide-react";
import airplaneIcon from "../../public/airplane-icon.svg";
import Image from "next/image";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "./ui/hover-card";
import { Separator } from "./ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./ui/tabs";

enum AdsbStatus {
  Starting = "starting",
  Running = "running",
  Stopping = "stopping",
  Stopped = "stopped",
}

const appWindow = getCurrentWebviewWindow();

export default function AdsbDecoderView({
  isSdrInUse,
  setIsSdrInUse,
  shouldStop,
  setShouldStop,
}: {
  isSdrInUse: boolean;
  setIsSdrInUse: Dispatch<SetStateAction<boolean>>;
  shouldStop: boolean;
  setShouldStop: Dispatch<SetStateAction<boolean>>;
}) {
  const [modesState, setModesState] = useState<ModesState | undefined>(
    undefined
  );
  const [adsbStatus, setAdsbStatus] = useState<AdsbStatus>(AdsbStatus.Stopped);
  const [currentAircraftIcao, setCurrentAircraftIcao] = useState<
    number | undefined
  >(undefined);
  const [currentMillisecondsSinceEpoch, setCurrentMillisecondsSinceEpoch] =
    useState(new Date().getTime());

  // initialize at geographic center of the US
  const [mapCenter, setMapCenter] = useState<[number, number]>([
    39.8283, -98.5795,
  ]);
  const [mapZoom, setMapZoom] = useState(4);

  const start_decoding = async () => {
    setAdsbStatus(AdsbStatus.Starting);
    await setIsSdrInUse(true);
    await invoke<string>("start_adsb_decoding", {
      streamSettings: {
        gain: 20.0,
      } as AdsbDecodeSettings,
    });
  };
  const stop_decoding = async () => {
    setAdsbStatus(AdsbStatus.Stopping);
    await invoke<string>("stop_adsb_decoding", {});
    setIsSdrInUse(false);
    setShouldStop(false);
    setModesState(undefined);
  };

  appWindow.listen("modes_state", (event: { payload: ModesState }) => {
    setModesState(event.payload);
  });

  appWindow.listen("adsb_status", (event: { payload: string }) => {
    setAdsbStatus(
      AdsbStatus[
        Object.keys(AdsbStatus)[
          Object.values(AdsbStatus).indexOf(event.payload as AdsbStatus)
        ] as keyof typeof AdsbStatus
      ]
    );
  });

  useEffect(() => {
    if (shouldStop) {
      stop_decoding();
    }
  }, [shouldStop]);

  useEffect(() => {
    const updateTimeSinceEpoch = () => {
      setCurrentMillisecondsSinceEpoch(new Date().getTime());
    };

    // update milliseconds every 100 milliseconds
    const intervalId = setInterval(updateTimeSinceEpoch, 100);
    return () => clearInterval(intervalId);
  });

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="px-4 pb-4 flex flex-col gap-1">
        <h1 className="font-bold text-2xl">ADS-B Decoder</h1>
        <Button
          className="w-max"
          variant={adsbStatus == AdsbStatus.Running ? "secondary" : "default"}
          disabled={
            adsbStatus == AdsbStatus.Starting ||
            adsbStatus == AdsbStatus.Stopping
          }
          onClick={() =>
            adsbStatus == AdsbStatus.Stopped
              ? start_decoding()
              : stop_decoding()
          }
        >
          {adsbStatus == AdsbStatus.Stopped ? (
            "Start Decoding"
          ) : adsbStatus == AdsbStatus.Running ? (
            "Stop Decoding"
          ) : (
            <>
              <Loader2 className="animate-spin mr-2" />{" "}
              {adsbStatus == AdsbStatus.Starting
                ? "Starting..."
                : "Stopping..."}
            </>
          )}
        </Button>
      </div>
      <ResizablePanelGroup direction="horizontal">
        <ResizablePanel>
          <Map
            center={mapCenter}
            zoom={mapZoom}
            onBoundsChanged={(newBounds) => {
              setMapCenter(newBounds.center);
              setMapZoom(newBounds.zoom);
            }}
          >
            <ZoomControl />
            {modesState?.aircraft.map((aircraft) => {
              if (
                aircraft.adsbState?.longitude &&
                aircraft.adsbState?.latitude &&
                aircraft.adsbState.heading
              ) {
                return (
                  <Overlay
                    anchor={[
                      aircraft.adsbState.latitude,
                      aircraft.adsbState.longitude,
                    ]}
                    offset={[14, 14]}
                    key={aircraft.icaoAddress + "-airplane-icon"}
                  >
                    <HoverCard>
                      <HoverCardTrigger>
                        <Image
                          src={airplaneIcon}
                          className="w-[28px] hover:cursor-pointer"
                          alt={`Icon of airplane with callsign ${aircraft.adsbState.callsign}`}
                          style={{
                            // offset 90 degrees because icon is facing east
                            rotate: `${aircraft.adsbState.heading - 90}deg`,
                          }}
                          onClick={() => {
                            setCurrentAircraftIcao(aircraft.icaoAddress);
                          }}
                        />
                      </HoverCardTrigger>
                      <HoverCardContent side="top" className="w-max p-2">
                        <p>
                          <b>
                            {aircraft.adsbState.callsign
                              ? "Call Sign: "
                              : "ICAO Address: "}
                          </b>
                          {aircraft.adsbState.callsign
                            ? aircraft.adsbState.callsign
                            : aircraft.icaoAddress.toString(16)}
                        </p>
                        <div className="flex gap-2">
                          <p>{aircraft.adsbState.altitude || "-----"} feet</p>
                          <span>·</span>
                          <p>{aircraft.adsbState.speed || "---"} knots</p>
                        </div>
                      </HoverCardContent>
                    </HoverCard>
                  </Overlay>
                );
              }
            })}
          </Map>
        </ResizablePanel>

        <ResizableHandle />

        <ResizablePanel
          defaultSize={25}
          className="grow h-full overflow-hidden"
        >
          <Card className="w-full h-full rounded-none border-x-0 border-b-0 overflow-hidden flex flex-col">
            <CardHeader>
              <CardTitle>Aircraft</CardTitle>
              <CardDescription>
                {modesState?.aircraft.filter((aircraft, index, array) => {
                  return (
                    aircraft.adsbState?.latitude &&
                    aircraft.adsbState.longitude &&
                    aircraft.adsbState.heading
                  );
                }).length || 0}
                /{modesState?.aircraft.length || 0} displayed on the map.
              </CardDescription>
            </CardHeader>
            <CardContent className="flex flex-col gap-3 overflow-y-auto grow pb-3 px-3">
              {modesState?.aircraft.map((aircraft: AircraftState) => {
                if (!currentAircraftIcao) {
                  return (
                    <AircraftDataPreview
                      aircraft={aircraft}
                      key={aircraft.icaoAddress}
                      onClick={() => {
                        setCurrentAircraftIcao(aircraft.icaoAddress);
                        // zoom into aircraft when clicking details if available
                        if (
                          aircraft.adsbState?.latitude &&
                          aircraft.adsbState.longitude &&
                          aircraft.adsbState.heading
                        ) {
                          setMapCenter([
                            aircraft.adsbState?.latitude,
                            aircraft.adsbState?.longitude,
                          ]);
                          setMapZoom(10);
                        }
                      }}
                    />
                  );
                } else if (currentAircraftIcao == aircraft.icaoAddress) {
                  return (
                    <AircraftData
                      key={aircraft.icaoAddress + "-full-details"}
                      aircraft={aircraft}
                      onClickBack={() => setCurrentAircraftIcao(undefined)}
                    />
                  );
                }
              })}
            </CardContent>
          </Card>
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}

function AircraftDataPreview({
  aircraft,
  onClick,
}: {
  aircraft: AircraftState;
  onClick: MouseEventHandler<HTMLDivElement>;
}) {
  return (
    <Card className="p-4 *:p-0 hover:cursor-pointer" onClick={onClick}>
      <CardHeader className="flex justify-between">
        <CardTitle className="text-xl flex gap-2">
          <Image
            src={airplaneIcon}
            className="w-[1.75rem] hover:cursor-pointer"
            alt={`Icon of airplane with callsign ${aircraft.adsbState?.callsign}`}
            style={{
              // offset 90 degrees because icon is facing east
              rotate: `${(aircraft.adsbState?.heading || 0) - 90}deg`,
            }}
          />
          {aircraft.adsbState?.callsign ||
            `ICAO: ${aircraft.icaoAddress.toString(16)}`}
        </CardTitle>
      </CardHeader>
      <CardContent className="mt-1 w-full flex flex-col gap-1">
        <div className="flex">
          <p className="grow basis-0">
            {aircraft.adsbState?.altitude || "-----"} feet
          </p>
          <b>·</b>
          <p className="grow basis-0 text-end">
            {aircraft.adsbState?.speed || "---"} knots
          </p>
        </div>

        <div className="flex gap-1">
          <Timer />
          <span>
            Last seen{" "}
            {Math.round(new Date().getTime() / 1000) -
              aircraft.lastMessageTimestamp.secs_since_epoch}{" "}
            seconds ago
          </span>
        </div>
      </CardContent>
    </Card>
  );
}

function AircraftData({
  aircraft,
  onClickBack,
}: {
  aircraft: AircraftState;
  onClickBack: MouseEventHandler<HTMLParagraphElement>;
}) {
  return (
    <Card className="p-6 *:p-0">
      <CardHeader>
        <CardDescription
          onClick={onClickBack}
          className="-mt-3 flex gap-1 align-middle items-center hover:cursor-pointer"
        >
          <MoveLeft scale={0.5} /> Back
        </CardDescription>
        {aircraft.icaoDetails?.url_photo && (
          <div className="relative w-full max-w-[24rem] aspect-video">
            <Image
              className="w-full"
              src={aircraft.icaoDetails.url_photo.replace(
                "https://",
                "http://"
              )}
              fill={true}
              alt={`Image of aircraft with ICAO address ${aircraft.icaoAddress}`}
            />
          </div>
        )}
        <CardTitle>
          {aircraft.adsbState?.callsign ||
            `ICAO Address: ${aircraft.icaoAddress.toString(16)}`}
        </CardTitle>
        {aircraft.adsbState?.callsign && (
          <CardDescription>
            ICAO Address: {aircraft.icaoAddress.toString(16)}
          </CardDescription>
        )}
      </CardHeader>
      <CardContent className="mt-3">
        <Tabs defaultValue="flight-details">
          <TabsList>
            <TabsTrigger value="flight-details">Flight Details</TabsTrigger>
            <TabsTrigger value="aircraft-details">Aircraft Details</TabsTrigger>
          </TabsList>
          <TabsContent value="flight-details">
            {aircraft.adsbState && (
              <>
                {aircraft.adsbState.altitude && (
                  <p className="flex gap-1">
                    <Mountain />
                    <span>
                      <b>
                        Altitude (
                        {aircraft.adsbState.altitudeSource?.toString()}
                        ):
                      </b>{" "}
                      {aircraft.adsbState.altitude} feet
                    </span>
                  </p>
                )}
                {aircraft.adsbState.speed && (
                  <>
                    <p className="flex gap-1">
                      <Gauge />
                      <span>
                        <b>
                          {aircraft.adsbState.velocityType == "GroundSpeed"
                            ? "Ground"
                            : aircraft.adsbState.velocityType?.AirSpeed ==
                              AirspeedType.IAS
                            ? "Indicated"
                            : "True"}{" "}
                          Speed:
                        </b>{" "}
                        {aircraft.adsbState.speed} knots
                      </span>
                    </p>
                  </>
                )}
                {aircraft.adsbState.latitude && (
                  <p className="flex gap-1">
                    <Globe />
                    <span>
                      {/* We round the lat and long to 5 decimal places (which is a precision of about 1.1 meters). Planes
                      are only accurate to around 1 meter, so any more decimal places is extra (and garbage) information. */}
                      <b>Position:</b> {aircraft.adsbState.latitude.toFixed(5)}
                      °, {aircraft.adsbState.longitude?.toFixed(5)}°
                    </span>
                  </p>
                )}

                {aircraft.adsbState.heading && (
                  <p className="flex gap-1">
                    <Compass />
                    <span>
                      <b>
                        Heading (
                        {aircraft.adsbState.velocityType == "GroundSpeed"
                          ? "GNSS"
                          : "Magnetic"}
                        ):
                      </b>{" "}
                      {aircraft.adsbState.heading.toFixed(2)}°
                    </span>
                  </p>
                )}
                {aircraft.adsbState.preferredVerticalVelocitySource &&
                  (() => {
                    let mainVerticalVelocity =
                      aircraft.adsbState?.preferredVerticalVelocitySource ==
                      AltitudeType.GNSS
                        ? aircraft.adsbState.gnssVerticalVelocity
                        : aircraft.adsbState?.barometerVerticalVelocity;
                    return (
                      <p className="flex gap-1">
                        <MoveVertical />
                        <span>
                          <b>
                            Vertical Speed (
                            {aircraft.adsbState.preferredVerticalVelocitySource.toString()}
                            ):
                          </b>{" "}
                          {mainVerticalVelocity} ft/min
                        </span>
                      </p>
                    );
                  })()}
                {aircraft.adsbState.wakeVortexCat && (
                  <p className="flex gap-1">
                    <Plane />{" "}
                    <span>
                      <b>Wake Vortex Category:</b>{" "}
                      {aircraft.adsbState.wakeVortexCat}
                    </span>
                  </p>
                )}
              </>
            )}
          </TabsContent>
          <TabsContent value="aircraft-details">
            <p>
              <b>Plane Model:</b> {aircraft.icaoDetails?.manufacturer}{" "}
              {aircraft.icaoDetails?.type}
            </p>
            <p>
              <b>Registered Owner:</b>{" "}
              {aircraft.icaoDetails?.registered_owner || "Unknown"}
            </p>
            <p>
              <b>Registration Country:</b>{" "}
              {aircraft.icaoDetails?.registered_owner_country_name || "Unknown"}
            </p>
            <p>
              <b>Registration Code:</b>{" "}
              {aircraft.icaoDetails?.registration || "Unknown"}
            </p>
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  );
}
