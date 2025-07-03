import { writable } from "svelte/store";

export type SandboxStatus = "off" | "starting" | "on" | "error";

export const sandboxStatus = writable<SandboxStatus>("off");
