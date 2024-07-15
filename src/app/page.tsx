import OsDisplay from "@/components/OsDisplay";
import Image from "next/image";

export default function Home() {
  return (
    <main className="flex h-screen w-screen flex-col items-center align-middle justify-center p-24">
      <OsDisplay />
    </main>
  );
}
