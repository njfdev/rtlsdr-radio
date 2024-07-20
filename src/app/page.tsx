import Nrsc5Controls from "@/components/Nrsc5Controls";
import SoapySdrControls from "@/components/SoapySdrConstrols";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

export default function Home() {
  return (
    <main className="flex h-screen w-screen flex-col items-center align-middle justify-center p-24 gap-4">
      <Tabs
        defaultValue="hd-radio"
        className="flex flex-col justify-start items-center align-middle h-[18rem]"
      >
        <TabsList>
          <TabsTrigger value="hd-radio">HD Radio</TabsTrigger>
          <TabsTrigger value="fm-radio">FM Radio</TabsTrigger>
        </TabsList>
        <TabsContent value="hd-radio">
          <Nrsc5Controls />
        </TabsContent>
        <TabsContent value="fm-radio">
          <SoapySdrControls />
        </TabsContent>
      </Tabs>
    </main>
  );
}
