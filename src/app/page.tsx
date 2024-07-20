"use client";

import Nrsc5Controls from "@/components/Nrsc5Controls";
import RtlSdrControls from "@/components/RtlSdrControls";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Station, StationType } from "@/lib/types";
import { appWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import SaveStationsMenu from "@/components/SavedStationsMenu";

export default function Home() {
  const [openTab, setOpenTab] = useState<"hd-radio" | "fm-radio" | "">(
    "hd-radio"
  );
  const [requestedStation, setRequestedStation] = useState<undefined | Station>(
    undefined
  );
  const [isSdrInUse, setIsSdrInUse] = useState(false);

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

  useEffect(() => {
    if (isSdrInUse && requestedStation) {
      setOpenTab("");
    }

    if (!requestedStation) {
    } else if (requestedStation.type == StationType.HDRadio) {
      setOpenTab("hd-radio");
    } else if (requestedStation.type == StationType.FMRadio) {
      setOpenTab("fm-radio");
    }
  }, [requestedStation, isSdrInUse]);

  return (
    <main className="flex h-screen w-screen gap-4">
      <div className="flex align-middle justify-center p-12 w-full h-screen overflow-y-scroll">
        <Tabs
          value={openTab}
          onValueChange={(value) => {
            setOpenTab(value as typeof openTab);
            setRequestedStation(undefined);
          }}
          className="flex flex-col justify-start items-center align-middle mt-8"
        >
          <TabsList>
            <TabsTrigger value="hd-radio">HD Radio</TabsTrigger>
            <TabsTrigger value="fm-radio">FM Radio</TabsTrigger>
          </TabsList>
          <TabsContent value="hd-radio">
            {requestedStation?.type == StationType.HDRadio ? (
              <Nrsc5Controls
                initialStation={requestedStation}
                autoPlay={true}
                setIsInUse={setIsSdrInUse}
              />
            ) : (
              <Nrsc5Controls setIsInUse={setIsSdrInUse} />
            )}
          </TabsContent>
          <TabsContent value="fm-radio">
            {requestedStation?.type == StationType.FMRadio ? (
              <RtlSdrControls
                initialStation={requestedStation}
                autoPlay={true}
                setIsInUse={setIsSdrInUse}
              />
            ) : (
              <RtlSdrControls setIsInUse={setIsSdrInUse} />
            )}
          </TabsContent>
        </Tabs>
      </div>
      <SaveStationsMenu setRequestedStation={setRequestedStation} />
    </main>
  );
}
