import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useEffect, useState } from "react";
import { ConnectedSdrArgs } from "@/lib/types";
import { invoke } from "@tauri-apps/api/core";

const appWindow = getCurrentWebviewWindow();

export default function SdrSelector() {
  const [connnectedSdrArgs, setConnectedSdrArgs] = useState<ConnectedSdrArgs[]>(
    []
  );

  useEffect(() => {
    (async () => {
      setConnectedSdrArgs(
        (await invoke<object>(
          "get_connected_sdr_args",
          {}
        )) as ConnectedSdrArgs[]
      );
    })();
  }, []);

  appWindow.listen("connected_sdrs", (event: { payload: object }) => {
    setConnectedSdrArgs(event.payload as ConnectedSdrArgs[]);
  });

  return (
    <div>
      <span>Connected SDRs</span>
      {connnectedSdrArgs.map((args) => {
        return <div key={args.serial}>{args.driver}</div>;
      })}
    </div>
  );
}
