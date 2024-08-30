import AdsbDecoderView from "@/components/AdsbDecoderView";
import RadioView from "@/components/Radio/RadioView";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Station, StreamType } from "@/lib/types";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { ReactNode, useEffect, useState } from "react";
import SdrSelector from "./SdrSelector";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "./ui/resizable";
import { Button } from "./ui/button";
import Nrsc5Controls from "./Radio/Nrsc5Controls";
import RtlSdrControls from "./Radio/RtlSdrControls";

const appWindow = getCurrentWebviewWindow();

interface ViewData {
  id: string;
  name: string;
  subviews?: ViewData[];
  view?: () => any;
}

const views: ViewData[] = [
  {
    id: "radio",
    name: "Radio",
    subviews: [
      {
        id: "hd-radio",
        name: "HD Radio",
        view: () => <RadioView type="hd" />,
      },
      {
        id: "fm-radio",
        name: "FM Radio",
        view: () => <RadioView type="fm" />,
      },
      {
        id: "am-radio",
        name: "AM Radio",
        view: () => <RadioView type="am" />,
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

function GetViewById(
  id: string,
  current_views: ViewData[] = views
): ViewData | null {
  for (const view of current_views) {
    if (view.id == id) {
      return view;
    }

    if (view.subviews) {
      const result = GetViewById(id, view.subviews);

      if (result) {
        return result;
      }
    }
  }

  return null;
}

export default function AppView() {
  const [currentViewId, setCurrentViewId] = useState<string>(views[0].id);

  return (
    <ResizablePanelGroup
      direction="horizontal"
      className="flex gap-4 h-screen w-screen"
    >
      <ResizablePanel maxSize={20} minSize={15}>
        <div className="flex flex-col gap-8 p-6">
          <MapViewData
            viewData={views}
            setCurrentViewId={setCurrentViewId}
            topLevel={true}
          />
        </div>
      </ResizablePanel>
      <ResizableHandle />
      <ResizablePanel>
        <main className="flex flex-col gap-4 h-screen">
          <SdrSelector />
          {(() => {
            const view = GetViewById(currentViewId);

            if (view?.view) {
              return <view.view />;
            }

            return <></>;
          })()}
        </main>
      </ResizablePanel>
    </ResizablePanelGroup>
  );
}

function MapViewData({
  viewData,
  setCurrentViewId,
  topLevel = false,
}: {
  viewData: ViewData[];
  setCurrentViewId: any;
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
                  <MapViewData
                    viewData={currentViewData.subviews}
                    setCurrentViewId={setCurrentViewId}
                  />
                )}
              </>
            ) : (
              <Button
                size="sm"
                variant="ghost"
                className="w-full justify-start -mx-[0.625rem]"
                onClick={() => {
                  setCurrentViewId(currentViewData.id);
                }}
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
