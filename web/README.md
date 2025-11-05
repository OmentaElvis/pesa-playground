# Pesa Playground Frontend

This directory contains the frontend code for Pesa Playground, built with SvelteKit.

## Architecture

The frontend is a Single Page Application (SPA) built using SvelteKit with `adapter-static`. This allows it to be served as a collection of static files, making it compatible with various hosting environments, including the Tauri desktop application and a standard web server with docker.

The application has two primary build modes:

1.  **Tauri Mode**: This is the default mode, configured to build the frontend for integration with the [Tauri](https://tauri.app/) desktop shell. In this mode, the application communicates with the Rust backend through the Tauri API.

2.  **Web Mode**: This mode builds the application as a standard web application that can be served by a web server (like the Axum server in `../crates/pesa-axum`). This is ideal for browser-based access and containerized deployments (e.g., using Docker).

## Key Technologies & Libraries

*   **Framework**: [SvelteKit](https://kit.svelte.dev/)
*   **UI Components**: [Shadcn-Svelte](https://www.shadcn-svelte.com/)
*   **Icons**: [Lucide Svelte](https://lucide.dev/guide/packages/lucide-svelte)
*   **Styling**: [Tailwind CSS](https://tailwindcss.com/)

## Available Scripts

The following scripts are available in `package.json` to manage the development and build process:

### Development

*   `pnpm tauri dev`: Starts the Tauri development environment. This is the primary command for desktop app development. It automatically handles starting the SvelteKit dev server.
*   `pnpm vite:web`: Runs the SvelteKit development server in **Web mode** for browser-based development.
*   `pnpm dev:backend`: Starts the Axum backend server and watches for changes in the Rust crates.
*   `pnpm dev:web`: A convenience script that runs both the frontend (`vite:web`) and the backend (`dev:backend`) concurrently for web-only development.

### Building

*   `pnpm tauri build`: Creates a production build of the Tauri application.
*   `pnpm build:web`: Creates a production build of the application in **Web mode** for browser/docker deployment.

### Other

*   `pnpm preview`: Starts a local server to preview the latest production build.
*   `pnpm check`: Runs Svelte's type checker to validate the code.
*   `pnpm check:watch`: Runs the type checker in watch mode.
*   `pnpm tauri <command>`: A wrapper for the Tauri CLI. Use this to run other Tauri commands (e.g., `pnpm tauri info`).
