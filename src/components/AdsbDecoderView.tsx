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
import { useState } from "react";
import Map, {
  AttributionControl,
  FullscreenControl,
  Marker,
  NavigationControl,
  ScaleControl,
} from "react-map-gl/maplibre";
import "maplibre-gl/dist/maplibre-gl.css";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Plane } from "lucide-react";
import airplaneIcon from "../../public/airplane-icon.svg";
import Image from "next/image";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "./ui/hover-card";
import { Separator } from "./ui/separator";

const appWindow = getCurrentWebviewWindow();

export default function AdsbDecoderView() {
  const [modesState, setModesState] = useState<ModesState | undefined>(
    undefined
  );

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
              {modesState?.aircraft.map((aircraft: AircraftState) => (
                <AircraftDataPreview aircraft={aircraft} />
              ))}
            </CardContent>
          </Card>
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}

function AircraftDataPreview({ aircraft }: { aircraft: AircraftState }) {
  return (
    <Card className="p-4 *:p-0">
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

function AircraftData({ aircraft }: { aircraft: AircraftState }) {
  return (
    <div key={aircraft.icaoAddress}>
      <h3>ICAO Address: {aircraft.icaoAddress.toString(16)}</h3>
      {aircraft.adsbState && (
        <>
          <p>ADS-B Data</p>
          <ul className="indent-4">
            {aircraft.adsbState.callsign && (
              <li>Callsign: {aircraft.adsbState.callsign}</li>
            )}
            {aircraft.adsbState.altitude && (
              <li>
                Altitude ({aircraft.adsbState.altitudeSource?.toString()}
                ): {aircraft.adsbState.altitude} feet
              </li>
            )}
            {aircraft.adsbState.latitude && (
              <>
                <li>Latitude: {aircraft.adsbState.latitude}°</li>
                <li>Longitude: {aircraft.adsbState.longitude}°</li>
              </>
            )}
            {aircraft.adsbState.preferredVerticalVelocitySource &&
              (() => {
                let mainVerticalVelocity =
                  aircraft.adsbState?.preferredVerticalVelocitySource ==
                  AltitudeType.GNSS
                    ? aircraft.adsbState.gnssVerticalVelocity
                    : aircraft.adsbState?.barometerVerticalVelocity;

                let secondaryVerticalVelocitySource =
                  aircraft.adsbState.preferredVerticalVelocitySource ==
                  AltitudeType.GNSS
                    ? AltitudeType.Barometer
                    : AltitudeType.GNSS;
                let secondaryVerticalVelocity =
                  aircraft.adsbState?.preferredVerticalVelocitySource !=
                  AltitudeType.GNSS
                    ? aircraft.adsbState.gnssVerticalVelocity
                    : aircraft.adsbState?.barometerVerticalVelocity;
                return (
                  <>
                    <li>
                      Vertical Velocity (
                      {aircraft.adsbState.preferredVerticalVelocitySource.toString()}
                      ): {mainVerticalVelocity} ft/min
                    </li>
                    <ul className="indent-8">
                      <li>
                        Vertical Velocity (
                        {secondaryVerticalVelocitySource.toString()}
                        ): {secondaryVerticalVelocity} ft/min
                      </li>
                    </ul>
                  </>
                );
              })()}

            {aircraft.adsbState.heading && (
              <>
                <li>
                  Heading (
                  {aircraft.adsbState.velocityType == "GroundSpeed"
                    ? "GNSS"
                    : "Magnetic"}
                  ): {aircraft.adsbState.heading}°
                </li>
              </>
            )}
            {aircraft.adsbState.speed && (
              <>
                <li>
                  {aircraft.adsbState.velocityType == "GroundSpeed"
                    ? "Ground"
                    : aircraft.adsbState.velocityType?.AirSpeed ==
                      AirspeedType.IAS
                    ? "Indicated"
                    : "True"}{" "}
                  Speed ({aircraft.adsbState.speedCategory}):{" "}
                  {aircraft.adsbState.speed} knots
                </li>
              </>
            )}
            {aircraft.adsbState.wakeVortexCat && (
              <li>Wake Vortex Category: {aircraft.adsbState.wakeVortexCat}</li>
            )}
          </ul>
        </>
      )}
    </div>
  );
}
