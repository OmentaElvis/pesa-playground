import { cubicOut } from 'svelte/easing';

export function spin(node: HTMLElement, { duration = 500, delay = 0, degrees = 360 }) {
    return {
        delay,
        duration,
        css: (t: number) => {
            const eased = cubicOut(t);
            return `
                transform: rotate(${eased * degrees}deg);
            `;
        }
    };
}
