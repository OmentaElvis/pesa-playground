import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { SimulationMode } from "./api";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, "children"> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };


export function formatPhoneNumber(phone: string) {
  if (!phone) return '';
  // Format as +254 759 289 552
  return phone.replace(/(\d{3})(\d{3})(\d{3})(\d{3})/, '+$1 $2 $3 $4');
}

export function formatAmount(amount: number) {
  return new Intl.NumberFormat('en-KE', {
    style: 'currency',
    currency: 'KES',
    minimumFractionDigits: 0
  }).format(amount);
}

export function getInitials(fullName: string) {
    const nameParts = fullName.trim().split(' ');
    const initials = nameParts.map(part => part.charAt(0).toUpperCase()).join('');

    return initials;
}

export function formatDate(dateString: string) {
    const options: Intl.DateTimeFormatOptions = { 
        year: 'numeric', 
        month: 'long', 
        day: 'numeric', 
        hour: '2-digit', 
        minute: '2-digit', 
        second: '2-digit', 
    };
    const date = new Date(dateString);
    return date.toLocaleString(undefined, options);
}

export function getSimulationModeColor(mode: SimulationMode) {
  switch (mode) {
    case SimulationMode.AlwaysSuccess: return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300';
    case SimulationMode.AlwaysFail: return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-300';
    case SimulationMode.Random: return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300';
    case SimulationMode.Realistic: return 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-300';
    default: return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-300';
  }
}
