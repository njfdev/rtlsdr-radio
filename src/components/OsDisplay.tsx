"use client";

import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api";

export default function OsDisplay() {
  const [os, setOs] = useState("");

  useEffect(() => {
    invoke<string>("get_os_name", { name: "Next.js" })
      .then((result) => setOs(result))
      .catch(console.error);
  }, []);

  return (
    <div>
      <h1>Operating System: {os}</h1>
    </div>
  );
}
