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
}

export default function Nrsc5Controls() {
  const [freq, setFreq] = useState<string | undefined>();
  const [channel, setChannel] = useState<string | undefined>();

  const [nrsc5Status, setNrsc5Status] = useState(Nrsc5Status.Stopped);

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

  return (
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
          } else if (nrsc5Status == Nrsc5Status.Synced) {
            stop_nrsc5();
          }
        }}
        isLoading={
          nrsc5Status == Nrsc5Status.SdrFound ||
          nrsc5Status == Nrsc5Status.Starting
        }
      >
        {nrsc5Status == Nrsc5Status.Stopped
          ? "Start nrsc5"
          : nrsc5Status == Nrsc5Status.Synced
          ? "Stop nrsc5"
          : nrsc5Status == Nrsc5Status.Starting
          ? "Starting..."
          : "Loading..."}
      </Button>
    </div>
  );
}
