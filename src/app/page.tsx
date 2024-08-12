"use client";

import AdsbDecoderView from "@/components/AdsbDecoderView";
import RadioView from "@/components/Radio/RadioView";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";

const appWindow = getCurrentWebviewWindow();

export default function Home() {
  const [isSdrInUse, setIsSdrInUse] = useState(false);
  const [isChangingTabWhileInUse, setIsChangingTabWhileInUse] = useState(false);
  const [currentTab, setCurrentTab] = useState("radio");

  appWindow.listen("rtlsdr_status", (event: { payload: string }) => {
    if (event.payload.endsWith("running") && !isChangingTabWhileInUse) {
      setCurrentTab("radio");
    } else if (event.payload.endsWith("stopped") && isChangingTabWhileInUse) {
      setIsChangingTabWhileInUse(false);
    }
  });

  appWindow.listen("nrsc5_status", (event: { payload: string }) => {
    if (event.payload != "stopped" && !isChangingTabWhileInUse) {
      setCurrentTab("radio");
    } else if (event.payload == "stopped" && isChangingTabWhileInUse) {
      setIsChangingTabWhileInUse(false);
    }
  });

  appWindow.listen("adsb_status", (event: { payload: string }) => {
    if (event.payload == "running" && !isChangingTabWhileInUse) {
      setCurrentTab("adsb");
    } else if (event.payload == "stopped" && isChangingTabWhileInUse) {
      setIsChangingTabWhileInUse(false);
    }
  });

  return (
    <main className="flex flex-col h-screen w-screen gap-4">
      <Tabs
        value={currentTab}
        onValueChange={async (newTab) => {
          if (isSdrInUse) {
            await setIsChangingTabWhileInUse(true);
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
          <RadioView isSdrInUse={isSdrInUse} setIsSdrInUse={setIsSdrInUse} />
        </TabsContent>
        <TabsContent value="adsb" className="grow w-full overflow-hidden">
          <AdsbDecoderView />
        </TabsContent>
      </Tabs>
    </main>
  );
}
