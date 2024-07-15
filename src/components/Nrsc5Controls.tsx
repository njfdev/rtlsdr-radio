"use client";

import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api";
import { appWindow } from "@tauri-apps/api/window";

export default function Nrsc5Controls() {
  const [isPlaying, setIsPlaying] = useState(false);
  const [messages, setMessages] = useState("");

  useEffect(() => {
    invoke<string>("start_nrsc5", { name: "Next.js" })
      .then((_result) => setIsPlaying(true))
      .catch(console.error);
  }, []);

  appWindow.listen("message", (event) => {
    setMessages(messages + "\n" + event.payload);
  });

  return (
    <div>
      <h1>Is nrsc5 Playing? {isPlaying}</h1>
      <div>{messages}</div>
    </div>
  );
}
