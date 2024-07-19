"use client";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { invoke } from "@tauri-apps/api";
import { appWindow } from "@tauri-apps/api/window";
import { Loader2 } from "lucide-react";
import { useState } from "react";

enum RtlSdrStatus {
  Stopped = "stopped",
  Starting = "starting",
  Pausing = "pausing",
  Idle = "idle",
  Running = "running",
}

interface StreamSettings {
  fm_freq: number;
  volume: number;
  sample_rate: number;
}

export default function SoapySdrControls() {
  const [status, setStatus] = useState(RtlSdrStatus.Stopped);
  const [streamSettings, setStreamSettings] = useState<StreamSettings>({
    fm_freq: 101.5,
    volume: 1.0,
    sample_rate: 48000.0,
  });

  const start_stream = () => {
    setStatus(RtlSdrStatus.Starting);
    invoke<string>("start_fm_stream", {
      streamSettings,
    })
      .then((_result) => console.log("FM Stream Started"))
      .catch(console.error);
  };
  const stop_stream = () => {
    setStatus(RtlSdrStatus.Pausing);
    invoke<string>("stop_fm_stream", {})
      .then((_result) => console.log("FM Stream Stopped"))
      .catch(console.error);
  };

  appWindow.listen("rtlsdr_status", (event: { payload: string }) => {
    setStatus(
      RtlSdrStatus[
        Object.keys(RtlSdrStatus)[
          Object.values(RtlSdrStatus).indexOf(event.payload as RtlSdrStatus)
        ] as keyof typeof RtlSdrStatus
      ]
    );
  });

  return (
    <div>
      <div className="grid w-full gap-1.5">
        <div>
          <Label htmlFor="fm_freq_slider">Fm Station</Label>
        </div>
        <Input
          type="number"
          step={0.2}
          min={88.1}
          max={107.9}
          placeholder="#"
          value={streamSettings.fm_freq}
          onChange={(e) =>
            setStreamSettings((old) => ({
              ...old,
              fm_freq: parseFloat(e.target.value),
            }))
          }
        />
      </div>
      <div className="grid w-full gap-1.5">
        <div>
          <Label htmlFor="audio_sr">Audio Sample Rate</Label>
        </div>
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
        <div>
          <Label htmlFor="volume_slider">Volume</Label>
          <Slider
            min={0.0}
            max={1.0}
            step={0.01}
            value={[streamSettings.volume]}
            id="volume_slider"
            onValueChange={(values) => {
              setStreamSettings((old) => ({ ...old, volume: values[0] }));
            }}
          />
        </div>
      </div>
      <Button
        onClick={() => {
          if (status == RtlSdrStatus.Stopped || status == RtlSdrStatus.Idle) {
            start_stream();
          } else if (status == RtlSdrStatus.Running) {
            stop_stream();
          }
        }}
        disabled={status == RtlSdrStatus.Starting}
      >
        {status == RtlSdrStatus.Running ? (
          "Stop FM Stream"
        ) : status == RtlSdrStatus.Starting ? (
          <>
            {" "}
            <Loader2 className="animate-spin" /> Starting...
          </>
        ) : (
          "Start FM Stream"
        )}
      </Button>
    </div>
  );
}
