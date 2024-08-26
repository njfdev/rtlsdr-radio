import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";
import { AvailableSdrArgs } from "@/lib/types";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";
import { Button } from "./ui/button";

const appWindow = getCurrentWebviewWindow();

export default function SdrSelector() {
  const [availableSdrArgs, setAvailableSdrArgs] = useState<AvailableSdrArgs[]>(
    []
  );

  useEffect(() => {
    (async () => {
      try {
        setAvailableSdrArgs(
          (await invoke<object>(
            "get_available_sdr_args",
            {}
          )) as AvailableSdrArgs[]
        );
      } catch {
        console.log("Error Getting Available SDRs");
      }
    })();
  }, []);

  appWindow.listen("available_sdrs", (event: { payload: object }) => {
    console.log(event.payload);
    setAvailableSdrArgs(event.payload as AvailableSdrArgs[]);
  });

  const connectToSdr = async (sdrArgs: AvailableSdrArgs) => {
    await invoke("connect_to_sdr", { args: sdrArgs });
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Available SDRs</CardTitle>
      </CardHeader>
      <CardContent>
        {availableSdrArgs.map((args) => {
          return (
            <div
              className="flex gap-2 align-middle items-center"
              key={args.serial}
            >
              <span>{args.label}</span>
              <Button onClick={() => connectToSdr(args)} size="sm">
                Connect
              </Button>
            </div>
          );
        })}
      </CardContent>
    </Card>
  );
}
