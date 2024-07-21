"use client";

import Nrsc5Controls from "@/components/Nrsc5Controls";
import RtlSdrControls from "@/components/RtlSdrControls";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Station, StationDetails, StationType } from "@/lib/types";
import { appWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import SaveStationsMenu from "@/components/SavedStationsMenu";
import { areStationsEqual } from "@/lib/stationsStorage";

export default function Home() {
  const [openTab, setOpenTab] = useState<string>(
    StationType.HDRadio.toString()
  );
  const [requestedStation, setRequestedStation] = useState<undefined | Station>(
    undefined
  );
  const [currentStation, setCurrentStation] = useState<undefined | Station>(
    undefined
  );
  const [isSdrInUse, setIsSdrInUse] = useState(false);

  appWindow.listen("rtlsdr_status", (event: { payload: string }) => {
    if (event.payload != "stopped" && event.payload != "pausing") {
      setOpenTab(StationType.FMRadio.toString());
    }
  });

  appWindow.listen("nrsc5_status", (event: { payload: string }) => {
    if (event.payload != "stopped") {
      setOpenTab(StationType.HDRadio.toString());
    }
  });

  useEffect(() => {
    if (
      isSdrInUse &&
      requestedStation &&
      !areStationsEqual(currentStation, requestedStation) &&
      openTab != requestedStation.type.toString()
    ) {
      setOpenTab("");
    }

    if (
      !requestedStation ||
      (isSdrInUse && !areStationsEqual(currentStation, requestedStation))
    )
      return;

    if (requestedStation.type == StationType.HDRadio) {
      setOpenTab(StationType.HDRadio.toString());
    } else if (requestedStation.type == StationType.FMRadio) {
      setOpenTab(StationType.FMRadio.toString());
    }
  }, [requestedStation, isSdrInUse, currentStation]);

  return (
    <main className="flex h-screen w-screen gap-4">
      <div className="flex align-middle justify-center p-12 w-full h-screen overflow-y-scroll">
        <Tabs
          value={openTab}
          onValueChange={(value) => {
            setCurrentStation(undefined);
            setOpenTab(value as typeof openTab);
          }}
          className="flex flex-col justify-start items-center align-middle mt-8"
        >
          <TabsList>
            <TabsTrigger value={StationType.HDRadio.toString()}>
              HD Radio
            </TabsTrigger>
            <TabsTrigger value={StationType.FMRadio.toString()}>
              FM Radio
            </TabsTrigger>
          </TabsList>
          <TabsContent value={StationType.HDRadio.toString()}>
            <Nrsc5Controls
              currentStation={currentStation}
              setCurrentStation={setCurrentStation}
              requestedStation={requestedStation}
              setRequestedStation={setRequestedStation}
              isInUse={isSdrInUse}
              setIsInUse={setIsSdrInUse}
            />
          </TabsContent>
          <TabsContent value={StationType.FMRadio.toString()}>
            <RtlSdrControls
              currentStation={currentStation}
              setCurrentStation={setCurrentStation}
              requestedStation={requestedStation}
              setRequestedStation={setRequestedStation}
              isInUse={isSdrInUse}
              setIsInUse={setIsSdrInUse}
            />
          </TabsContent>
        </Tabs>
      </div>
      <SaveStationsMenu
        setRequestedStation={setRequestedStation}
        currentStation={requestedStation}
        isStationPlaying={
          (currentStation &&
            requestedStation &&
            areStationsEqual(currentStation, requestedStation)) ||
          false
        }
      />
    </main>
  );
}
