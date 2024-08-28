import AdsbDecoderView from "@/components/AdsbDecoderView";
import RadioView from "@/components/Radio/RadioView";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Station } from "@/lib/types";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ReactNode, useEffect, useState } from "react";
import SdrSelector from "./SdrSelector";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "./ui/resizable";
import { Button } from "./ui/button";

const appWindow = getCurrentWebviewWindow();

interface ViewData {
  id: string;
  name: string;
  subviews?: ViewData[];
  view?: any;
}

const views: ViewData[] = [
  {
    id: "radio",
    name: "Radio",
    subviews: [
      {
        id: "hd-radio",
        name: "HD Radio",
        view: RadioView,
      },
      {
        id: "fm-radio",
        name: "FM Radio",
        view: RadioView,
      },
      {
        id: "am-radio",
        name: "AM Radio",
        view: RadioView,
      },
    ],
  },
  {
    id: "aircraft",
    name: "Aircraft",
    subviews: [
      {
        id: "ads-b",
        name: "ADS-B",
        view: AdsbDecoderView,
      },
    ],
  },
];

export default function AppView() {
  const [currentViewId, setCurrentViewId] = useState<ReactNode>(views[0].id);

  const [isSdrInUse, setIsSdrInUse] = useState(false);
  const [shouldStopAdsb, setShouldStopAdsb] = useState(false);
  const [requestedStation, setRequestedStation] = useState<undefined | Station>(
    undefined
  );
  const [currentStation, setCurrentStation] = useState<undefined | Station>(
    undefined
  );
  const [requestedTab, setRequestedTab] = useState<string | undefined>(
    undefined
  );
  const [currentTab, setCurrentTab] = useState("radio");

  appWindow.listen("rtlsdr_status", (event: { payload: string }) => {
    if (event.payload.endsWith("running")) {
      setCurrentTab("radio");
    }
  });

  appWindow.listen("nrsc5_status", (event: { payload: string }) => {
    if (event.payload != "stopped") {
      setCurrentTab("radio");
    }
  });

  appWindow.listen("adsb_status", (event: { payload: string }) => {
    if (event.payload == "running") {
      setCurrentTab("adsb");
    }
  });

  useEffect(() => {
    if (requestedTab && !shouldStopAdsb) {
      setCurrentTab(requestedTab);
      setRequestedTab(undefined);
    }
  });

  return (
    <ResizablePanelGroup
      direction="horizontal"
      className="flex gap-4 h-screen w-screen"
    >
      <ResizablePanel maxSize={20} minSize={15}>
        <div className="flex flex-col gap-8 p-6">
          <MapViewData viewData={views} topLevel={true} />
        </div>
      </ResizablePanel>
      <ResizableHandle />
      <ResizablePanel>
        <main className="flex flex-col gap-4">
          <SdrSelector />
          <Tabs
            value={currentTab}
            onValueChange={async (newTab) => {
              if (isSdrInUse) {
                if (currentStation) {
                  setRequestedStation(undefined);
                } else {
                  setShouldStopAdsb(true);
                  setRequestedTab(newTab);
                }
              }
              setCurrentTab(newTab);
            }}
            className="flex flex-col justify-start items-center align-middle h-screen w-screen overflow-hidden"
          >
            <TabsList className="mb-2">
              <TabsTrigger value="radio">Radio</TabsTrigger>
              <TabsTrigger value="adsb">ADS-B</TabsTrigger>
            </TabsList>
            <TabsContent value="radio" className="grow w-full overflow-hidden">
              <RadioView
                isSdrInUse={isSdrInUse}
                setIsSdrInUse={setIsSdrInUse}
                currentStation={currentStation}
                setCurrentStation={setCurrentStation}
                requestedStation={requestedStation}
                setRequestedStation={setRequestedStation}
              />
            </TabsContent>
            <TabsContent value="adsb" className="grow w-full overflow-hidden">
              <AdsbDecoderView
                //isSdrInUse={isSdrInUse}
                setIsSdrInUse={setIsSdrInUse}
                shouldStop={shouldStopAdsb}
                setShouldStop={setShouldStopAdsb}
              />
            </TabsContent>
          </Tabs>
        </main>
      </ResizablePanel>
    </ResizablePanelGroup>
  );
}

function MapViewData({
  viewData,
  topLevel = false,
}: {
  viewData: ViewData[];
  topLevel?: boolean;
}) {
  return (
    <div className={`flex flex-col ${topLevel ? "gap-8" : "gap-0"}`}>
      {viewData.map((currentViewData) => {
        return (
          <div key={currentViewData.name}>
            {topLevel ? (
              <>
                <h2 className="font-bold text-2xl mb-2">
                  {currentViewData.name}
                </h2>
                {currentViewData.subviews && (
                  <MapViewData viewData={currentViewData.subviews} />
                )}
              </>
            ) : (
              <Button
                size="sm"
                variant="ghost"
                className="w-full justify-start -mx-[0.625rem]"
              >
                {currentViewData.name}
              </Button>
            )}
          </div>
        );
      })}
    </div>
  );
}
