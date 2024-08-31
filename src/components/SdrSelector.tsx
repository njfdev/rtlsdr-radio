import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";
import { AvailableSdrArgs, SDRState } from "@/lib/types";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Button } from "./ui/button";

const appWindow = getCurrentWebviewWindow();

export default function SdrSelector() {
  const [sdrStates, setSDRState] = useState<SDRState[]>([]);

  useEffect(() => {
    (async () => {
      try {
        setSDRState((await invoke<object>("get_sdr_states", {})) as SDRState[]);
      } catch {
        console.error("Error Getting SDR States");
      }
    })();
  }, []);

  appWindow.listen("sdr_states", (event: { payload: object }) => {
    console.log(event.payload, typeof event.payload);
    setSDRState(event.payload as SDRState[]);
  });

  const connectToSdr = async (sdrArgs: AvailableSdrArgs) => {
    await invoke("connect_to_sdr", { args: sdrArgs });
  };

  const disconnectSdr = async (sdrArgs: AvailableSdrArgs) => {
    await invoke("disconnect_sdr", { args: sdrArgs });
  };

  return (
    <Card className="max-w-[36rem] mx-auto my-1">
      <CardHeader>
        <CardTitle>Available SDRs</CardTitle>
      </CardHeader>
      <CardContent>
        {sdrStates.map((state) => {
          return (
            <div
              className="flex gap-2 align-middle items-center"
              key={state.args.serial}
            >
              <span>{state.args.label}</span>
              <Button
                onClick={() =>
                  state.dev == "Available"
                    ? connectToSdr(state.args)
                    : state.dev == "Connected" && disconnectSdr(state.args)
                }
                variant={state.dev == "Connected" ? "secondary" : "default"}
                disabled={state.dev == "InUse"}
                size="sm"
              >
                {state.dev == "Connected"
                  ? "Disconnect"
                  : state.dev == "Available"
                  ? "Connect"
                  : "In Use"}
              </Button>
            </div>
          );
        })}
      </CardContent>
    </Card>
  );
}
