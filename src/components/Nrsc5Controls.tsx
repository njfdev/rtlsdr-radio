"use client";

import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api";
import { appWindow } from "@tauri-apps/api/window";
import { Input, Button } from "@nextui-org/react";

enum Nrsc5Status {
  Stopped = "stopped",
  Starting = "starting",
  SdrFound = "sdr-found",
  Synced = "synchronized",
  SyncLost = "synchronization_lost",
}

export default function Nrsc5Controls() {
  const [freq, setFreq] = useState<string | undefined>();
  const [channel, setChannel] = useState<string | undefined>();

  const [nrsc5Status, setNrsc5Status] = useState(Nrsc5Status.Stopped);
  const [songTitle, setSongTitle] = useState("");
  const [songArtist, setSongArtist] = useState("");
  const [audioBitRate, setAudioBitRate] = useState("");
  const [stationName, setStationName] = useState("");
  const [slogan, setSlogan] = useState("");
  const [message, setMessage] = useState("");
  const [bitErrorRate, setBitErrorRate] = useState(0.0);

  const start_nrsc5 = () => {
    setNrsc5Status(Nrsc5Status.Starting);
    invoke<string>("start_nrsc5", { fmFreq: freq, channel: channel })
      .then((_result) => console.log("Started Playing"))
      .catch(console.error);
  };
  const stop_nrsc5 = () => {
    invoke<string>("stop_nrsc5", {})
      .then((_result) => console.log("Stopped Playing"))
      .catch(console.error);
  };

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
    setSongTitle(event.payload);
  });
  appWindow.listen("nrsc5_artist", (event: { payload: string }) => {
    setSongArtist(event.payload);
  });
  appWindow.listen("nrsc5_br", (event: { payload: string }) => {
    setAudioBitRate(event.payload);
  });
  appWindow.listen("nrsc5_station", (event: { payload: string }) => {
    setStationName(event.payload);
  });
  appWindow.listen("nrsc5_slogan", (event: { payload: string }) => {
    setSlogan(event.payload);
  });
  appWindow.listen("nrsc5_message", (event: { payload: string }) => {
    setMessage(event.payload);
  });
  appWindow.listen("nrsc5_ber", (event: { payload: string }) => {
    setBitErrorRate(parseFloat(event.payload));
  });

  return (
    <div className="flex gap-4">
      <div className="flex items-center justify-center align-middle min-w-[16rem]">
        {nrsc5Status == Nrsc5Status.Synced && (
          <div className="w-full">
            <h1>Title: {songTitle}</h1>
            <h2>Artist: {songArtist}</h2>
            <h3>Bit Rate: {audioBitRate}</h3>
            <h3>Bit Error Rate: {Math.floor(bitErrorRate * 100_00) / 100}%</h3>
            <hr />
            <h1>Station: {stationName}</h1>
            <h2>Slogan: {slogan}</h2>
            <h2>Message: {message}</h2>
          </div>
        )}
        {nrsc5Status == Nrsc5Status.SyncLost && (
          <>
            <span className="text-red-500">Synchronization Lost</span>
          </>
        )}
        {nrsc5Status == Nrsc5Status.Stopped && <span>SDR Not Running</span>}
        {nrsc5Status == Nrsc5Status.SdrFound && <span>Loading...</span>}
      </div>
      <div>
        <Input
          type="number"
          label="FM Frequency"
          value={freq}
          onChange={(e) => setFreq(e.target.value)}
        />
        <Input
          type="number"
          label="Channel"
          value={channel}
          onChange={(e) => setChannel(e.target.value)}
        />
        <Button
          onClick={() => {
            if (nrsc5Status == Nrsc5Status.Stopped) {
              start_nrsc5();
            } else if (nrsc5Status != Nrsc5Status.Starting) {
              stop_nrsc5();
            }
          }}
          isLoading={nrsc5Status == Nrsc5Status.Starting}
        >
          {nrsc5Status == Nrsc5Status.Stopped
            ? "Start nrsc5"
            : nrsc5Status == Nrsc5Status.Starting
            ? "Starting..."
            : "Stop nrsc5"}
        </Button>
      </div>
    </div>
  );
}
