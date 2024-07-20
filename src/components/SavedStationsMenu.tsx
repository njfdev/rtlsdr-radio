import { Card, CardHeader, CardTitle } from "./ui/card";

export default function SavedStationsMenu() {
  return (
    <>
      <div className="max-w-[24rem] float-right w-full m-2" />
      <div className="max-w-[24rem] right-0 w-full m-2 h-[calc(100vh_-_1rem)] absolute">
        <Card className="h-full">
          <CardHeader>
            <CardTitle>Saved Stations</CardTitle>
          </CardHeader>
        </Card>
      </div>
    </>
  );
}
