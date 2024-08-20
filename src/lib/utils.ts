import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function titleCapitalization(input: string) {
  return input
    .split(" ")
    .map((word) => word[0].toUpperCase() + word.substring(1).toLowerCase())
    .join(" ");
}

export function snakeToTitle(snakeCase: string) {
  return titleCapitalization(snakeCase.replace(/([a-z])([A-Z])/g, "$1 $2"));
}

export function formatZipCode(zip_code: string) {
  // if the zip code is a number and is 9 characters long, we need to insert a dash
  // e.g. 284126225 -> 28412-6225
  if (!Number.isNaN(Number(zip_code)) && zip_code.length == 9) {
    return `${zip_code.substring(0, 5)}-${zip_code.substring(5)}`;
  }
  return zip_code;
}
