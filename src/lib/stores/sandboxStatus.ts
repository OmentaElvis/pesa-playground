import { getProject } from "$lib/api";
import { invoke } from "@tauri-apps/api/core";
import { writable } from "svelte/store";

export type SandboxStatus = "off" | "starting" | "on" | "error";

export const sandboxStatus = writable<SandboxStatus>("off");

export interface SandboxInfo {
  status: SandboxStatus,
  port: number,
  project_id: number,
  name: string,
}

export async function getSandboxes() {
  try {
    let list: SandboxInfo[] = await invoke("list_running_sandboxes");

    for (let info of list) {
      let project = await getProject(info.project_id);
      info.name = project.name;
    }

    sandboxes.update(() => list);
  } catch (err) {
    console.log(err);
  }
}

export const sandboxes = writable<SandboxInfo[]>([]);
getSandboxes();

setInterval(async () => {
  await getSandboxes();
}, 10000);
