"use client";

import { invoke } from "@tauri-apps/api/core";
import { AdsbDecodeSettings, ModesState } from "@/lib/types";
import { Button } from "./ui/button";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
const appWindow = getCurrentWebviewWindow();

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

  appWindow.listen("modes_state", (event: { payload: any }) => {
    console.log(event.payload as ModesState);
  });

  return (
    <div>
      <h1>ADS-B Decoder</h1>
      <Button onClick={() => start_decoding()}>Start Decoding</Button>
      <Button onClick={() => stop_decoding()}>Stop Decoding</Button>
    </div>
  );
}
