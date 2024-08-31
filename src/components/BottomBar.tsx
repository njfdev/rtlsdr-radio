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
    <div className="w-screen">
      <Separator />
      <SdrSelector setDefaultSdrArgs={setDefaultSdrArgs} />
    </div>
  );
}
