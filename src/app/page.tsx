"use client";

import AdsbDecoderView from "@/components/AdsbDecoderView";
import RadioView from "@/components/Radio/RadioView";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Station } from "@/lib/types";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";

const appWindow = getCurrentWebviewWindow();

export default function Home() {
  const [isSdrInUse, setIsSdrInUse] = useState(false);
  const [shouldStopAdsb, setShouldStopAdsb] = useState(false);
  const [requestedStation, setRequestedStation] = useState<undefined | Station>(
    undefined
  );
  const [currentStation, setCurrentStation] = useState<undefined | Station>(
    undefined
  );
  const [requestedTab, setRequestedTab] = useState<string | undefined>(
    undefined
  );
  const [currentTab, setCurrentTab] = useState("radio");

  appWindow.listen("rtlsdr_status", (event: { payload: string }) => {
    if (event.payload.endsWith("running")) {
      setCurrentTab("radio");
    }
  });

  appWindow.listen("nrsc5_status", (event: { payload: string }) => {
    if (event.payload != "stopped") {
      setCurrentTab("radio");
    }
  });

  appWindow.listen("adsb_status", (event: { payload: string }) => {
    if (event.payload == "running") {
      setCurrentTab("adsb");
    }
  });

  useEffect(() => {
    if (requestedTab && !shouldStopAdsb) {
      setCurrentTab(requestedTab);
      setRequestedTab(undefined);
    }
  });

  return (
    <main className="flex flex-col h-screen w-screen gap-4">
      <Tabs
        value={currentTab}
        onValueChange={async (newTab) => {
          if (isSdrInUse) {
            if (currentStation) {
              setRequestedStation(undefined);
            } else {
              setShouldStopAdsb(true);
              setRequestedTab(newTab);
            }
          }
          setCurrentTab(newTab);
        }}
        className="flex flex-col justify-start items-center align-middle h-screen w-screen overflow-hidden"
      >
        <TabsList className="mt-4 mb-2">
          <TabsTrigger value="radio">Radio</TabsTrigger>
          <TabsTrigger value="adsb">ADS-B</TabsTrigger>
        </TabsList>
        <TabsContent value="radio" className="grow w-full overflow-hidden">
          <RadioView
            isSdrInUse={isSdrInUse}
            setIsSdrInUse={setIsSdrInUse}
            currentStation={currentStation}
            setCurrentStation={setCurrentStation}
            requestedStation={requestedStation}
            setRequestedStation={setRequestedStation}
          />
        </TabsContent>
        <TabsContent value="adsb" className="grow w-full overflow-hidden">
          <AdsbDecoderView
            isSdrInUse={isSdrInUse}
            setIsSdrInUse={setIsSdrInUse}
            shouldStop={shouldStopAdsb}
            setShouldStop={setShouldStopAdsb}
          />
        </TabsContent>
      </Tabs>
    </main>
  );
}
