"use client";

import Nrsc5Controls from "@/components/Radio/Nrsc5Controls";
import RtlSdrControls from "@/components/Radio/RtlSdrControls";
import { Station, StationType, StreamType } from "@/lib/types";
import { useState } from "react";
import SaveStationsMenu from "@/components/Radio/SavedStationsMenu";
import { areStationsEqual } from "@/lib/stationsStorage";
import { GlobalState } from "../AppView";

const isNrsc5Available =
  import.meta.env.VITE_EXCLUDE_SIDECAR == "true" ? false : true;

export default function RadioView({
  type,
  globalState,
  setGlobalState,
}: {
  type: StationType;
  globalState: GlobalState;
  setGlobalState: React.Dispatch<React.SetStateAction<GlobalState>>;
}) {
  const [requestedStation, setRequestedStation] = useState<
    undefined | null | Station
  >(undefined);
  const [currentStation, setCurrentStation] = useState<undefined | Station>(
    undefined
  );

  return (
    <div className="flex h-full w-full gap-4 p-4">
      <div className="flex align-middle justify-center w-full h-full overflow-y-auto">
        {type == StationType.HDRadio ? (
          isNrsc5Available ? (
            <Nrsc5Controls
              currentStation={currentStation}
              setCurrentStation={setCurrentStation}
              requestedStation={requestedStation}
              setRequestedStation={setRequestedStation}
            />
          ) : (
            <div className="max-w-[32rem] text-center my-8 text-gray-400">
              HD Radio is disabled in the precompiled version of RTL-SDR Radio.
              Please{" "}
              <a
                className="text-blue-400 hover:underline"
                href="https://github.com/njfdev/rtlsdr-radio#compiling-from-source"
                target="_blank"
              >
                build from source
              </a>{" "}
              to enable HD Radio features.
            </div>
          )
        ) : type == StationType.FMRadio ? (
          <RtlSdrControls
            currentStation={currentStation}
            setCurrentStation={setCurrentStation}
            requestedStation={requestedStation}
            setRequestedStation={setRequestedStation}
            streamType={StreamType.FM}
            globalState={globalState}
            setGlobalState={setGlobalState}
          />
        ) : (
          <RtlSdrControls
            currentStation={currentStation}
            setCurrentStation={setCurrentStation}
            requestedStation={requestedStation}
            setRequestedStation={setRequestedStation}
            streamType={StreamType.AM}
            globalState={globalState}
            setGlobalState={setGlobalState}
          />
        )}
      </div>
      <SaveStationsMenu
        setRequestedStation={setRequestedStation}
        currentStation={currentStation}
        isStationPlaying={
          (currentStation &&
            requestedStation &&
            areStationsEqual(currentStation, requestedStation)) ||
          false
        }
        stationType={type}
      />
    </div>
  );
}
