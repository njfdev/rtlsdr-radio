import AdsbDecoderView from "@/components/AdsbDecoderView";
import RadioView from "@/components/Radio/RadioView";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

export default function Home() {
  return (
    <main className="flex flex-col h-screen w-screen gap-4">
      <Tabs
        defaultValue="radio"
        className="flex flex-col justify-start items-center align-middle p-4 h-screen w-screen"
      >
        <TabsList className="mt-4 mb-2">
          <TabsTrigger value="radio">Radio</TabsTrigger>
          <TabsTrigger value="adsb">ADS-B</TabsTrigger>
        </TabsList>
        <TabsContent value="radio" className="grow w-full">
          <RadioView />
        </TabsContent>
        <TabsContent value="adsb" className="grow w-full">
          <AdsbDecoderView />
        </TabsContent>
      </Tabs>
    </main>
  );
}
