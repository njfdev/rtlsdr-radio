import { invoke } from "@tauri-apps/api";
import { Button } from "./ui/button";
import { useState } from "react";
import {
  srStorageName,
  StreamSettings,
  StreamType,
  volumeStorageName,
} from "@/lib/types";

export default function AmRadioControls() {
  const [streamSettings, setStreamSettings] = useState<StreamSettings>({
    freq: 680,
    volume: parseFloat(localStorage.getItem(volumeStorageName) || "1.0"),
    gain: 10.0,
    sample_rate: parseFloat(localStorage.getItem(srStorageName) || "48000.0"),
    stream_type: StreamType.AM,
  });

  const start_stream = async () => {
    await invoke<string>("start_stream", {
      streamSettings,
    });
  };

  return (
    <div className="flex flex-col items-center align-middle justify-center gap-4">
      <span className="text-center text-amber-300">
        RTL-SDRs often struggle with AM radio signals below 24 MHz (without an
        upconvertor), resulting in significant static. Reception quality will
        likely be poor.
      </span>
      <Button onClick={() => start_stream()}>Start AM Stream</Button>
    </div>
  );
}
