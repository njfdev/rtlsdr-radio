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
import { Station, StationDetails, StationType } from "@/lib/types";
import { invoke } from "@tauri-apps/api";
import { appWindow } from "@tauri-apps/api/window";
import { Loader2 } from "lucide-react";
import { Dispatch, SetStateAction, useEffect, useState } from "react";

enum RtlSdrStatus {
  Stopped = "stopped",
  Starting = "starting",
  Pausing = "pausing",
  Running = "running",
}

interface StreamSettings {
  fm_freq: number;
  volume: number;
  sample_rate: number;
}

const srStorageName = "fm_radio_sample_rate";
const volumeStorageName = "fm_radio_volume";

export default function RtlSdrControls({
  currentStation,
  setCurrentStation,
  requestedStation,
  setRequestedStation,
  isInUse,
  setIsInUse,
}: {
  currentStation: Station | undefined;
  setCurrentStation: Dispatch<SetStateAction<Station | undefined>>;
  requestedStation: Station | undefined;
  setRequestedStation: Dispatch<SetStateAction<Station | undefined>>;
  isInUse: boolean;
  setIsInUse: Dispatch<SetStateAction<boolean>>;
}) {
  const [status, setStatus] = useState(RtlSdrStatus.Stopped);
  const [streamSettings, setStreamSettings] = useState<StreamSettings>({
    fm_freq: 101.5,
    volume: parseFloat(localStorage.getItem(volumeStorageName) || "0.5"),
    sample_rate: parseFloat(localStorage.getItem(srStorageName) || "48000.0"),
  });
  const [isProcessingRequest, setIsProcessingRequest] = useState(false);

  const [isSaved, setIsSaved] = useState(
    isStationSaved({
      type: StationType.FMRadio,
      frequency: streamSettings.fm_freq,
    })
  );

  useEffect(() => {
    localStorage.setItem(volumeStorageName, streamSettings.volume.toString());
    localStorage.setItem(srStorageName, streamSettings.sample_rate.toString());
  });

  useEffect(() => {
    if (currentStation) {
      setIsSaved(isStationSaved(currentStation));
    }
  });

  useEffect(() => {
    if (isProcessingRequest) return;
    (async () => {
      if (
        requestedStation &&
        requestedStation.type == StationType.FMRadio &&
        !areStationsEqual(requestedStation, currentStation)
      ) {
        await setIsProcessingRequest(true);
        if (isInUse) {
          console.log("stopping stream");
          await stop_stream();
        }

        setIsInUse(true);
        await setStreamSettings((old) => ({
          ...old,
          fm_freq: requestedStation.frequency,
        }));
        await start_stream();
        setIsProcessingRequest(false);
      }
    })();
  });

  useEffect(() => {
    if (!requestedStation && status != RtlSdrStatus.Stopped) {
      stop_stream();
    }
  });

  const start_stream = async () => {
    setIsInUse(true);
    setStatus(RtlSdrStatus.Starting);
    await invoke<string>("start_fm_stream", {
      streamSettings,
    });
    setCurrentStation({
      type: StationType.FMRadio,
      frequency: streamSettings.fm_freq,
      channel: undefined,
    });
  };
  const stop_stream = async () => {
    setStatus(RtlSdrStatus.Pausing);
    await invoke<string>("stop_fm_stream", {});
    await setIsInUse(false);
    setCurrentStation(undefined);
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

  let firstRun = true;
  useEffect(() => {
    return () => {
      if (firstRun) {
        firstRun = false;
      } else {
        (async () => {
          await stop_stream();
        })();
      }
    };
  }, []);

  return (
    <form
      className="grid gap-3 min-w-[24rem]"
      onSubmit={(e) => {
        e.preventDefault();
        if (status == RtlSdrStatus.Stopped) {
          setRequestedStation({
            type: StationType.FMRadio,
            frequency: streamSettings.fm_freq,
          });
        }
      }}
    >
      <div className="grid w-full gap-1.5">
        <Label htmlFor="fm_freq_slider">Fm Station</Label>
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
        <Label htmlFor="volume_slider">Volume</Label>
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
      <Button
        onClick={() => {
          if (status == RtlSdrStatus.Running) {
            setRequestedStation(undefined);
          }
        }}
        disabled={
          status == RtlSdrStatus.Starting || status == RtlSdrStatus.Pausing
        }
      >
        {status == RtlSdrStatus.Running ? (
          "Stop FM Stream"
        ) : status == RtlSdrStatus.Starting ? (
          <>
            <Loader2 className="animate-spin mr-2" /> Starting...
          </>
        ) : status == RtlSdrStatus.Pausing ? (
          <>
            <Loader2 className="animate-spin mr-2" /> Stopping...
          </>
        ) : (
          "Start FM Stream"
        )}
      </Button>
      {status == RtlSdrStatus.Running && (
        <Button
          className="w-full"
          variant={isSaved ? "secondary" : "default"}
          onClick={async () => {
            let stationData: StationDetails = {
              type: StationType.FMRadio,
              title: `FM Radio: ${streamSettings.fm_freq}`,
              frequency: streamSettings.fm_freq,
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
    </form>
  );
}
