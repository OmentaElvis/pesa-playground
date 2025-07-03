import { writable } from "svelte/store";
import { getCurrentWindow } from "@tauri-apps/api/window";

let default_tilte = "Pesa Playground";
const titleStore = writable(default_tilte);

export async function setTitle(newTitle: string) {
  titleStore.set(newTitle);
  getCurrentWindow().setTitle(newTitle).catch(console.error);
}
export async function resetTitle() {
  titleStore.set(default_tilte);
  getCurrentWindow().setTitle(default_tilte).catch(console.error);
}

export const title = {
  subscribe: titleStore.subscribe,
  set: setTitle,
  update: titleStore.update
};
