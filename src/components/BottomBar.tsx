import SdrSelector from "./SdrSelector";
import { Separator } from "./ui/separator";
import { GlobalState } from "./AppView";

export default function BottomBar({
  globalState,
  setGlobalState,
}: {
  globalState: GlobalState;
  setGlobalState: React.Dispatch<React.SetStateAction<GlobalState>>;
}) {
  return (
    <div className="w-screen h-[3.5rem]">
      <Separator />
      <div className="w-screen h-full flex gap-4 items-center align-middle justify-center">
        <SdrSelector
          globalState={globalState}
          setGlobalState={setGlobalState}
        />
      </div>
    </div>
  );
}
