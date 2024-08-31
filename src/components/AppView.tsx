import AdsbDecoderView from "@/components/AdsbDecoderView";
import RadioView from "@/components/Radio/RadioView";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  AvailableSdrArgs,
  RbdsData,
  Station,
  StationType,
  StreamType,
} from "@/lib/types";
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
  view?: (props: {
    globalState: GlobalState;
    setGlobalState: React.Dispatch<React.SetStateAction<GlobalState>>;
  }) => ReactNode;
}

const views: ViewData[] = [
  {
    id: "radio",
    name: "Radio",
    subviews: [
      {
        id: "hd-radio",
        name: "HD Radio",
        view: (props) => <RadioView type={StationType.HDRadio} {...props} />,
      },
      {
        id: "fm-radio",
        name: "FM Radio",
        view: (props) => <RadioView type={StationType.FMRadio} {...props} />,
      },
      {
        id: "am-radio",
        name: "AM Radio",
        view: (props) => <RadioView type={StationType.AMRadio} {...props} />,
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
        view: (props) => <AdsbDecoderView {...props} />,
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

export interface GlobalState {
  rbdsData: RbdsData;
  defaultSdrArgs: AvailableSdrArgs | undefined;
}

export default function AppView() {
  const [currentViewId, setCurrentViewId] = useState<string>("fm-radio");
  const [globalState, setGlobalState] = useState<GlobalState>({
    rbdsData: {} as RbdsData,
    defaultSdrArgs: undefined,
  } as GlobalState);

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
              currentViewId={currentViewId}
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
                return (
                  <view.view
                    globalState={globalState}
                    setGlobalState={setGlobalState}
                  />
                );
              }

              return <></>;
            })()}
          </main>
        </ResizablePanel>
      </ResizablePanelGroup>
      <BottomBar globalState={globalState} setGlobalState={setGlobalState} />
    </div>
  );
}

function MapViewData({
  viewData,
  setCurrentViewId,
  currentViewId,
  topLevel = false,
}: {
  viewData: ViewData[];
  setCurrentViewId: any;
  currentViewId: string;
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
                    currentViewId={currentViewId}
                  />
                )}
              </>
            ) : (
              <Button
                size="sm"
                variant="ghost"
                className={`w-full justify-start -mx-[0.625rem] ${
                  currentViewId === currentViewData.id ? "font-bold" : ""
                }`}
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
