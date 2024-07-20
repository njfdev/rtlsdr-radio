"use client";

import Nrsc5Controls from "@/components/Nrsc5Controls";
import RtlSdrControls from "@/components/RtlSdrControls";
import SavedStationsMenu from "@/components/SavedStationsMenu";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { appWindow } from "@tauri-apps/api/window";
import { useState } from "react";

export default function Home() {
  const [openTab, setOpenTab] = useState<"hd-radio" | "fm-radio">("hd-radio");

  appWindow.listen("rtlsdr_status", (event: { payload: string }) => {
    if (event.payload != "stopped" && event.payload != "pausing") {
      setOpenTab("fm-radio");
    }
  });

  appWindow.listen("nrsc5_status", (event: { payload: string }) => {
    if (event.payload != "stopped") {
      setOpenTab("hd-radio");
    }
  });

  return (
    <main className="flex h-screen w-screen gap-4">
      <div className="flex align-middle justify-center p-12 w-full h-screen overflow-y-scroll">
        <Tabs
          value={openTab}
          onValueChange={(value) => {
            setOpenTab(value as typeof openTab);
          }}
          className="flex flex-col justify-start items-center align-middle mt-8"
        >
          <TabsList>
            <TabsTrigger value="hd-radio">HD Radio</TabsTrigger>
            <TabsTrigger value="fm-radio">FM Radio</TabsTrigger>
          </TabsList>
          <TabsContent value="hd-radio">
            <Nrsc5Controls />
          </TabsContent>
          <TabsContent value="fm-radio">
            <RtlSdrControls />
          </TabsContent>
        </Tabs>
      </div>
      <SavedStationsMenu />
    </main>
  );
}
