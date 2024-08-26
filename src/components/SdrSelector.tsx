import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";
import { ConnectedSdrArgs } from "@/lib/types";
import { invoke } from "@tauri-apps/api/core";
import { Card, CardContent, CardHeader, CardTitle } from "./ui/card";

const appWindow = getCurrentWebviewWindow();

export default function SdrSelector() {
  const [connnectedSdrArgs, setConnectedSdrArgs] = useState<ConnectedSdrArgs[]>(
    []
  );

  useEffect(() => {
    (async () => {
      try {
        setConnectedSdrArgs(
          (await invoke<object>(
            "get_connected_sdr_args",
            {}
          )) as ConnectedSdrArgs[]
        );
      } catch {
        console.log("Error Getting Connected SDRs");
      }
    })();
  }, []);

  appWindow.listen("connected_sdrs", (event: { payload: object }) => {
    console.log(event.payload);
    setConnectedSdrArgs(event.payload as ConnectedSdrArgs[]);
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle>Connected SDRs</CardTitle>
      </CardHeader>
      <CardContent>
        {connnectedSdrArgs.map((args) => {
          return <div key={args.serial}>{args.label}</div>;
        })}
      </CardContent>
    </Card>
  );
}
