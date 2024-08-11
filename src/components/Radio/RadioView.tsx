"use client";

import Nrsc5Controls from "@/components/Radio/Nrsc5Controls";
import RtlSdrControls from "@/components/Radio/RtlSdrControls";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Station, StationDetails, StationType, StreamType } from "@/lib/types";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";
import SaveStationsMenu from "@/components/Radio/SavedStationsMenu";
import { areStationsEqual } from "@/lib/stationsStorage";
import Link from "next/link";
const appWindow = getCurrentWebviewWindow();

const isNrsc5Available =
  process.env.NEXT_PUBLIC_EXCLUDE_SIDECAR == "true" ? false : true;

export default function RadioView() {
  const [openTab, setOpenTab] = useState<string>(
    isNrsc5Available
      ? StationType.HDRadio.toString()
      : StationType.FMRadio.toString()
  );
  const [requestedStation, setRequestedStation] = useState<undefined | Station>(
    undefined
  );
  const [currentStation, setCurrentStation] = useState<undefined | Station>(
    undefined
  );
  const [isSdrInUse, setIsSdrInUse] = useState(false);

  appWindow.listen("rtlsdr_status", (event: { payload: string }) => {
    console.log(event.payload);
    if (
      !event.payload.endsWith("stopped") &&
      !event.payload.endsWith("pausing")
    ) {
      if (event.payload.startsWith("fm")) {
        setOpenTab(StationType.FMRadio.toString());
      } else if (event.payload.startsWith("am")) {
        setOpenTab(StationType.AMRadio.toString());
      }
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

    // TODO: Optimize this by put in for loop
    if (requestedStation.type == StationType.HDRadio) {
      setOpenTab(StationType.HDRadio.toString());
    } else if (requestedStation.type == StationType.FMRadio) {
      setOpenTab(StationType.FMRadio.toString());
    } else if (requestedStation.type == StationType.AMRadio) {
      setOpenTab(StationType.AMRadio.toString());
    }
  }, [requestedStation, isSdrInUse, currentStation]);

  return (
    <div className="flex h-full w-full gap-4 px-4 pb-4">
      <div className="flex align-middle justify-center w-full h-full overflow-y-auto">
        <Tabs
          value={openTab}
          onValueChange={(value) => {
            setRequestedStation(undefined);
            setCurrentStation(undefined);
            setOpenTab(value as typeof openTab);
          }}
          className="flex flex-col justify-start items-center align-middle mt-4 *:pb-8 w-full"
        >
          <TabsList className="!pb-1">
            <TabsTrigger value={StationType.HDRadio.toString()}>
              HD Radio
            </TabsTrigger>
            <TabsTrigger value={StationType.FMRadio.toString()}>
              FM Radio
            </TabsTrigger>
            <TabsTrigger value={StationType.AMRadio.toString()}>
              AM Radio
            </TabsTrigger>
          </TabsList>
          <TabsContent value={StationType.HDRadio.toString()}>
            {isNrsc5Available ? (
              <Nrsc5Controls
                currentStation={currentStation}
                setCurrentStation={setCurrentStation}
                requestedStation={requestedStation}
                setRequestedStation={setRequestedStation}
                isInUse={isSdrInUse}
                setIsInUse={setIsSdrInUse}
              />
            ) : (
              <div className="max-w-[32rem] text-center my-8 text-gray-400">
                HD Radio is disabled in the precompiled version of RTL-SDR
                Radio. Please{" "}
                <Link
                  className="text-blue-400 hover:underline"
                  href="https://github.com/njfdev/rtlsdr-radio#compiling-from-source"
                  target="_blank"
                >
                  build from source
                </Link>{" "}
                to enable HD Radio features.
              </div>
            )}
          </TabsContent>
          <TabsContent value={StationType.FMRadio.toString()}>
            <RtlSdrControls
              currentStation={currentStation}
              setCurrentStation={setCurrentStation}
              requestedStation={requestedStation}
              setRequestedStation={setRequestedStation}
              isInUse={isSdrInUse}
              setIsInUse={setIsSdrInUse}
              streamType={StreamType.FM}
            />
          </TabsContent>
          <TabsContent value={StationType.AMRadio.toString()}>
            <RtlSdrControls
              currentStation={currentStation}
              setCurrentStation={setCurrentStation}
              requestedStation={requestedStation}
              setRequestedStation={setRequestedStation}
              isInUse={isSdrInUse}
              setIsInUse={setIsSdrInUse}
              streamType={StreamType.AM}
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
    </div>
  );
}
