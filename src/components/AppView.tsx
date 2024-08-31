import AdsbDecoderView from "@/components/AdsbDecoderView";
import RadioView from "@/components/Radio/RadioView";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Station, StationType, StreamType } from "@/lib/types";
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
import BottomBar from "./BottomBar";

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
        view: () => <RadioView type={StationType.HDRadio} />,
      },
      {
        id: "fm-radio",
        name: "FM Radio",
        view: () => <RadioView type={StationType.FMRadio} />,
      },
      {
        id: "am-radio",
        name: "AM Radio",
        view: () => <RadioView type={StationType.AMRadio} />,
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
  const [currentViewId, setCurrentViewId] = useState<string>("fm-radio");

  return (
    <div className="flex flex-col w-screen h-screen">
      <ResizablePanelGroup
        direction="horizontal"
        className="flex w-screen grow"
      >
        <ResizablePanel maxSize={25} minSize={15}>
          <div className="flex flex-col gap-8 p-6 select-none">
            <MapViewData
              viewData={views}
              setCurrentViewId={setCurrentViewId}
              topLevel={true}
            />
          </div>
        </ResizablePanel>
        <ResizableHandle />
        <ResizablePanel>
          <main className="flex flex-col gap-4 h-full">
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
      <BottomBar />
    </div>
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
