"use client";

import { Button } from "@/components/ui/button";
import { invoke } from "@tauri-apps/api";

export default function SoapySdrControls() {
  const start_stream = () => {
    invoke<string>("start_fm_stream", {})
      .then((_result) => console.log("FM Stream Started"))
      .catch(console.error);
  };

  return (
    <div>
      <Button onClick={() => start_stream}>Start FM Stream</Button>
    </div>
  );
}
