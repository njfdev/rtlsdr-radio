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
import { MouseEventHandler, useState } from "react";
import Map, {
  AttributionControl,
  FullscreenControl,
  Marker,
  NavigationControl,
  ScaleControl,
} from "react-map-gl/maplibre";
import "maplibre-gl/dist/maplibre-gl.css";
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
  Mountain,
  MoveLeft,
  MoveVertical,
  Plane,
} from "lucide-react";
import airplaneIcon from "../../public/airplane-icon.svg";
import Image from "next/image";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "./ui/hover-card";
import { Separator } from "./ui/separator";

const appWindow = getCurrentWebviewWindow();

export default function AdsbDecoderView() {
  const [modesState, setModesState] = useState<ModesState | undefined>(
    undefined
  );
  const [currentAircraftIcao, setCurrentAircraftIcao] = useState<
    number | undefined
  >(undefined);

  const start_decoding = async () => {
    await invoke<string>("start_adsb_decoding", {
      streamSettings: {
        gain: 20.0,
      } as AdsbDecodeSettings,
    });
  };
  const stop_decoding = async () => {
    await invoke<string>("stop_adsb_decoding", {});
  };

  appWindow.listen("modes_state", (event: { payload: ModesState }) => {
    setModesState(event.payload);
  });

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <h1>ADS-B Decoder</h1>
      <Button className="w-max" onClick={() => start_decoding()}>
        Start Decoding
      </Button>
      <Button className="w-max" onClick={() => stop_decoding()}>
        Stop Decoding
      </Button>
      <ResizablePanelGroup direction="horizontal">
        <ResizablePanel>
          <Map
            initialViewState={{
              // initialize at geographic center of the US
              latitude: 39.8283,
              longitude: -98.5795,
              zoom: 2,
            }}
            style={{
              flexGrow: 1,
              height: "100%",
            }}
            mapStyle="https://tiles.stadiamaps.com/styles/alidade_satellite.json"
          >
            <NavigationControl />
            <ScaleControl />
            <FullscreenControl />
            {modesState?.aircraft.map((aircraft) => {
              if (
                aircraft.adsbState?.longitude &&
                aircraft.adsbState?.latitude &&
                aircraft.adsbState.heading
              ) {
                return (
                  <Marker
                    longitude={aircraft.adsbState.longitude}
                    latitude={aircraft.adsbState.latitude}
                    pitchAlignment="map"
                    rotationAlignment="map"
                    anchor="center"
                  >
                    <HoverCard>
                      <HoverCardTrigger>
                        <Image
                          src={airplaneIcon}
                          className="w-[1.75rem] hover:cursor-pointer"
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
      <CardHeader>
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
      <CardContent className="mt-1 flex w-full">
        <p className="grow basis-0">
          {aircraft.adsbState?.altitude || "-----"} feet
        </p>
        <b>·</b>
        <p className="grow basis-0 text-end">
          {aircraft.adsbState?.speed || "---"} knots
        </p>
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
        {aircraft.adsbState && (
          <>
            {aircraft.adsbState.altitude && (
              <p className="flex gap-1">
                <Mountain />
                <span>
                  <b>
                    Altitude ({aircraft.adsbState.altitudeSource?.toString()}
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
                  <b>Position:</b> {aircraft.adsbState.latitude.toFixed(5)}°,{" "}
                  {aircraft.adsbState.longitude?.toFixed(5)}°
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
                  {aircraft.adsbState.heading}°
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
      </CardContent>
    </Card>
  );
}
