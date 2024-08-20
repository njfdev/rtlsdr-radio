"use client";

import { invoke } from "@tauri-apps/api/core";
import {
  AdsbDecodeSettings,
  AircraftState,
  AirspeedType,
  AltitudeType,
  EngineType,
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
  useRef,
  useState,
} from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "./ui/card";
import {
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
import airplaneIcon from "@/assets/airplane-icon.svg";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "./ui/hover-card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "./ui/tabs";
import Map, {
  FullscreenControl,
  Marker,
  NavigationControl,
} from "react-map-gl/maplibre";
import type { MapRef } from "react-map-gl/maplibre";
import "maplibre-gl/dist/maplibre-gl.css";
import { formatZipCode, snakeToTitle, titleCapitalization } from "@/lib/utils";

enum AdsbStatus {
  Starting = "starting",
  Running = "running",
  Stopping = "stopping",
  Stopped = "stopped",
}

const appWindow = getCurrentWebviewWindow();

export default function AdsbDecoderView({
  //isSdrInUse,
  setIsSdrInUse,
  shouldStop,
  setShouldStop,
}: {
  //isSdrInUse: boolean;
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
  const mapRef = useRef<MapRef>(null);

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

  // if the details of an aircraft that disappears is open, go back to the main list
  useEffect(() => {
    if (
      modesState?.aircraft.find((a) => a.icaoAddress == currentAircraftIcao) ==
      undefined
    ) {
      setCurrentAircraftIcao(undefined);
    }
  }, [modesState, currentAircraftIcao]);

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
            ref={mapRef}
            initialViewState={{
              latitude: 39.8283,
              longitude: -98.5795,
              zoom: 3.2,
            }}
            mapStyle={{
              version: 8,
              sources: {
                "satellite-tiles": {
                  type: "raster",
                  tiles: [
                    "https://mt0.google.com/vt/lyrs=s&x={x}&y={y}&z={z}",
                    "https://mt1.google.com/vt/lyrs=s&x={x}&y={y}&z={z}",
                    "https://mt2.google.com/vt/lyrs=s&x={x}&y={y}&z={z}",
                    "https://mt3.google.com/vt/lyrs=s&x={x}&y={y}&z={z}",
                  ],
                  tileSize: 256,
                  minzoom: 0,
                  maxzoom: 20,
                  attribution: `Map data @${new Date().getFullYear()} Google`,
                },
                "labels-lines-tiles": {
                  type: "raster",
                  tiles: [
                    "https://mt0.google.com/vt/lyrs=h&x={x}&y={y}&z={z}",
                    "https://mt1.google.com/vt/lyrs=h&x={x}&y={y}&z={z}",
                    "https://mt2.google.com/vt/lyrs=h&x={x}&y={y}&z={z}",
                    "https://mt3.google.com/vt/lyrs=h&x={x}&y={y}&z={z}",
                  ],
                  tileSize: 256,
                  minzoom: 0,
                  maxzoom: 20,
                  attribution: `Map data @${new Date().getFullYear()} Google`,
                },
              },
              layers: [
                {
                  id: "satellite-tiles",
                  type: "raster",
                  source: "satellite-tiles",
                },
                {
                  id: "labels-lines-tiles",
                  type: "raster",
                  source: "labels-lines-tiles",
                },
              ],
            }}
          >
            <FullscreenControl />
            <NavigationControl />
            {modesState?.aircraft.map((aircraft) => {
              if (
                aircraft.adsbState?.longitude &&
                aircraft.adsbState?.latitude &&
                aircraft.adsbState.heading
              ) {
                return (
                  <Marker
                    latitude={aircraft.adsbState.latitude}
                    longitude={aircraft.adsbState.longitude}
                    anchor="center"
                    key={aircraft.icaoAddress + "-airplane-icon"}
                  >
                    <HoverCard>
                      <HoverCardTrigger>
                        <img
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
                  </Marker>
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
                {modesState?.aircraft.filter((aircraft, _index, _array) => {
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
                          mapRef.current?.flyTo({
                            center: [
                              aircraft.adsbState?.longitude,
                              aircraft.adsbState?.latitude,
                            ],
                            zoom: 10,
                          });
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
          <img
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
            {Math.abs(
              (new Date().getTime() -
                aircraft.lastMessageTimestamp.secs_since_epoch * 1000 +
                aircraft.lastMessageTimestamp.nanos_since_epoch / 1000000) /
                1000
            ).toFixed(0)}{" "}
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
            <img
              className="w-full h-full object-cover"
              src={aircraft.icaoDetails.url_photo.replace(
                "https://",
                "http://"
              )}
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
            <TabsTrigger
              value="aircraft-details"
              disabled={
                !aircraft.icaoDetails &&
                !aircraft.registration &&
                !aircraft.flightRoute
              }
            >
              Aircraft Details
            </TabsTrigger>
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
                    const mainVerticalVelocity =
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
              <b>Plane Model:</b>{" "}
              {aircraft.icaoDetails
                ? `${aircraft.icaoDetails?.manufacturer} ${aircraft.icaoDetails?.type}`
                : `${
                    aircraft.registration?.aircraft_info
                      ? titleCapitalization(
                          `${aircraft.registration.aircraft_info.mfr} ${aircraft.registration.aircraft_info.model}`
                        )
                      : "Unknown"
                  }`}{" "}
              {aircraft.registration?.aircraft_type && (
                <>
                  (
                  {snakeToTitle(aircraft.registration.aircraft_type.toString())}
                  )
                </>
              )}
            </p>
            {aircraft.registration && (
              <>
                <p>
                  <b>Weight Class: </b>
                  {(() => {
                    switch (
                      aircraft.registration.aircraft_info.weight_class.toString()
                    ) {
                      case "Class1":
                        return "Up to 12,499 lbs";
                      case "Class2":
                        return "12,500 lbs to 19,999 lbs";
                      case "Class3":
                        return "20,000 lbs and over";
                      case "Class4":
                        return "Unmanned aerial vehicle (UAV) up to 55 lbs";
                    }
                  })()}
                </p>
                {aircraft.registration.aircraft_info.avg_cruising_speed && (
                  <p>
                    <b>Average Cruising Speed: </b>
                    {aircraft.registration.aircraft_info.avg_cruising_speed} mph
                  </p>
                )}
                <p>
                  <b>Number of Seats: </b>
                  {aircraft.registration.aircraft_info.seat_count}
                </p>
                <p>
                  <b>Number of Engines: </b>
                  {aircraft.registration.aircraft_info.engine_count}
                </p>
                {aircraft.registration.engine_info && (
                  <>
                    <p>
                      <b>Engine Model: </b>
                      {aircraft.registration.engine_info.mfr}{" "}
                      {aircraft.registration.engine_info.model}{" "}
                      {aircraft.registration.engine_type.toString() !=
                        "Unknown" && (
                        <>
                          (
                          {snakeToTitle(
                            aircraft.registration.engine_type.toString()
                          )}
                          )
                        </>
                      )}
                    </p>
                  </>
                )}
                {aircraft.registration.engine_info.horsepower && (
                  <p>
                    <b>Engine Horsepower: </b>
                    {aircraft.registration.engine_info.horsepower}
                  </p>
                )}
                {aircraft.registration.engine_info.lbs_of_thrust && (
                  <p>
                    <b>Engine Pounds of Thrust: </b>
                    {aircraft.registration.engine_info.lbs_of_thrust} lbs
                  </p>
                )}
                {aircraft.registration.air_worth_date && (
                  <p>
                    <b>Date of Air Worthiness Test: </b>
                    {new Date(
                      aircraft.registration.air_worth_date
                    ).toLocaleDateString()}
                  </p>
                )}
                {aircraft.registration.cert_issue_date && (
                  <p>
                    <b>Date Certified: </b>
                    {new Date(
                      aircraft.registration.cert_issue_date
                    ).toLocaleDateString()}
                  </p>
                )}
                <p>
                  <b>Certification Expiration: </b>
                  {new Date(
                    aircraft.registration.expiration_date
                  ).toLocaleDateString()}
                </p>
              </>
            )}
            <p>
              <b>Registered Owner:</b>{" "}
              {aircraft.icaoDetails
                ? aircraft.icaoDetails?.registered_owner
                : titleCapitalization(
                    aircraft.registration?.registrant_name || "Unknown"
                  )}{" "}
              {aircraft.registration && (
                <>
                  (
                  {snakeToTitle(
                    aircraft.registration.registrant_type?.toString() ||
                      "Unknown"
                  )}
                  )
                </>
              )}
            </p>
            <p>
              <b>Registration Country:</b>{" "}
              {aircraft.icaoDetails?.registered_owner_country_name || "Unknown"}
            </p>
            <p>
              <b>Registration Code:</b>{" "}
              {aircraft.icaoDetails
                ? aircraft.icaoDetails?.registration
                : aircraft.registration?.n_number
                ? `N${aircraft.registration?.n_number}`
                : "Unknown"}
            </p>
            {aircraft.registration?.registrant_street &&
              aircraft.registration.registrant_city &&
              aircraft.registration.registrant_state &&
              aircraft.registration.registrant_zip_code &&
              aircraft.registration.registrant_country_code == "US" && (
                <p>
                  <b>Registrant Address:</b>{" "}
                  {`${titleCapitalization(
                    aircraft.registration.registrant_street
                  )}, ${titleCapitalization(
                    aircraft.registration.registrant_city
                  )}, ${aircraft.registration.registrant_state} ${formatZipCode(
                    aircraft.registration.registrant_zip_code
                  )}`}
                </p>
              )}
            {aircraft.flightRoute?.airline && (
              <>
                <p>
                  <b>Airline:</b> {aircraft.flightRoute.airline.name}
                </p>
                <p>
                  <b>Airline Country:</b> {aircraft.flightRoute.airline.country}
                </p>
                <p>
                  <b>Airline Call Sign:</b>{" "}
                  {aircraft.flightRoute.airline.callsign}
                </p>
              </>
            )}
          </TabsContent>
        </Tabs>
      </CardContent>
    </Card>
  );
}
