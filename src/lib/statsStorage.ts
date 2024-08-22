const STATS_STORAGE_NAME = "stats";

export async function increaseListeningDuration(
  seconds_listened: number
): Promise<number> {
  const prev_seconds = await getSecondsListenedTo();

  const new_seconds = prev_seconds + seconds_listened;

  await localStorage.setItem(STATS_STORAGE_NAME, new_seconds.toString());

  return new_seconds;
}

export async function getSecondsListenedTo(): Promise<number> {
  const current_seconds_raw = localStorage.getItem(STATS_STORAGE_NAME);

  if (current_seconds_raw === null) {
    localStorage.setItem(STATS_STORAGE_NAME, "0");
    return 0;
  }

  const current_seconds = Number(current_seconds_raw);

  return isNaN(current_seconds) ? 0 : current_seconds;
}
