import { AvailableSdrArgs } from "@/lib/types";
import SdrSelector from "./SdrSelector";
import { Separator } from "./ui/separator";

export default function BottomBar({
  setDefaultSdrArgs,
}: {
  setDefaultSdrArgs: React.Dispatch<
    React.SetStateAction<AvailableSdrArgs | undefined>
  >;
}) {
  return (
    <div className="w-screen h-[4.5rem]">
      <Separator />
      <div className="w-screen h-full flex gap-4 items-center align-middle justify-center">
        <SdrSelector setDefaultSdrArgs={setDefaultSdrArgs} />
      </div>
    </div>
  );
}
