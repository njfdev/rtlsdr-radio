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
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useState } from "react";
import Map from "react-map-gl/maplibre";

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
    <div>
      <h1>ADS-B Decoder</h1>
      <Button onClick={() => start_decoding()}>Start Decoding</Button>
      <Button onClick={() => stop_decoding()}>Stop Decoding</Button>
      <Map
        initialViewState={{
          longitude: -122.4,
          latitude: 37.8,
          zoom: 14,
        }}
        style={{ width: 600, height: 400 }}
        mapStyle={{
          version: 8,
          sources: {
            "raster-tiles": {
              type: "raster",
              tiles: ["https://tile.openstreetmap.org/{z}/{x}/{y}.png"],
              tileSize: 256,
              attribution:
                '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
            },
          },
          layers: [
            {
              id: "simple-tiles",
              type: "raster",
              source: "raster-tiles",
              minzoom: 0,
              maxzoom: 22,
            },
          ],
        }}
      />
      {modesState && (
        <>
          <h2>Aircraft</h2>
          <div className="flex flex-col gap-4">
            {modesState?.aircraft.map((aircraft: AircraftState) => {
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
                            Altitude (
                            {aircraft.adsbState.altitudeSource?.toString()}):{" "}
                            {aircraft.adsbState.altitude} feet
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
                              aircraft.adsbState
                                ?.preferredVerticalVelocitySource ==
                              AltitudeType.GNSS
                                ? aircraft.adsbState.gnssVerticalVelocity
                                : aircraft.adsbState?.barometerVerticalVelocity;

                            let secondaryVerticalVelocitySource =
                              aircraft.adsbState
                                .preferredVerticalVelocitySource ==
                              AltitudeType.GNSS
                                ? AltitudeType.Barometer
                                : AltitudeType.GNSS;
                            let secondaryVerticalVelocity =
                              aircraft.adsbState
                                ?.preferredVerticalVelocitySource !=
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
                          <li>
                            Wake Vortex Category:{" "}
                            {aircraft.adsbState.wakeVortexCat}
                          </li>
                        )}
                      </ul>
                    </>
                  )}
                </div>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}
