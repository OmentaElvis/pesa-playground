<h1 align="center">Pesa playground</h1>
<p align="center">
  <i>
  A local M-Pesa ecosystem simulator built for developers who need reliable, controllable test environments.
  </i><br />
  <img src="web/static/pesaplayground_logo.png" alt="Logo" />
  <br />
  <b>
    Code. Trigger. Respond. Repeat
  </b>
</p>

<p align="center">
    <a href="https://github.com/OmentaElvis/pesa-playground/actions/workflows/build-tauri-linux.yml">
        <img src="https://github.com/OmentaElvis/pesa-playground/actions/workflows/build-tauri-linux.yml/badge.svg" alt="Build Status" />
    </a>
    <a href="https://opensource.org/licenses/BSD-2-Clause">
        <img src="https://img.shields.io/badge/License-BSD_2--Clause-orange.svg" alt="BSD 2-Clause License" />
    </a>
</p>
<p align="center">
    <a href="https://svelte.dev/">
      <img src="https://img.shields.io/badge/sveltekit-%23f1413d.svg?style=for-the-badge&logo=svelte&logoColor=white" alt="Sveltekit"/>
    </a>
    <a href="https://tauri.app/">
      <img src="https://img.shields.io/badge/tauri-%2324C8DB.svg?style=for-the-badge&logo=tauri&logoColor=%23FFFFFF" alt="Tauri"/>
    </a>
    <a href="https://rust-lang.org/learn/get-started/">
      <img src="https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white" />
    </a>
</p>

## 🦎 Overview

**Pesa Playground** emulates the core behaviour of the M-Pesa platform - users, tills, paybills, wallet balances, and STK push flows - entirely locally on your machine.
It’s a lightweight desktop tauri app with a simple API layer for automation and integration testing.

The goal is to make development around M-Pesa integrations straightforward and predictable.
You run the app, point your backend to the local endpoints, and test complete payment flows end-to-end.


## ✨ Features

- **Complete Wallet Simulation**
  Create and manage users with phone numbers, balances, and transaction histories.

- **Transfer and Payment Flows**
  Send money between users, tills, and paybills using familiar API patterns and inbuilt Sim Toolkit.

- **Interactive STK Push**
  A visual prompt in the UI lets you decide how the simulated device responds.
  Respond with correct PIN, Timeout, Offline or Cancelled prompt. So that you dont have to test these edge cases in prod.

- **Offline-First**
  Runs entirely locally - no external services, account logins or network tunnels.

- **Clear Logging**
  Every request and transaction is logged for easy debugging and traceability.
  We provide explanations for why your API request failed.

## 🚀 Installation

### Building from Source

To build Pesa Playground from source, you'll need a <a href="https://rustup.rs/">Rust toolchain</a> and <a href="https://pnpm.io/installation">pnpm installed</a>.

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/OmentaElvis/pesa-playground.git
    cd pesa-playground
    ```
2.  **Install pnpm dependencies:**

    ```bash
    pnpm install
    ```
3.  **Build the Tauri application:**

    ```bash
    pnpm tauri build
    ```
    The binaries will be located in `target/release/bundle/`.

### 🐋 Using Docker

A prebuilt Docker image is available on GitHub Container Registry.

1.  **Pull the Docker image:**

    ```bash
    docker pull ghcr.io/omentaelvis/pesa-playground:<version>
    ```
    Replace `<version>` with the desired version tag (e.g., `v0.0.1`).

2.  **Run the Docker container:**

    ```bash
    docker run --init -it --rm -p 3001:3000 -p 8001:8001 ghcr.io/omentaelvis/pesa-playground:v0.0.1
    ```
    Note: Ports `800*` are dependent on the project ID. For example, project 1 will open port `8001`.

### Prebuilt Binaries

You can download prebuilt binaries directly from the [Releases page](https://github.com/OmentaElvis/pesa-playground/releases) on GitHub.

Available formats include:

-   AppImage (Linux)
-   .deb (Linux)
-   .rpm (Linux)
-   setup.exe (Windows)


## 📝 Feature To-Do List

Core M-Pesa API coverage roadmap for **Pesa Playground**.

- ✅ **Authentication** - OAuth token generation and validation
- ✅ **STK Push (C2B Payment Simulation)** - full request/response flow with interactive prompt

#### Customer to Business (C2B)
- ✅ Register URL - simulate callback registration
- ✅ C2B Payment - simulate customer-initiated payment to paybill/till
- ✅ C2B Validation and Confirmation callbacks

#### Business to Customer (B2C)
- [ ] B2C Payment Request - simulate disbursements to mobile numbers
- [ ] Transaction Status query
- [ ] Balance query

#### Business to Business (B2B)
- [ ] B2B Payment Request - simulate transfers between business accounts
- [ ] B2B Transaction Status query
- [ ] Balance query

#### Account Management & Queries
- [ ] Reversal Request - simulate transaction reversals
- [ ] Transaction Status - verify or track payment progress
- [ ] Balance Inquiry - return current wallet or till balance

#### Utility & Simulation
- [ ] Configurable failure modes (timeout, insufficient funds, declined)
- [ ] Adjustable latency simulation
- [ ] Sandbox-to-production parity testing mode

#### User Interface
- [ ] Interactive transaction viewer
- [ ] Wallet editor (balances, user creation, history)
- ✅ STK push prompt (approve/decline/time-out)

## 🙏 Contributing

Contributions, feedback, and ideas are welcome.
Open an issue or PR to discuss improvements.
