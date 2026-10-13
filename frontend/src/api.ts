export type Place = "fridge" | "freezer" | "cupboard";

export interface NewItem {
  name: string;
  place: Place;
  qty: number;
  unit: string;
  cook_unit: string | null;
  cook_per_unit: number | null;
  expires_on: string | null;
  expiry_guessed: boolean;
}

export interface Item extends NewItem {
  id: number;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, { headers: { "content-type": "application/json" }, ...init });
  if (!res.ok) throw new Error((await res.text()) || res.statusText);
  return res.json() as Promise<T>;
}

export const listItems = () => request<Item[]>("/api/items");

export const parseText = (text: string) =>
  request<{ items: NewItem[] }>("/api/items/parse", { method: "POST", body: JSON.stringify({ text }) });

export const addItems = (items: NewItem[], source: "text" | "manual") =>
  request<number[]>("/api/items", { method: "POST", body: JSON.stringify({ items, source }) });
