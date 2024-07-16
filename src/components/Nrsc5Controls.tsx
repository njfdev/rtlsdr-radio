"use client";

import React, { useState } from "react";
import { invoke } from "@tauri-apps/api";
import { appWindow } from "@tauri-apps/api/window";
import { Loader2 } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

enum Nrsc5Status {
  Stopped = "stopped",
  Starting = "starting",
  SdrFound = "sdr-found",
  Synced = "synchronized",
  SyncLost = "synchronization_lost",
}

interface StreamDetails {
  songTitle?: string;
  songArtist?: string;
  stationName?: string;
  slogan?: string;
  message?: string;
  audioBitRate?: number;
  bitErrorRate?: number;
}

export default function Nrsc5Controls() {
  const [freq, setFreq] = useState<number>(101.5);
  const [channel, setChannel] = useState<number>(1);

  const [nrsc5Status, setNrsc5Status] = useState(Nrsc5Status.Stopped);
  const [streamDetails, setStreamDetails] = useState<StreamDetails>({});

  const start_nrsc5 = () => {
    setNrsc5Status(Nrsc5Status.Starting);
    invoke<string>("start_nrsc5", {
      fmFreq: freq.toString(),
      channel: (channel - 1).toString(),
    }).catch(console.error);
  };
  const stop_nrsc5 = () => {
    invoke<string>("stop_nrsc5", {})
      .then((_result) => setStreamDetails({}))
      .catch(console.error);
  };

  appWindow.listen("nrsc5_status", (event: { payload: string }) => {
    setNrsc5Status(
      Nrsc5Status[
        Object.keys(Nrsc5Status)[
          Object.values(Nrsc5Status).indexOf(event.payload as Nrsc5Status)
        ] as keyof typeof Nrsc5Status
      ]
    );
  });

  appWindow.listen("nrsc5_title", (event: { payload: string }) => {
    setStreamDetails((old) => ({ ...old, songTitle: event.payload }));
  });
  appWindow.listen("nrsc5_artist", (event: { payload: string }) => {
    setStreamDetails((old) => ({ ...old, songArtist: event.payload }));
  });
  appWindow.listen("nrsc5_br", (event: { payload: string }) => {
    setStreamDetails((old) => ({
      ...old,
      audioBitRate: parseFloat(event.payload),
    }));
  });
  appWindow.listen("nrsc5_station", (event: { payload: string }) => {
    setStreamDetails((old) => ({ ...old, stationName: event.payload }));
  });
  appWindow.listen("nrsc5_slogan", (event: { payload: string }) => {
    setStreamDetails((old) => ({ ...old, slogan: event.payload }));
  });
  appWindow.listen("nrsc5_message", (event: { payload: string }) => {
    setStreamDetails((old) => ({ ...old, message: event.payload }));
  });
  appWindow.listen("nrsc5_ber", (event: { payload: string }) => {
    setStreamDetails((old) => ({
      ...old,
      bitErrorRate: parseFloat(event.payload),
    }));
  });

  return (
    <div className="flex w-[48rem] gap-4">
      <div className="flex items-center grow basis-0 justify-center align-middle min-w-[16rem] w-full">
        <div className="grid grid-cols-1 grid-rows-1 relative w-full">
          <Tabs
            defaultValue="radioInfo"
            className={`w-full ${
              nrsc5Status != Nrsc5Status.Synced ? "opacity-75" : ""
            }`}
            style={{ gridColumn: 0, gridRow: 0 }}
          >
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger
                disabled={nrsc5Status != Nrsc5Status.Synced}
                value="radioInfo"
              >
                Radio Info
              </TabsTrigger>
              <TabsTrigger
                disabled={nrsc5Status != Nrsc5Status.Synced}
                value="stationInfo"
              >
                Station Info
              </TabsTrigger>
            </TabsList>
            <TabsContent value="radioInfo">
              <Card
                className={`
                  ${
                    nrsc5Status == Nrsc5Status.SyncLost ? "*:blur-sm" : ""
                  } *:transition-all`}
              >
                <CardHeader>
                  <CardTitle>{streamDetails.songTitle}</CardTitle>
                  <CardDescription>{streamDetails.songArtist}</CardDescription>
                </CardHeader>
                <CardContent>
                  {streamDetails.audioBitRate != undefined && (
                    <Badge
                      variant="outline"
                      className={`before:content-[''] before:inline-block before:w-2 before:h-2 before:${
                        streamDetails.audioBitRate > 64
                          ? "bg-green-500"
                          : streamDetails.audioBitRate > 32
                          ? "bg-orange-500"
                          : "bg-red-500"
                      } before:rounded-full before:mr-2`}
                    >
                      {streamDetails.audioBitRate}kbps
                    </Badge>
                  )}
                  {streamDetails.bitErrorRate != undefined && (
                    <Badge
                      variant="outline"
                      className={`before:content-[''] before:inline-block before:w-2 before:h-2 before:${
                        streamDetails.bitErrorRate < 0.0075
                          ? "bg-green-500"
                          : streamDetails.bitErrorRate < 0.05
                          ? "bg-orange-500"
                          : "bg-red-500"
                      } before:rounded-full before:mr-2`}
                    >
                      {Math.floor(streamDetails.bitErrorRate * 100_00) / 100}%
                      BER
                    </Badge>
                  )}
                </CardContent>
              </Card>
            </TabsContent>
            <TabsContent value="stationInfo">
              <Card>
                <CardHeader>
                  <CardTitle>{streamDetails.stationName}</CardTitle>
                  <CardDescription>{streamDetails.slogan}</CardDescription>
                </CardHeader>
                {streamDetails.message &&
                  streamDetails.message.trim().length > 0 && (
                    <CardContent>
                      <h2 className="text-lg font-bold">Station Message</h2>
                      <span className="text-sm">{streamDetails.message}</span>
                    </CardContent>
                  )}
              </Card>
            </TabsContent>
          </Tabs>
          {nrsc5Status == Nrsc5Status.SyncLost && (
            <div
              className="absolute w-full h-full flex align-middle items-center justify-center"
              style={{ gridColumn: 0, gridRow: 0 }}
            >
              <span className="text-red-500 bg-black rounded-sm border-stone-900 border-[2px] px-4 py-2">
                Synchronization Lost
              </span>
            </div>
          )}
        </div>
        {nrsc5Status == Nrsc5Status.Stopped && <span>SDR Not Running</span>}
        {nrsc5Status == Nrsc5Status.SdrFound && <span>Loading...</span>}
      </div>
      <div className="grid gap-2 grow basis-0 w-full">
        <div className="grid w-full gap-1.5">
          <Label htmlFor="fm_freq_slider" className="flex">
            FM Station
          </Label>
          <Input
            id="fm_freq_slider"
            type="number"
            step={0.2}
            min={88.1}
            max={107.9}
            value={freq}
            onChange={(e) => setFreq(parseFloat(e.target.value))}
          />
        </div>
        <div className="grid w-full gap-1.5">
          <div>
            <Label htmlFor="fm_freq_slider">Station Channel</Label>
          </div>
          <Input
            type="number"
            step={1}
            min={1}
            max={4}
            placeholder="#"
            value={channel}
            onChange={(e) => setChannel(parseInt(e.target.value))}
          />
        </div>
        <Button
          onClick={() => {
            if (nrsc5Status == Nrsc5Status.Stopped) {
              start_nrsc5();
            } else if (nrsc5Status != Nrsc5Status.Starting) {
              stop_nrsc5();
            }
          }}
          disabled={nrsc5Status == Nrsc5Status.Starting}
        >
          {nrsc5Status == Nrsc5Status.Starting && (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          )}
          {nrsc5Status == Nrsc5Status.Stopped
            ? "Start nrsc5"
            : nrsc5Status == Nrsc5Status.Starting
            ? "Starting..."
            : "Stop nrsc5"}
        </Button>
      </div>
    </div>
  );
}
