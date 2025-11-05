# Pesa Playground Backend Crates

This directory contains the Rust backend for Pesa Playground, organized as a Cargo workspace with multiple crates.

## Architecture Overview

The backend is designed with a clean separation of concerns, decoupling the core business logic from the specific API or UI framework being used.

-   **`pesa-core`**: This is the heart of the application. It is a backend-agnostic library containing all the business logic, database interactions (using `sea-orm`), data structures, and service definitions. It is shared between the other backend crates.

-   **`pesa-axum`**: An adapter crate that exposes the functionality of `pesa-core` as a JSON-RPC web API using the [Axum](https://github.com/tokio-rs/axum) framework. This allows the frontend to interact with the backend when running in a standard web browser (e.g., in a Docker container).

-   **`pesa-tauri`**: Another adapter crate that bridges `pesa-core` with the [Tauri](https://tauri.app/) desktop framework. It exposes the core logic as Tauri commands that the Svelte frontend can invoke when running as a native desktop application.

-   **`pesa-macros`**: A utility crate that defines procedural macros. These macros are the key to minimizing boilerplate code in the adapter crates (`pesa-axum` and `pesa-tauri`), providing a declarative way to map core logic to the web or desktop layers.

## The Core (`pesa-core`)

All business operations are defined in `pesa-core`. Functions intended to be exposed to the outside world typically follow a standard signature:

```rust
// Example from a `ui.rs` module in pesa-core
pub async fn some_operation(
    context: &AppContext,
    arg1: Type1,
    arg2: Type2,
) -> anyhow::Result<SerializableReturnType>
```

-   `context: &AppContext`: A shared application context containing the database connection pool and an event manager.
-   `anyhow::Result`: All functions return a `Result` to handle errors gracefully.
-   `SerializableReturnType`: The success type must be serializable with `serde` so it can be sent over the API.

Some functions may not require the `AppContext` and will omit it.

## Adapters & Code Generation (`pesa-macros`)

The `pesa-axum` and `pesa-tauri` crates are intentionally lightweight. Their primary job is to translate incoming requests (HTTP or Tauri IPC) into calls to `pesa-core` functions and format the responses.

This is achieved with minimal code thanks to the `pesa-macros` crate. This crate provides macros like `generate_axum_rpc_handler` and `generate_tauri_wrappers` that automatically generate the necessary wrapper and boilerplate code.

The developer simply provides a declarative mapping:

```rust
// Example from `pesa-axum/src/main.rs`
generate_axum_rpc_handler! {
    // The name of the public-facing RPC method.
    //       |
    //       |        The arguments it accepts.
    //       |                |
    //       v                v
    create_project(input: CreateProject) => pesa_core::projects::ui::create_project,
    get_users() => pesa_core::accounts::user_profiles::ui::get_users,
    //   ^
    //   |
    //   The actual function in `pesa-core` that implements the logic.
}
```

This approach makes it trivial to expose new functionality. Once a function is written in `pesa-core`, adding a single line to the macro invocation in `pesa-axum` and/or `pesa-tauri` is all that's needed to make it accessible to the frontend.
