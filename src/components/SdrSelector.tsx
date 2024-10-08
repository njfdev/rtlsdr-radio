import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";
import { AvailableSdrArgs, SDRState } from "@/lib/types";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "./ui/button";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";
import { GlobalState } from "./AppView";

const appWindow = getCurrentWebviewWindow();

export default function SdrSelector({
  globalState,
  setGlobalState,
}: {
  globalState: GlobalState;
  setGlobalState: React.Dispatch<React.SetStateAction<GlobalState>>;
}) {
  //const [sdrStates, setSDRState] = useState<SDRState[] | undefined>(undefined);
  const [selectedSdrSerial, setSelectedSdrSerial] = useState("none");

  const handleNewSdrStates = (newSdrStates: SDRState[]) => {
    // if more that 1 state,
    if (
      newSdrStates.length > 0 &&
      (globalState.sdrStates === undefined ||
        selectedSdrSerial == "none" ||
        getSdrFromSerial(selectedSdrSerial)?.dev === "InUse")
    ) {
      // check if any are in use, if so, set it as default
      const inUseSdrs = newSdrStates.filter((value) => value.dev === "InUse");
      if (inUseSdrs.length > 0) {
        setSelectedSdrSerial(inUseSdrs[0].args.serial);
        return;
      }

      // check if any are connected, if so, set it as default
      const connectedSdrs = newSdrStates.filter(
        (value) => value.dev === "Connected"
      );
      if (connectedSdrs.length > 0) {
        setSelectedSdrSerial(connectedSdrs[0].args.serial);
        return;
      }

      setSelectedSdrSerial(newSdrStates[0].args.serial);
    } else if (getSdrFromSerial(selectedSdrSerial) === undefined) {
      setSelectedSdrSerial("none");
    }
  };

  const getSdrFromSerial = (serial: string) => {
    return globalState.sdrStates?.find((state) => state.args.serial == serial);
  };

  useEffect(() => {
    const currentSdrState = getSdrFromSerial(selectedSdrSerial);
    setGlobalState((old) => ({
      ...old,
      defaultSdrArgs: currentSdrState?.args,
    }));
  }, [selectedSdrSerial]);

  useEffect(() => {
    (async () => {
      try {
        const newSdrStates = (await invoke<object>(
          "get_sdr_states",
          {}
        )) as SDRState[];

        setGlobalState((old) => ({ ...old, sdrStates: newSdrStates }));

        handleNewSdrStates(newSdrStates);
      } catch {
        console.error("Error Getting SDR States");
      }
    })();
  }, []);

  appWindow.listen("sdr_states", (event: { payload: object }) => {
    const newSdrStates = event.payload as SDRState[];
    setGlobalState((old) => ({ ...old, sdrStates: newSdrStates }));
    handleNewSdrStates(newSdrStates);
  });

  const connectToSdr = async (sdrArgs: AvailableSdrArgs) => {
    await invoke("connect_to_sdr", { args: sdrArgs });
  };

  const disconnectSdr = async (sdrArgs: AvailableSdrArgs) => {
    await invoke("disconnect_sdr", { args: sdrArgs });
  };

  return (
    <div className="flex gap-2 w-screen items-center align-middle justify-center">
      <div className="flex-1" />
      <Select
        value={selectedSdrSerial}
        onValueChange={(serial) => setSelectedSdrSerial(serial)}
      >
        <SelectTrigger className="justify-content-center w-[24rem]">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            <SelectItem value="none">Select an SDR</SelectItem>
            {globalState.sdrStates?.map((state) => {
              return (
                <SelectItem value={state.args.serial} key={state.args.serial}>
                  <div className="flex gap-2 justify-between w-full align-middle items-center">
                    <span>{state.args.label}</span>
                  </div>
                </SelectItem>
              );
            })}
          </SelectGroup>
        </SelectContent>
      </Select>
      {(() => {
        const selectedSdr = getSdrFromSerial(selectedSdrSerial);

        return (
          <div className="flex flex-1">
            {selectedSdr && (
              <Button
                onClick={() =>
                  selectedSdr.dev == "Available"
                    ? connectToSdr(selectedSdr.args)
                    : selectedSdr.dev == "Connected" &&
                      disconnectSdr(selectedSdr.args)
                }
                variant={
                  selectedSdr.dev == "Available" ? "default" : "secondary"
                }
                disabled={selectedSdr.dev == "InUse"}
                size="sm"
              >
                {selectedSdr.dev == "Connected"
                  ? "Disconnect"
                  : selectedSdr.dev == "Available"
                  ? "Connect"
                  : "In Use"}
              </Button>
            )}
          </div>
        );
      })()}
    </div>
  );
}
