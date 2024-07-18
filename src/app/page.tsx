import Nrsc5Controls from "@/components/Nrsc5Controls";
import SoapySdrControls from "@/components/SoapySdrConstrols";
import { Separator } from "@/components/ui/separator";

export default function Home() {
  return (
    <main className="flex h-screen w-screen flex-col items-center align-middle justify-center p-24 gap-4">
      <Nrsc5Controls />
      <Separator />
      <SoapySdrControls />
    </main>
  );
}
