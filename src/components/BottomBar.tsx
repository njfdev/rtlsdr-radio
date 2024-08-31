import SdrSelector from "./SdrSelector";
import { Separator } from "./ui/separator";

export default function BottomBar() {
  return (
    <div className="w-screen">
      <Separator />
      <SdrSelector />
    </div>
  );
}
