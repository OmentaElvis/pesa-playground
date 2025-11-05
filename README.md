<h1 align="center">Pesa playground</h1>
<p align="center">
  <i>
  A local M-Pesa ecosystem simulator built for developers who need reliable, controllable test environments.
  </i><br />
  <img src="web/static/pesaplayground_logo.png" alt="Logo" />
</p>

## Overview

**Pesa Playground** emulates the core behaviour of the M-Pesa platform — users, tills, paybills, wallet balances, and STK push flows — entirely locally on your machine.  
It’s a lightweight desktop tauri app with a simple API layer for automation and integration testing.

The goal is to make development around M-Pesa integrations straightforward and predictable.  
You run the app, point your backend to the local endpoints, and test complete payment flows end-to-end.


## Features

- **Complete Wallet Simulation**  
  Create and manage users with phone numbers, balances, and transaction histories.

- **Transfer and Payment Flows**  
  Send money between users, tills, and paybills using familiar API patterns and inbuilt Sim Toolkit.

- **Interactive STK Push**  
  A visual prompt in the UI lets you decide how the simulated device responds.
  Respond with correct PIN, Timeout, Offline or Cancelled prompt. So that you dont have to test these edge cases in prod.

- **Offline-First**  
  Runs entirely locally — no external services, account logins or network tunnels.

- **Clear Logging**  
  Every request and transaction is logged for easy debugging and traceability.
  We provide explanations for why your API request failed.



##  Feature To-Do List

Core M-Pesa API coverage roadmap for **Pesa Playground**.

- [x] **Authentication** — OAuth token generation and validation  
- [x] **STK Push (C2B Payment Simulation)** — full request/response flow with interactive prompt

#### Customer to Business (C2B)
- [x] Register URL — simulate callback registration
- [x] C2B Payment — simulate customer-initiated payment to paybill/till
- [x] C2B Validation and Confirmation callbacks

#### Business to Customer (B2C)
- [ ] B2C Payment Request — simulate disbursements to mobile numbers
- [ ] Transaction Status query
- [ ] Balance query

#### Business to Business (B2B)
- [ ] B2B Payment Request — simulate transfers between business accounts
- [ ] B2B Transaction Status query
- [ ] Balance query

#### Account Management & Queries
- [ ] Reversal Request — simulate transaction reversals
- [ ] Transaction Status — verify or track payment progress
- [ ] Balance Inquiry — return current wallet or till balance

#### Utility & Simulation
- [ ] Configurable failure modes (timeout, insufficient funds, declined)
- [ ] Adjustable latency simulation
- [ ] Sandbox-to-production parity testing mode

#### User Interface
- [ ] Interactive transaction viewer
- [ ] Wallet editor (balances, user creation, history)
- [X] STK push prompt (approve/decline/time-out)

## Contributing

Contributions, feedback, and ideas are welcome.
Open an issue or PR to discuss improvements.
