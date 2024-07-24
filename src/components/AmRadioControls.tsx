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
    volume: parseFloat(localStorage.getItem(volumeStorageName) || "0.5"),
    sample_rate: parseFloat(localStorage.getItem(srStorageName) || "48000.0"),
    stream_type: StreamType.AM,
  });

  const start_stream = async () => {
    await invoke<string>("start_stream", {
      streamSettings,
    });
  };

  return (
    <div>
      <Button onClick={() => start_stream()}>Start AM Stream</Button>
    </div>
  );
}
