import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";
import { AvailableSdrArgs, SDRState } from "@/lib/types";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Button } from "./ui/button";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "./ui/select";

const appWindow = getCurrentWebviewWindow();

export default function SdrSelector() {
  const [sdrStates, setSDRState] = useState<SDRState[] | undefined>(undefined);
  const [selectedSdrSerial, setSelectedSdrSerial] = useState("none");

  const handleNewSdrStates = (newSdrStates: SDRState[]) => {
    // if more that 1 state,
    if (
      newSdrStates.length > 0 &&
      (sdrStates === undefined ||
        selectedSdrSerial == "none" ||
        sdrStates.find((value) => value.args.serial == selectedSdrSerial)
          ?.dev === "InUse")
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
    }
  };

  useEffect(() => {
    (async () => {
      try {
        const newSdrStates = (await invoke<object>(
          "get_sdr_states",
          {}
        )) as SDRState[];

        setSDRState(newSdrStates);

        handleNewSdrStates(newSdrStates);
      } catch {
        console.error("Error Getting SDR States");
      }
    })();
  }, []);

  appWindow.listen("sdr_states", (event: { payload: object }) => {
    const newSdrStates = event.payload as SDRState[];
    setSDRState(newSdrStates);
    handleNewSdrStates(newSdrStates);
  });

  const connectToSdr = async (sdrArgs: AvailableSdrArgs) => {
    await invoke("connect_to_sdr", { args: sdrArgs });
  };

  const disconnectSdr = async (sdrArgs: AvailableSdrArgs) => {
    await invoke("disconnect_sdr", { args: sdrArgs });
  };

  return (
    <div className="flex max-w-[36rem] mx-auto">
      {(() => {
        const selectedSdr = sdrStates?.find(
          (state) => state.args.serial == selectedSdrSerial
        );

        return (
          <Card className="my-1">
            <CardHeader>
              <CardTitle>
                {selectedSdr ? selectedSdr.args.label : "No SDR Selected"}
              </CardTitle>
            </CardHeader>
            <CardContent>
              {selectedSdr && (
                <div
                  className="flex gap-2 align-middle items-center"
                  key={selectedSdr.args.serial}
                >
                  <span>{selectedSdr.args.label}</span>
                  <Button
                    onClick={() =>
                      selectedSdr.dev == "Available"
                        ? connectToSdr(selectedSdr.args)
                        : selectedSdr.dev == "Connected" &&
                          disconnectSdr(selectedSdr.args)
                    }
                    variant={
                      selectedSdr.dev == "Connected" ? "secondary" : "default"
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
                </div>
              )}
            </CardContent>
          </Card>
        );
      })()}
      <Select
        value={selectedSdrSerial}
        onValueChange={(serial) => setSelectedSdrSerial(serial)}
      >
        <SelectTrigger>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            <SelectItem value="none">Select an SDR</SelectItem>
            {sdrStates?.map((state) => {
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
    </div>
  );
}
