"use client";

import { invoke } from "@tauri-apps/api/core";
import { AdsbDecodeSettings } from "@/lib/types";
import { Button } from "./ui/button";

export default function AdsbDecoderView() {
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

  return (
    <div>
      <h1>ADS-B Decoder</h1>
      <Button onClick={() => start_decoding()}>Start Decoding</Button>
      <Button onClick={() => stop_decoding()}>Stop Decoding</Button>
    </div>
  );
}
