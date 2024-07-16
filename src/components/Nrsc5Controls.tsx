"use client";

import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api";
import { appWindow } from "@tauri-apps/api/window";
import { Loader2 } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";

enum Nrsc5Status {
  Stopped = "stopped",
  Starting = "starting",
  SdrFound = "sdr-found",
  Synced = "synchronized",
  SyncLost = "synchronization_lost",
}

export default function Nrsc5Controls() {
  const [freq, setFreq] = useState<number>(101.5);
  const [channel, setChannel] = useState<number>(1);

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
    invoke<string>("start_nrsc5", {
      fmFreq: freq.toString(),
      channel: (channel - 1).toString(),
    })
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
    <div className="flex w-[48rem] gap-4">
      <div className="flex items-center grow basis-0 justify-center align-middle min-w-[16rem]">
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
      <div className="grid gap-2 grow basis-0 w-full">
        <div className="grid w-full gap-1.5">
          <Label htmlFor="fm_freq_slider" className="flex">
            FM Station
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
            if (nrsc5Status == Nrsc5Status.Stopped) {
              start_nrsc5();
            } else if (nrsc5Status != Nrsc5Status.Starting) {
              stop_nrsc5();
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
      </div>
    </div>
  );
}
